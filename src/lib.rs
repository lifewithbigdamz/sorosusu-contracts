#![no_std]

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, event, panic_with_error, symbol_short,
    Address, Env, Vec,
};

const MAX_MEMBERS: u32 = 50;

#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    Circle(u32),
    CircleCount,
}

#[derive(Clone)]
#[contracttype]
pub struct Circle {
    admin: Address,
    contribution: i128,
    members: Vec<Address>,
    cycle_number: u32,
    current_payout_index: u32,
    has_received_payout: Vec<bool>,
    total_volume_distributed: i128,
}

#[derive(Clone)]
#[contracttype]
pub struct CycleCompletedEvent {
    group_id: u32,
    total_volume_distributed: i128,
}

#[derive(Clone)]
#[contracttype]
pub struct GroupRolloverEvent {
    group_id: u32,
    new_cycle_number: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[contracterror]
pub enum Error {
    CycleNotComplete = 1001,
    InsufficientAllowance = 1002,
    AlreadyJoined = 1003,
    CircleNotFound = 1004,
    Unauthorized = 1005,
    MaxMembersReached = 1006,
    CircleNotFinalized = 1007,
}

#[contract]
pub struct SoroSusu;

fn read_circle(env: &Env, id: u32) -> Circle {
    let key = DataKey::Circle(id);
    let storage = env.storage().instance();
    match storage.get(&key) {
        Some(circle) => circle,
        None => panic_with_error!(env, Error::CircleNotFound),
    }
}

fn write_circle(env: &Env, id: u32, circle: &Circle) {
    let key = DataKey::Circle(id);
    let storage = env.storage().instance();
    storage.set(&key, circle);
}

fn next_circle_id(env: &Env) -> u32 {
    let key = DataKey::CircleCount;
    let storage = env.storage().instance();
    let current: u32 = storage.get(&key).unwrap_or(0);
    let next = current.saturating_add(1);
    storage.set(&key, &next);
    next
}

#[contractimpl]
impl SoroSusu {
    pub fn create_circle(env: Env, contribution: i128, is_random_queue: bool) -> u32 {
        let admin = env.invoker();
        let id = next_circle_id(&env);
        let members = Vec::new(&env);
        let has_received_payout = Vec::new(&env);
        let circle = Circle {
            admin,
            contribution,
            members,
            cycle_number: 1,
            current_payout_index: 0,
            has_received_payout,
            total_volume_distributed: 0,
        };
        write_circle(&env, id, &circle);
        id
    }

    pub fn join_circle(env: Env, circle_id: u32) {
        let invoker = env.invoker();
        let mut circle = read_circle(&env, circle_id);
        for member in circle.members.iter() {
            if member == invoker {
                panic_with_error!(&env, Error::AlreadyJoined);
            }
        }
        let member_count: u32 = circle.members.len();
        if member_count >= MAX_MEMBERS {
            panic_with_error!(&env, Error::MaxMembersReached);
        }
        circle.members.push_back(invoker);
        circle.has_received_payout.push_back(false);
        write_circle(&env, circle_id, &circle);
    }

    pub fn process_payout(env: Env, circle_id: u32, recipient: Address) {
        let mut circle = read_circle(&env, circle_id);

        // Only admin can process payouts
        if env.invoker() != circle.admin {
            panic_with_error!(&env, Error::Unauthorized);
        }

        // Check if recipient is a member
        let mut member_index = None;
        for (i, member) in circle.members.iter().enumerate() {
            if member == recipient {
                member_index = Some(i);
                break;
            }
        }

        if member_index.is_none() {
            panic_with_error!(&env, Error::Unauthorized);
        }

        let index = member_index.unwrap();

        // Check if member has already received payout for current cycle
        if circle.has_received_payout.get(index).unwrap_or(&false) == &true {
            panic_with_error!(&env, Error::Unauthorized);
        }

        // Mark as received
        circle.has_received_payout.set(index, true);
        circle.current_payout_index += 1;

        // Add to total volume distributed
        circle.total_volume_distributed += circle.contribution;

        // Check if this was the last payout for the cycle
        let all_paid = circle.has_received_payout.iter().all(|&paid| paid);

        if all_paid {
            // Emit CycleCompleted event
            let event = CycleCompletedEvent {
                group_id: circle_id,
                total_volume_distributed: circle.total_volume_distributed,
            };
            event::publish(&env, symbol_short!("CYCLE_COMP"), &event);
        }

        write_circle(&env, circle_id, &circle);
    }

    pub fn rollover_group(env: Env, circle_id: u32) {
        let mut circle = read_circle(&env, circle_id);

        // Only admin can rollover the group
        if env.invoker() != circle.admin {
            panic_with_error!(&env, Error::Unauthorized);
        }

        // Check if all members have received payout for current cycle
        for received in circle.has_received_payout.iter() {
            if !received {
                panic_with_error!(&env, Error::CycleNotComplete);
            }
        }

        // Reset for next cycle
        circle.cycle_number += 1;
        circle.current_payout_index = 0;

        // Reset payout flags
        for i in 0..circle.has_received_payout.len() {
            circle.has_received_payout.set(i, false);
        }

        // Reset volume for new cycle
        circle.total_volume_distributed = 0;

        // Emit GroupRollover event
        let event = GroupRolloverEvent {
            group_id: circle_id,
            new_cycle_number: circle.cycle_number,
        };
        event::publish(&env, symbol_short!("GROUP_ROLL"), &event);

        write_circle(&env, circle_id, &circle);
    }

    pub fn get_cycle_info(env: Env, circle_id: u32) -> (u32, u32, i128) {
        let circle = read_circle(&env, circle_id);
        (
            circle.cycle_number,
            circle.current_payout_index,
            circle.total_volume_distributed,
        )
    }

    pub fn get_payout_status(env: Env, circle_id: u32) -> Vec<bool> {
        let circle = read_circle(&env, circle_id);
        circle.has_received_payout
    }
}

#[cfg(test)]
mod test {
    extern crate std;

    use super::*;
    use soroban_sdk::testutils::{Address as _, Env as _};

    #[test]
    fn join_circle_enforces_max_members() {
        let env = Env::default();
        let contract_id = env.register_contract(None, SoroSusu);
        let client = SoroSusuClient::new(&env, &contract_id);
        let contribution = 10_i128;
        let circle_id = client.create_circle(&contribution, &false);

        for _ in 0..MAX_MEMBERS {
            let member = Address::generate(&env);
            client.join_circle(&circle_id);
        }

        let extra_member = Address::generate(&env);
        let result = std::panic::catch_unwind(|| {
            client.join_circle(&circle_id);
        });
        assert!(result.is_err());
    }

    #[test]
    fn test_process_payout_and_cycle_completion() {
        let env = Env::default();
        let contract_id = env.register_contract(None, SoroSusu);
        let client = SoroSusuClient::new(&env, &contract_id);
        let contribution = 100_i128;

        // Create circle and add members
        let circle_id = client.create_circle(&contribution);
        let members: Vec<Address> = (0..3).map(|_| Address::generate(&env)).collect();

        for member in &members {
            client.join_circle(&circle_id);
        }

        // Process payouts for all members
        for member in &members {
            client.process_payout(&circle_id, member);
        }

        // Verify cycle info
        let (cycle_num, payout_index, total_volume) = client.get_cycle_info(&circle_id);
        assert_eq!(cycle_num, 1);
        assert_eq!(payout_index, 3);
        assert_eq!(total_volume, 300_i128);

        // Check that events were emitted
        let events = env.events().all();
        assert_eq!(events.len(), 1); // One CycleCompleted event

        let event = &events[0];
        assert_eq!(event.0, symbol_short!("CYCLE_COMP"));
    }

    #[test]
    fn test_group_rollover() {
        let env = Env::default();
        let contract_id = env.register_contract(None, SoroSusu);
        let client = SoroSusuClient::new(&env, &contract_id);
        let contribution = 50_i128;

        // Create circle and add members
        let circle_id = client.create_circle(&contribution);
        let members: Vec<Address> = (0..2).map(|_| Address::generate(&env)).collect();

        for member in &members {
            client.join_circle(&circle_id);
        }

        // Process all payouts
        for member in &members {
            client.process_payout(&circle_id, member);
        }

        // Clear events to test rollover event
        env.events().all();

        // Perform rollover
        client.rollover_group(&circle_id);

        // Verify new cycle info
        let (cycle_num, payout_index, total_volume) = client.get_cycle_info(&circle_id);
        assert_eq!(cycle_num, 2);
        assert_eq!(payout_index, 0);
        assert_eq!(total_volume, 0_i128);

        // Check that rollover event was emitted
        let events = env.events().all();
        assert_eq!(events.len(), 1);

        let event = &events[0];
        assert_eq!(event.0, symbol_short!("GROUP_ROLL"));
    }

    #[test]
    fn test_payout_unauthorized() {
        let env = Env::default();
        let contract_id = env.register_contract(None, SoroSusu);
        let client = SoroSusuClient::new(&env, &contract_id);
        let contribution = 10_i128;

        let circle_id = client.create_circle(&contribution);
        let member = Address::generate(&env);
        client.join_circle(&circle_id);

        // Try to process payout with non-admin
        let unauthorized_user = Address::generate(&env);
        env.set_source_account(&unauthorized_user);

        let result = std::panic::catch_unwind(|| {
            client.process_payout(&circle_id, &member);
        });
        assert!(result.is_err());
    }

    #[test]
    fn test_rollover_before_cycle_complete() {
        let env = Env::default();
        let contract_id = env.register_contract(None, SoroSusu);
        let client = SoroSusuClient::new(&env, &contract_id);
        let contribution = 10_i128;

        let circle_id = client.create_circle(&contribution);
        let member = Address::generate(&env);
        client.join_circle(&circle_id);

        // Try to rollover without completing payouts
        let result = std::panic::catch_unwind(|| {
            client.rollover_group(&circle_id);
        });
        assert!(result.is_err());
    }

    #[test]
    fn test_duplicate_payout() {
        let env = Env::default();
        let contract_id = env.register_contract(None, SoroSusu);
        let client = SoroSusuClient::new(&env, &contract_id);
        let contribution = 10_i128;

        let circle_id = client.create_circle(&contribution);
        let member = Address::generate(&env);
        client.join_circle(&circle_id);

        // Process payout once
        client.process_payout(&circle_id, &member);

        // Try to process payout again for same member
        let result = std::panic::catch_unwind(|| {
            client.process_payout(&circle_id, &member);
        });
        assert!(result.is_err());
    }
}

#![no_std]
#![cfg_attr(test, allow(dead_code))]

use soroban_sdk::{
    contract, contracterror, contractimpl, contractmeta, contracttype, panic_with_error,


#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    LastActiveTimestamp,
    UserBalance(Address),
    Admin,
    Circle(u32),
    CircleCount,
}

#[derive(Clone)]
#[contracttype]
pub struct Circle {

}

#[derive(Clone)]
#[contracttype]
pub struct CycleCompletedEvent {
    pub group_id: u32,
    pub total_volume_distributed: i128,
}

#[derive(Clone)]
#[contracttype]
pub struct GroupRolloverEvent {
    pub group_id: u32,
    pub new_cycle_number: u32,
}

#[derive(Clone)]
#[contracttype]
pub struct LateJoinerCaughtUpEvent {
    pub member_address: Address,
    pub amount_paid: i128,
}

#[contracterror]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum Error {
    CycleNotComplete = 1001,
    InsufficientAllowance = 1002,
    AlreadyJoined = 1003,

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
    env.storage().instance().set(&key, circle);
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

        let circle = Circle {
            admin,
            contribution,
            members: Vec::new(&env),
            is_random_queue,

        };
        write_circle(&env, id, &circle);
        id
    }

    pub fn join_circle(env: Env, invoker: Address, circle_id: u32) {
        invoker.require_auth();
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

main
        circle.has_received_payout.push_back(false);
        write_circle(&env, circle_id, &circle);
    }


        let mut circle = read_circle(&env, circle_id);


        for (i, member) in circle.members.iter().enumerate() {
            if member == recipient {
                member_index = Some(i as u32);
                break;
            }
        }

        let index = match member_index {
            Some(i) => i,
            None => panic_with_error!(&env, Error::NotMember),
        };


        if all_paid {
            let event = CycleCompletedEvent {
                group_id: circle_id,
                total_volume_distributed: circle.total_volume_distributed,
            };

        }

        write_circle(&env, circle_id, &circle);
    }

    pub fn rollover_group(env: Env, admin: Address, circle_id: u32) {
        admin.require_auth();
        let mut circle = read_circle(&env, circle_id);



        for received in circle.has_received_payout.iter() {
            if !received {
                panic_with_error!(&env, Error::CycleNotComplete);
            }
        }

        circle.cycle_number += 1;
        circle.current_payout_index = 0;
        circle.total_volume_distributed = 0;


        let event = GroupRolloverEvent {
            group_id: circle_id,
            new_cycle_number: circle.cycle_number,
        };

        write_circle(&env, circle_id, &circle);
    }

    pub fn finalize_circle(env: Env, admin: Address, circle_id: u32) {
        admin.require_auth();
        let mut circle = read_circle(&env, circle_id);



        if !circle.payout_queue.is_empty() {
            return;
        }

        if circle.is_random_queue {

        } else {
            circle.payout_queue = circle.members.clone();
        }

        write_circle(&env, circle_id, &circle);
    }

    // ============================================================
    // VIEW GETTERS & HELPERS
    // ============================================================

    pub fn get_payout_queue(env: Env, circle_id: u32) -> Vec<Address> {
        let circle = read_circle(&env, circle_id);
        circle.payout_queue
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

    fn require_admin(env: &Env) -> Result<(), Error> {
        let admin: Address = env
            .storage()
            .instance()
            .get(&ADMIN_KEY)
            .ok_or(Error::Unauthorized)?;
        admin.require_auth();
        Ok(())
    }
}

contractmeta!(
    key = "Description",
    val = "SoroSusu ROSCA protocol with protocol payout fee"
);

#[contract]
pub struct SorosusuContracts;

#[contractimpl]
impl SorosusuContracts {
    pub fn initialize(env: Env, admin: Address) -> Result<(), Error> {
        if env.storage().instance().has(&ADMIN_KEY) {
            return Err(Error::Unauthorized);
        }
        env.storage().instance().set(&ADMIN_KEY, &admin);
        env.storage().instance().set(&FEE_BASIS_POINTS_KEY, &0u32);
        env.storage().instance().set(&IS_PUBLIC_KEY, &true);
        
        let empty_members: Vec<Address> = Vec::new(&env);
        let empty_contribs: Map<Address, i128> = Map::new(&env);
        env.storage().instance().set(&MEMBERS_KEY, &empty_members);
        env.storage().instance().set(&CONTRIBS_KEY, &empty_contribs);
        
        Ok(())
    }

    pub fn set_protocol_fee(
        env: Env,
        fee_basis_points: u32,
        treasury: Address,
    ) -> Result<(), Error> {
        Self::require_admin(&env)?;
        if fee_basis_points > MAX_BASIS_POINTS {
            return Err(Error::InvalidFeeConfig);
        }
        env.storage().instance().set(&FEE_BASIS_POINTS_KEY, &fee_basis_points);
        env.storage().instance().set(&TREASURY_KEY, &treasury);
        Ok(())
    }

    pub fn fee_basis_points(env: Env) -> u32 {
        env.storage()
            .instance()
            .get::<_, u32>(&FEE_BASIS_POINTS_KEY)
            .unwrap_or(0)
    }

    pub fn treasury_address(env: Env) -> Option<Address> {
        env.storage().instance().get::<_, Address>(&TREASURY_KEY)
    }

    pub fn compute_and_transfer_payout(
        env: Env,
        token: Address,
        from: Address,
        recipient: Address,
        gross_payout: i128,
    ) -> Result<(), Error> {
        let fee_bps = env.storage().instance().get::<_, u32>(&FEE_BASIS_POINTS_KEY).unwrap_or(0);
        let fee = if fee_bps == 0 {
            0_i128
        } else {
            (gross_payout as i128 * fee_bps as i128) / MAX_BASIS_POINTS as i128
        };
        let net_payout = gross_payout - fee;

        let token_client = token::Client::new(&env, &token);
        token_client.transfer(&from, &recipient, &net_payout);

        if fee > 0 {
            let treasury: Address = env
                .storage()
                .instance()
                .get::<_, Address>(&TREASURY_KEY)
                .ok_or(Error::InvalidFeeConfig)?;
            token_client.transfer(&from, &treasury, &fee);
        }

        Ok(())
    }

    pub fn kick_member(
        env: Env,
        token: Address,
        member: Address,
        penalty: i128,
    ) -> Result<(), Error> {
        Self::require_admin(&env)?;

        let mut members: Vec<Address> = env.storage().instance().get(&MEMBERS_KEY).unwrap_or(Vec::new(&env));
        let mut member_index = None;
        
        for (i, m) in members.iter().enumerate() {
            if m == member {
                member_index = Some(i as u32);
                break;
            }
        }

        let index = member_index.ok_or(Error::MemberNotFound)?;

        let mut contribs: Map<Address, i128> = env.storage().instance().get(&CONTRIBS_KEY).unwrap_or(Map::new(&env));
        let total_contributed = contribs.get(member.clone()).unwrap_or(0);

        if total_contributed < penalty {
            return Err(Error::PenaltyExceedsContribution);
        }

        members.remove(index);
        contribs.remove(member.clone());
        
        env.storage().instance().set(&MEMBERS_KEY, &members);
        env.storage().instance().set(&CONTRIBS_KEY, &contribs);

        let refund = total_contributed - penalty;
        let contract_address = env.current_contract_address();
        let token_client = token::Client::new(&env, &token);

        if refund > 0 {
            token_client.transfer(&contract_address, &member, &refund);
        }

        if penalty > 0 {
            if let Some(treasury) = Self::treasury_address(env.clone()) {
                token_client.transfer(&contract_address, &treasury, &penalty);
            }
        }

        env.events().publish(
            (symbol_short!("Kicked"), member.clone()),
            (refund, penalty),
        );

        Ok(())
    }

    pub fn join(env: Env, member: Address, invite_code: Option<u64>) -> Result<(), Error> {
        member.require_auth();

        let mut count: u32 = env.storage().instance().get(&MEMBERS_COUNT_KEY).unwrap_or(0);
        if count >= MAX_MEMBERS {
            return Err(Error::MemberLimitExceeded);
        }

        let mut members: Vec<Address> = env.storage().instance().get(&MEMBERS_KEY).unwrap_or(Vec::new(&env));
        if members.contains(&member) {
            return Err(Error::AlreadyJoined);
        }

        let is_public: bool = env.storage().instance().get(&IS_PUBLIC_KEY).unwrap_or(true);

        if !is_public {
            let mut authorized_by_code = false;
            if let Some(code) = invite_code {
                if let Some(expected_code) = env.storage().instance().get::<_, u64>(&INVITE_CODE_KEY) {
                    if code == expected_code {
                        authorized_by_code = true;
                    }
                }
            }
            if !authorized_by_code {
                Self::require_admin(&env)?;
            }
        }

        members.push_back(member.clone());
        env.storage().instance().set(&MEMBERS_KEY, &members);

        count += 1;
        env.storage().instance().set(&MEMBERS_COUNT_KEY, &count);

        let mut contribs: Map<Address, i128> = env.storage().instance().get(&CONTRIBS_KEY).unwrap_or(Map::new(&env));
        contribs.set(member, 0);
        env.storage().instance().set(&CONTRIBS_KEY, &contribs);

        Ok(())
    }

    pub fn member_count(env: Env) -> u32 {
        env.storage().instance().get(&MEMBERS_COUNT_KEY).unwrap_or(0)
    }

    fn require_admin(env: &Env) -> Result<(), Error> {
        let admin: Address = env
            .storage()
            .instance()
            .get(&ADMIN_KEY)
            .ok_or(Error::Unauthorized)?;
        admin.require_auth();
        Ok(())
    }
    
    pub fn set_privacy_config(
        env: Env,
        is_public: bool,
        invite_code: Option<u64>,
    ) -> Result<(), Error> {
        Self::require_admin(&env)?;
        env.storage().instance().set(&IS_PUBLIC_KEY, &is_public);
        if let Some(code) = invite_code {
            env.storage().instance().set(&INVITE_CODE_KEY, &code);
        } else {
            env.storage().instance().remove(&INVITE_CODE_KEY);
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {
    extern crate std;
    use super::*;

        assert!(result.is_err());
    }

    #[test]
    fn test_random_queue_finalization() {

        }
    }


        }
    }

    #[test]
    fn test_process_payout_and_cycle_completion() {

        }

        let (cycle_num, payout_index, total_volume) = client.get_cycle_info(&circle_id);
        assert_eq!(cycle_num, 1);
        assert_eq!(payout_index, 3);
        assert_eq!(total_volume, 300_i128);

        let events = env.events().all();

        let (cycle_num, payout_index, total_volume) = client.get_cycle_info(&circle_id);
        assert_eq!(cycle_num, 2);
        assert_eq!(payout_index, 0);
        assert_eq!(total_volume, 0_i128);
    }

    #[test]
    fn test_payout_unauthorized() {

        assert!(result.is_err());
    }

    #[test]
    fn test_duplicate_payout() {
ssert!(result.is_err());
    }


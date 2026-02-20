#![no_std]
use soroban_sdk::{contract, contracttype, contractimpl, Address, Env, Vec, Symbol, token};

// --- DATA STRUCTURES ---

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Admin,
    Circle(u64),
    Member(Address),
    CircleCount,
    // New: Tracks if a user has paid for a specific circle (CircleID, UserAddress)
    Deposit(u64, Address), 
}

#[contracttype]
#[derive(Clone)]
pub struct Member {
    pub address: Address,
    pub has_contributed: bool,
    pub contribution_count: u32,
    pub last_contribution_time: u64,
}

#[contracttype]
#[derive(Clone)]
pub struct CircleInfo {
    pub id: u64,
    pub creator: Address,
    pub contribution_amount: u64, // Optimized from i128 to u64
    pub max_members: u16, // Optimized from u32 to u16
    pub member_count: u16, // Track count separately from Vec
    pub current_recipient_index: u16, // Track by index instead of Address
    pub is_active: bool,
    pub token: Address, // The token used (USDC, XLM)
}

// --- CONTRACT TRAIT ---

pub trait SoroSusuTrait {
    // Initialize the contract
    fn init(env: Env, admin: Address);
    
    // Create a new savings circle
    fn create_circle(env: Env, creator: Address, amount: u64, max_members: u16, token: Address) -> u64;

    // Join an existing circle
    fn join_circle(env: Env, user: Address, circle_id: u64);

    // Make a deposit (Pay your weekly/monthly due)
    fn deposit(env: Env, user: Address, circle_id: u64);
}

// --- IMPLEMENTATION ---

#[contract]
pub struct SoroSusu;

#[contractimpl]
impl SoroSusuTrait for SoroSusu {
    fn init(env: Env, admin: Address) {
        // Initialize the circle counter to 0 if it doesn't exist
        if !env.storage().instance().has(&DataKey::CircleCount) {
            env.storage().instance().set(&DataKey::CircleCount, &0u64);
        }
        // Set the admin
        env.storage().instance().set(&DataKey::Admin, &admin);
    }

    fn create_circle(env: Env, creator: Address, amount: u64, max_members: u16, token: Address) -> u64 {
        // 1. Get the current Circle Count
        let mut circle_count: u64 = env.storage().instance().get(&DataKey::CircleCount).unwrap_or(0);
        
        // 2. Increment the ID for the new circle
        circle_count += 1;

        // 3. Create the Circle Data Struct
        let new_circle = CircleInfo {
            id: circle_count,
            creator: creator.clone(),
            contribution_amount: amount,
            max_members,
            member_count: 0,
            current_recipient_index: 0,
            is_active: true,
            token,
        };

        // 4. Save the Circle and the new Count
        env.storage().instance().set(&DataKey::Circle(circle_count), &new_circle);
        env.storage().instance().set(&DataKey::CircleCount, &circle_count);

        // 5. Return the new ID
        circle_count
    }

    fn join_circle(env: Env, user: Address, circle_id: u64) {
        // 1. Authorization: The user MUST sign this transaction
        user.require_auth();

        // 2. Retrieve the circle data
        let mut circle: CircleInfo = env.storage().instance().get(&DataKey::Circle(circle_id)).unwrap();

        // 3. Check if the circle is full
        if circle.member_count >= circle.max_members {
            panic!("Circle is full");
        }

        // 4. Check if user is already a member to prevent duplicates
        let member_key = DataKey::Member(user.clone());
        if env.storage().instance().has(&member_key) {
            panic!("User is already a member");
        }

        // 5. Create and store the new member
        let new_member = Member {
            address: user.clone(),
            has_contributed: false,
            contribution_count: 0,
            last_contribution_time: 0,
        };
        
        // 6. Store the member and update circle count
        env.storage().instance().set(&member_key, &new_member);
        circle.member_count += 1;
        
        // 7. Save the updated circle back to storage
        env.storage().instance().set(&DataKey::Circle(circle_id), &circle);
    }

    fn deposit(env: Env, user: Address, circle_id: u64) {
        // 1. Authorization: The user must sign this!
        user.require_auth();

        // 2. Load the Circle Data
        let circle: CircleInfo = env.storage().instance().get(&DataKey::Circle(circle_id)).unwrap();

        // 3. Check if user is actually a member
        let member_key = DataKey::Member(user.clone());
        let mut member: Member = env.storage().instance().get(&member_key)
            .unwrap_or_else(|| panic!("User is not a member of this circle"));

        // 4. Create the Token Client
        let client = token::Client::new(&env, &circle.token);

        // 5. Transfer the Money
        client.transfer(
            &user, 
            &env.current_contract_address(), 
            &circle.contribution_amount
        );

        // 6. Update member contribution info
        member.has_contributed = true;
        member.contribution_count += 1;
        member.last_contribution_time = env.ledger().timestamp();
        
        // 7. Save updated member info
        env.storage().instance().set(&member_key, &member);

        // 8. Mark as Paid in the old format for backward compatibility
        env.storage().instance().set(&DataKey::Deposit(circle_id, user), &true);
    }
}
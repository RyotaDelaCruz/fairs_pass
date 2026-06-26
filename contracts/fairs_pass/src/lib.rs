#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, Address, Env, Symbol};

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DataKey {
    Admin,
    Coordinator(Address),
    ResidentClaim(Address),
}

#[contract]
pub struct FairsPassContract;

#[contractimpl]
impl FairsPassContract {
    // Initializes the contract and sets the main local government administrator account
    pub fn initialize(env: Env, admin: Address) {
        if env.storage().instance().has(&DataKey::Admin) {
            panic!("Contract is already initialized");
        }
        env.storage().instance().set(&DataKey::Admin, &admin);
    }

    // Authorizes a local municipal relief coordinator or barangay official
    pub fn add_coordinator(env: Env, admin: Address, coordinator: Address) {
        admin.require_auth();
        let stored_admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        if admin != stored_admin {
            panic!("Only the assigned administrator can add coordinators");
        }
        env.storage().instance().set(&DataKey::Coordinator(coordinator), &true);
    }

    // Records an aid item handover, verifying identity and preventing duplicate distribution
    pub fn distribute_aid(env: Env, coordinator: Address, resident: Address, batch_id: Symbol) {
        coordinator.require_auth();
        
        // Ensure the caller is an approved local relief worker
        if !env.storage().instance().has(&DataKey::Coordinator(coordinator.clone())) {
            panic!("Caller is not an authorized relief coordinator");
        }

        let claim_key = DataKey::ResidentClaim(resident.clone());
        
        // Verify that this specific household address has not claimed a pack in this batch yet
        if env.storage().persistent().has(&claim_key) {
            let last_claimed_batch: Symbol = env.storage().persistent().get(&claim_key).unwrap();
            if last_claimed_batch == batch_id {
                panic!("This household has already claimed their relief package for this batch");
            }
        }

        // Commit the transaction to the immutable ledger
        env.storage().persistent().set(&claim_key, &batch_id);
    }

    // Public view function to verify an individual household's current claim status
    pub fn get_claim_status(env: Env, resident: Address) -> Symbol {
        let claim_key = DataKey::ResidentClaim(resident);
        if env.storage().persistent().has(&claim_key) {
            env.storage().persistent().get(&claim_key).unwrap()
        } else {
            Symbol::new(&env, "none")
        }
    }
}
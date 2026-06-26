#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::{testutils::Address as _, Address, Env, Symbol};

    // Helper struct to group setup components together cleanly
    struct TestEnv {
        env: Env,
        contract_id: Address,
        client: FairsPassContractClient<'static>,
        admin: Address,
        coordinator: Address,
        resident: Address,
    }

    fn setup_test() -> TestEnv {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register_contract(None, FairsPassContract);
        let client = FairsPassContractClient::new(&env, &contract_id);

        let admin = Address::generate(&env);
        let coordinator = Address::generate(&env);
        let resident = Address::generate(&env);

        client.initialize(&admin);

        TestEnv {
            env,
            contract_id,
            client,
            admin,
            coordinator,
            resident,
        }
    }

    #[test]
    fn test_happy_path_distribution() {
        let t = setup_test();
        t.client.add_coordinator(&t.admin, &t.coordinator);
        
        let batch_id = Symbol::new(&t.env, "batch_01");
        t.client.distribute_aid(&t.coordinator, &t.resident, &batch_id);

        assert_eq!(t.client.get_claim_status(&t.resident), batch_id);
    }

    #[test]
    #[should_panic(expected = "Caller is not an authorized relief coordinator")]
    fn test_unauthorized_coordinator_failure() {
        let t = setup_test();
        let rogue_caller = Address::generate(&t.env);
        let batch_id = Symbol::new(&t.env, "batch_01");

        t.client.distribute_aid(&rogue_caller, &t.resident, &batch_id);
    }

    #[test]
    #[should_panic(expected = "This household has already claimed their relief package for this batch")]
    fn test_duplicate_claim_prevention() {
        let t = setup_test();
        t.client.add_coordinator(&t.admin, &t.coordinator);
        
        let batch_id = Symbol::new(&t.env, "batch_01");
        t.client.distribute_aid(&t.coordinator, &t.resident, &batch_id);
        
        // Attempting to claim a second time within the exact same distribution run
        t.client.distribute_aid(&t.coordinator, &t.resident, &batch_id);
    }

    #[test]
    fn test_state_verification_after_log() {
        let t = setup_test();
        t.client.add_coordinator(&t.admin, &t.coordinator);
        
        let batch_1 = Symbol::new(&t.env, "batch_01");
        let batch_2 = Symbol::new(&t.env, "batch_02");

        t.client.distribute_aid(&t.coordinator, &t.resident, &batch_1);
        assert_eq!(t.client.get_claim_status(&t.resident), batch_1);

        // Verify that updating to a completely new relief run overwrites state properly
        t.client.distribute_aid(&t.coordinator, &t.resident, &batch_2);
        assert_eq!(t.client.get_claim_status(&t.resident), batch_2);
    }

    #[test]
    fn test_empty_initial_state_returns_none() {
        let t = setup_test();
        let brand_new_resident = Address::generate(&t.env);
        
        assert_eq!(
            t.client.get_claim_status(&brand_new_resident), 
            Symbol::new(&t.env, "none")
        );
    }
}
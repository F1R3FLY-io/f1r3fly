#[cfg(test)]
mod tests {
    use rspace_plus_plus::setup::Setup;

    //memseq
    #[test]
    fn memseq_test_consume_persist_existing_matches() {
        let setup = Setup::new();
        let rspace = setup.rspace;

        let retrieve1 = Setup::create_retrieve(
            String::from("friends"),
            setup.alice.clone(),
            Setup::get_city_field(setup.alice.clone()),
        );
        let retrieve2 = Setup::create_retrieve(
            String::from("friends"),
            setup.bob.clone(),
            Setup::get_city_field(setup.bob),
        );
        let retrieve3 = Setup::create_retrieve(
            String::from("friends"),
            setup.alice.clone(),
            Setup::get_city_field(setup.alice),
        );
        let commit1 = Setup::create_commit(
            vec![String::from("friends")],
            vec![setup.city_match_case.clone()],
            String::from("I am the continuation, for now..."),
        );
        let commit2 = Setup::create_commit(
            vec![String::from("friends")],
            vec![setup.city_match_case.clone()],
            String::from("I am the continuation, for now..."),
        );
        let commit3 = Setup::create_commit(
            vec![String::from("friends")],
            vec![setup.city_match_case],
            String::from("I am the continuation, for now..."),
        );

        let _pres1 = rspace.get_once_non_durable_sequential(retrieve1);
        let _pres2 = rspace.get_once_non_durable_sequential(retrieve2);
        let cres1 = rspace.put_always_non_durable_sequential(commit1);
        assert_eq!(cres1.unwrap().len(), 1);
        assert!(!rspace.is_memseq_empty());

        let cres2 = rspace.put_always_non_durable_sequential(commit2);

        assert_eq!(cres2.unwrap().len(), 1);
        assert!(rspace.is_memseq_empty());

        let cres3 = rspace.put_always_non_durable_sequential(commit3);

        assert!(cres3.is_none());
        assert!(!rspace.is_memseq_empty());

        let pres3 = rspace.get_once_non_durable_sequential(retrieve3);

        assert!(pres3.is_some());
        assert!(!rspace.is_memseq_empty());

        let _ = rspace.clear_store();
    }

    #[test]
    fn memseq_test_multiple_channels_consume_match() {
        let setup = Setup::new();
        let rspace = setup.rspace;

        let retrieve1 = Setup::create_retrieve(
            String::from("colleagues"),
            setup.dan.clone(),
            Setup::get_state_field(setup.dan),
        );
        let retrieve2 = Setup::create_retrieve(
            String::from("friends"),
            setup.erin.clone(),
            Setup::get_state_field(setup.erin),
        );
        let commit = Setup::create_commit(
            vec![String::from("friends"), String::from("colleagues")],
            vec![setup.state_match_case.clone(), setup.state_match_case],
            String::from("I am the continuation, for now..."),
        );

        let pres1 = rspace.get_once_non_durable_sequential(retrieve1);
        let pres2 = rspace.get_once_non_durable_sequential(retrieve2);

        let cres = rspace.put_once_non_durable_sequential(commit);

        assert!(pres1.is_none());
        assert!(pres2.is_none());
        assert!(cres.is_some());
        assert_eq!(cres.unwrap().len(), 2);
        assert!(rspace.is_memseq_empty());

        let _ = rspace.clear_store();
    }

    #[test]
    fn memseq_test_consume_match() {
        let setup = Setup::new();
        let rspace = setup.rspace;

        let retrieve = Setup::create_retrieve(
            String::from("friends"),
            setup.bob.clone(),
            Setup::get_last_name_field(setup.bob),
        );
        let commit = Setup::create_commit(
            vec![String::from("friends")],
            vec![setup.name_match_case],
            String::from("I am the continuation, for now..."),
        );

        let pres = rspace.get_once_non_durable_sequential(retrieve);
        let cres = rspace.put_once_non_durable_sequential(commit);

        assert!(pres.is_none());
        assert!(cres.is_some());
        assert!(rspace.is_memseq_empty());

        let _ = rspace.clear_store();
    }

    #[test]
    fn memseq_test_produce_match() {
        let setup = Setup::new();
        let rspace = setup.rspace;

        let commit = Setup::create_commit(
            vec![String::from("friends")],
            vec![setup.city_match_case],
            String::from("I am the continuation, for now..."),
        );
        let retrieve = Setup::create_retrieve(
            String::from("friends"),
            setup.alice.clone(),
            Setup::get_city_field(setup.alice),
        );
        let cres = rspace.put_once_non_durable_sequential(commit);
        let pres = rspace.get_once_non_durable_sequential(retrieve);

        assert!(cres.is_none());
        assert!(pres.is_some());
        assert!(rspace.is_memseq_empty());

        let _ = rspace.clear_store();
    }

    #[test]
    fn memseq_test_produce_no_match() {
        let setup = Setup::new();
        let rspace = setup.rspace;

        let commit = Setup::create_commit(
            vec![String::from("friends")],
            vec![setup.city_match_case],
            String::from("I am the continuation, for now..."),
        );
        let retrieve = Setup::create_retrieve(
            String::from("friends"),
            setup.carol.clone(),
            Setup::get_city_field(setup.carol),
        );

        let cres = rspace.put_once_non_durable_sequential(commit);
        let pres = rspace.get_once_non_durable_sequential(retrieve);

        assert!(cres.is_none());
        assert!(pres.is_none());
        assert!(!rspace.is_memseq_empty());

        let _ = rspace.clear_store();
    }

    #[test]
    fn memseq_test_consume_persist() {
        let setup = Setup::new();
        let rspace = setup.rspace;

        let commit = Setup::create_commit(
            vec![String::from("friends")],
            vec![setup.city_match_case],
            String::from("I am the continuation, for now..."),
        );
        let retrieve = Setup::create_retrieve(
            String::from("friends"),
            setup.alice.clone(),
            Setup::get_city_field(setup.alice),
        );

        let cres = rspace.put_always_non_durable_sequential(commit);

        assert!(cres.is_none());
        assert!(!rspace.is_memseq_empty());

        let pres = rspace.get_once_non_durable_sequential(retrieve);

        assert!(pres.is_some());
        assert!(!rspace.is_memseq_empty());

        let _ = rspace.clear_store();
    }

    #[test]
    fn memseq_test_produce_persist() {
        let setup = Setup::new();
        let rspace = setup.rspace;

        let retrieve = Setup::create_retrieve(
            String::from("friends"),
            setup.alice.clone(),
            Setup::get_city_field(setup.alice),
        );
        let commit = Setup::create_commit(
            vec![String::from("friends")],
            vec![setup.city_match_case],
            String::from("I am the continuation, for now..."),
        );

        let pres = rspace.get_always_non_durable_sequential(retrieve);

        assert!(pres.is_none());
        assert!(!rspace.is_memseq_empty());

        let cres = rspace.put_once_non_durable_sequential(commit);

        assert!(cres.is_some());
        assert_eq!(cres.unwrap().len(), 1);
        assert!(!rspace.is_memseq_empty());

        let _ = rspace.clear_store();
    }

    #[test]
    fn memseq_test_produce_persist_existing_matches() {
        let setup = Setup::new();
        let rspace = setup.rspace;

        let commit1 = Setup::create_commit(
            vec![String::from("friends")],
            vec![setup.city_match_case.clone()],
            String::from("I am the continuation, for now..."),
        );
        let retrieve1 = Setup::create_retrieve(
            String::from("friends"),
            setup.alice.clone(),
            Setup::get_city_field(setup.alice.clone()),
        );
        let retrieve2 = Setup::create_retrieve(
            String::from("friends"),
            setup.alice.clone(),
            Setup::get_city_field(setup.alice),
        );
        let commit2 = Setup::create_commit(
            vec![String::from("friends")],
            vec![setup.city_match_case],
            String::from("I am the continuation, for now..."),
        );

        let cres1 = rspace.put_once_non_durable_sequential(commit1);

        assert!(cres1.is_none());
        assert!(!rspace.is_memseq_empty());

        let pres1 = rspace.get_always_non_durable_sequential(retrieve1);

        assert!(pres1.is_some());
        assert!((rspace.is_memseq_empty()));

        let pres2 = rspace.get_always_non_durable_sequential(retrieve2);
        let _cres2 = rspace.put_once_non_durable_sequential(commit2);

        assert!(pres2.is_none());
        assert!(!rspace.is_memseq_empty());

        let _ = rspace.clear_store();
    }

    //memconc
    #[test]
    fn memconc_test_consume_persist_existing_matches() {
        let setup = Setup::new();
        let rspace = setup.rspace;

        let retrieve1 = Setup::create_retrieve(
            String::from("friends"),
            setup.alice.clone(),
            Setup::get_city_field(setup.alice.clone()),
        );
        let retrieve2 = Setup::create_retrieve(
            String::from("friends"),
            setup.bob.clone(),
            Setup::get_city_field(setup.bob),
        );
        let commit1 = Setup::create_commit(
            vec![String::from("friends")],
            vec![setup.city_match_case.clone()],
            String::from("I am the continuation, for now..."),
        );
        let commit2 = Setup::create_commit(
            vec![String::from("friends")],
            vec![setup.city_match_case.clone()],
            String::from("I am the continuation, for now..."),
        );
        let commit3 = Setup::create_commit(
            vec![String::from("friends")],
            vec![setup.city_match_case],
            String::from("I am the continuation, for now..."),
        );
        let retrieve3 = Setup::create_retrieve(
            String::from("friends"),
            setup.alice.clone(),
            Setup::get_city_field(setup.alice),
        );
        let _pres1 = rspace.get_once_non_durable_concurrent(retrieve1);
        let _pres2 = rspace.get_once_non_durable_concurrent(retrieve2);
        let cres1 = rspace.put_always_non_durable_concurrent(commit1);

        assert_eq!(cres1.unwrap().len(), 1);
        assert!(!rspace.is_memconc_empty());

        let cres2 = rspace.put_always_non_durable_concurrent(commit2);

        assert_eq!(cres2.unwrap().len(), 1);
        assert!(rspace.is_memconc_empty());

        let cres3 = rspace.put_always_non_durable_concurrent(commit3);

        assert!(cres3.is_none());
        assert!(!rspace.is_memconc_empty());

        let pres3 = rspace.get_once_non_durable_concurrent(retrieve3);

        assert!(pres3.is_some());
        assert!(!rspace.is_memconc_empty());

        let _ = rspace.clear_store();
    }

    #[test]
    fn memconc_test_multiple_channels_consume_match() {
        let setup = Setup::new();
        let rspace = setup.rspace;

        let retrieve1 = Setup::create_retrieve(
            String::from("colleagues"),
            setup.dan.clone(),
            Setup::get_state_field(setup.dan),
        );
        let retrieve2 = Setup::create_retrieve(
            String::from("friends"),
            setup.erin.clone(),
            Setup::get_state_field(setup.erin),
        );
        let commit = Setup::create_commit(
            vec![String::from("friends"), String::from("colleagues")],
            vec![setup.state_match_case.clone(), setup.state_match_case],
            String::from("I am the continuation, for now..."),
        );

        let pres1 = rspace.get_once_non_durable_concurrent(retrieve1);
        let pres2 = rspace.get_once_non_durable_concurrent(retrieve2);

        let cres = rspace.put_once_non_durable_concurrent(commit);

        assert!(pres1.is_none());
        assert!(pres2.is_none());
        assert!(cres.is_some());
        assert_eq!(cres.unwrap().len(), 2);
        assert!(rspace.is_memconc_empty());

        let _ = rspace.clear_store();
    }

    #[test]
    fn memconc_test_consume_match() {
        let setup = Setup::new();
        let rspace = setup.rspace;

        let retrieve = Setup::create_retrieve(
            String::from("friends"),
            setup.bob.clone(),
            Setup::get_last_name_field(setup.bob),
        );
        let commit = Setup::create_commit(
            vec![String::from("friends")],
            vec![setup.name_match_case],
            String::from("I am the continuation, for now..."),
        );
        let pres = rspace.get_once_non_durable_concurrent(retrieve);
        let cres = rspace.put_once_non_durable_concurrent(commit);

        assert!(pres.is_none());
        assert!(cres.is_some());
        assert!(rspace.is_memconc_empty());

        let _ = rspace.clear_store();
    }

    #[test]
    fn memconc_test_produce_match() {
        let setup = Setup::new();
        let rspace = setup.rspace;

        let commit = Setup::create_commit(
            vec![String::from("friends")],
            vec![setup.city_match_case],
            String::from("I am the continuation, for now..."),
        );
        let retrieve = Setup::create_retrieve(
            String::from("friends"),
            setup.alice.clone(),
            Setup::get_city_field(setup.alice),
        );

        let cres = rspace.put_once_non_durable_concurrent(commit);
        let pres = rspace.get_once_non_durable_concurrent(retrieve);

        assert!(cres.is_none());
        assert!(pres.is_some());
        assert!(rspace.is_memconc_empty());

        let _ = rspace.clear_store();
    }

    #[test]
    fn memconc_test_produce_no_match() {
        let setup = Setup::new();
        let rspace = setup.rspace;

        let commit = Setup::create_commit(
            vec![String::from("friends")],
            vec![setup.city_match_case],
            String::from("I am the continuation, for now..."),
        );
        let retrieve = Setup::create_retrieve(
            String::from("friends"),
            setup.carol.clone(),
            Setup::get_city_field(setup.carol),
        );

        let cres = rspace.put_once_non_durable_concurrent(commit);
        let pres = rspace.get_once_non_durable_concurrent(retrieve);

        assert!(cres.is_none());
        assert!(pres.is_none());
        assert!(!rspace.is_memconc_empty());

        let _ = rspace.clear_store();
    }

    #[test]
    fn memconc_test_consume_persist() {
        let setup = Setup::new();
        let rspace = setup.rspace;

        let commit = Setup::create_commit(
            vec![String::from("friends")],
            vec![setup.city_match_case],
            String::from("I am the continuation, for now..."),
        );
        let retrieve = Setup::create_retrieve(
            String::from("friends"),
            setup.alice.clone(),
            Setup::get_city_field(setup.alice),
        );

        let cres = rspace.put_always_non_durable_concurrent(commit);

        assert!(cres.is_none());
        assert!(!rspace.is_memconc_empty());

        let pres = rspace.get_once_non_durable_concurrent(retrieve);

        assert!(pres.is_some());
        assert!(!rspace.is_memconc_empty());

        let _ = rspace.clear_store();
    }

    #[test]
    fn memconc_test_produce_persist() {
        let setup = Setup::new();
        let rspace = setup.rspace;

        let retrieve = Setup::create_retrieve(
            String::from("friends"),
            setup.alice.clone(),
            Setup::get_city_field(setup.alice),
        );
        let commit = Setup::create_commit(
            vec![String::from("friends")],
            vec![setup.city_match_case],
            String::from("I am the continuation, for now..."),
        );

        let pres = rspace.get_always_non_durable_concurrent(retrieve);

        assert!(pres.is_none());
        assert!(!rspace.is_memconc_empty());

        let cres = rspace.put_once_non_durable_concurrent(commit);

        assert!(cres.is_some());
        assert_eq!(cres.unwrap().len(), 1);
        assert!(!rspace.is_memconc_empty());

        let _ = rspace.clear_store();
    }

    #[test]
    fn memconc_test_produce_persist_existing_matches() {
        let setup = Setup::new();
        let rspace = setup.rspace;

        let commit1 = Setup::create_commit(
            vec![String::from("friends")],
            vec![setup.city_match_case.clone()],
            String::from("I am the continuation, for now..."),
        );
        let retrieve1 = Setup::create_retrieve(
            String::from("friends"),
            setup.alice.clone(),
            Setup::get_city_field(setup.alice.clone()),
        );
        let retrieve2 = Setup::create_retrieve(
            String::from("friends"),
            setup.alice.clone(),
            Setup::get_city_field(setup.alice),
        );
        let commit2 = Setup::create_commit(
            vec![String::from("friends")],
            vec![setup.city_match_case],
            String::from("I am the continuation, for now..."),
        );

        let cres1 = rspace.put_once_non_durable_concurrent(commit1);

        assert!(cres1.is_none());
        assert!(!rspace.is_memconc_empty());

        let pres1 = rspace.get_always_non_durable_concurrent(retrieve1);

        assert!(pres1.is_some());
        assert!((rspace.is_memconc_empty()));

        let pres2 = rspace.get_always_non_durable_concurrent(retrieve2);
        let _cres2 = rspace.put_once_non_durable_concurrent(commit2);

        assert!(pres2.is_none());
        assert!(!rspace.is_memconc_empty());

        let _ = rspace.clear_store();
    }

    //diskconc
    #[test]
    fn diskconc_test_consume_persist_existing_matches() {
        let setup = Setup::new();
        let rspace = setup.rspace;

        let retrieve1 = Setup::create_retrieve(
            String::from("friends"),
            setup.alice.clone(),
            Setup::get_city_field(setup.alice.clone()),
        );
        let retrieve2 = Setup::create_retrieve(
            String::from("friends"),
            setup.bob.clone(),
            Setup::get_city_field(setup.bob),
        );
        let commit1 = Setup::create_commit(
            vec![String::from("friends")],
            vec![setup.city_match_case.clone()],
            String::from("I am the continuation, for now..."),
        );
        let commit2 = Setup::create_commit(
            vec![String::from("friends")],
            vec![setup.city_match_case.clone()],
            String::from("I am the continuation, for now..."),
        );
        let commit3 = Setup::create_commit(
            vec![String::from("friends")],
            vec![setup.city_match_case],
            String::from("I am the continuation, for now..."),
        );
        let retrieve3 = Setup::create_retrieve(
            String::from("friends"),
            setup.alice.clone(),
            Setup::get_city_field(setup.alice),
        );

        let _pres1 = rspace.get_once_durable_concurrent(retrieve1);
        let _pres2 = rspace.get_once_durable_concurrent(retrieve2);
        rspace.print_data("friends");
        let cres1 = rspace.put_always_durable_concurrent(commit1);
        rspace.print_data("friends");
        assert_eq!(cres1.unwrap().len(), 1);
        assert!(!rspace.is_diskconc_empty());

        let cres2 = rspace.put_always_durable_concurrent(commit2);

        assert_eq!(cres2.unwrap().len(), 1);
        assert!(rspace.is_diskconc_empty());

        let cres3 = rspace.put_always_durable_concurrent(commit3);

        assert!(cres3.is_none());
        assert!(!rspace.is_diskconc_empty());

        let pres3 = rspace.get_once_durable_concurrent(retrieve3);

        assert!(pres3.is_some());
        assert!(!rspace.is_diskconc_empty());

        let _ = rspace.clear_store();
    }

    #[test]
    fn diskconc_test_multiple_channels_consume_match() {
        let setup = Setup::new();
        let rspace = setup.rspace;

        let retrieve1 = Setup::create_retrieve(
            String::from("colleagues"),
            setup.dan.clone(),
            Setup::get_state_field(setup.dan),
        );
        let retrieve2 = Setup::create_retrieve(
            String::from("friends"),
            setup.erin.clone(),
            Setup::get_state_field(setup.erin),
        );
        let commit = Setup::create_commit(
            vec![String::from("friends"), String::from("colleagues")],
            vec![setup.state_match_case.clone(), setup.state_match_case],
            String::from("I am the continuation, for now..."),
        );

        let pres1 = rspace.get_once_durable_concurrent(retrieve1);
        let pres2 = rspace.get_once_durable_concurrent(retrieve2);

        let cres = rspace.put_once_durable_concurrent(commit);

        assert!(pres1.is_none());
        assert!(pres2.is_none());
        assert!(cres.is_some());
        assert_eq!(cres.unwrap().len(), 2);
        assert!(rspace.is_diskconc_empty());

        let _ = rspace.clear_store();
    }

    #[test]
    fn diskconc_test_consume_match() {
        let setup = Setup::new();
        let rspace = setup.rspace;

        let retrieve = Setup::create_retrieve(
            String::from("friends"),
            setup.bob.clone(),
            Setup::get_last_name_field(setup.bob),
        );
        let commit = Setup::create_commit(
            vec![String::from("friends")],
            vec![setup.name_match_case],
            String::from("I am the continuation, for now..."),
        );

        let pres = rspace.get_once_durable_concurrent(retrieve);
        let cres = rspace.put_once_durable_concurrent(commit);

        assert!(pres.is_none());
        assert!(cres.is_some());
        assert!(rspace.is_diskconc_empty());

        let _ = rspace.clear_store();
    }

    #[test]
    fn diskconc_test_produce_match() {
        let setup = Setup::new();
        let rspace = setup.rspace;

        let commit = Setup::create_commit(
            vec![String::from("friends")],
            vec![setup.city_match_case],
            String::from("I am the continuation, for now..."),
        );
        let retrieve = Setup::create_retrieve(
            String::from("friends"),
            setup.alice.clone(),
            Setup::get_city_field(setup.alice),
        );

        let cres = rspace.put_once_durable_concurrent(commit);
        let pres = rspace.get_once_durable_concurrent(retrieve);

        assert!(cres.is_none());
        assert!(pres.is_some());
        assert!(rspace.is_diskconc_empty());

        let _ = rspace.clear_store();
    }

    #[test]
    fn diskconc_test_produce_no_match() {
        let setup = Setup::new();
        let rspace = setup.rspace;

        let commit = Setup::create_commit(
            vec![String::from("friends")],
            vec![setup.city_match_case],
            String::from("I am the continuation, for now..."),
        );
        let retrieve = Setup::create_retrieve(
            String::from("friends"),
            setup.carol.clone(),
            Setup::get_city_field(setup.carol),
        );
        let cres = rspace.put_once_durable_concurrent(commit);
        let pres = rspace.get_once_durable_concurrent(retrieve);

        assert!(cres.is_none());
        assert!(pres.is_none());
        assert!(!rspace.is_diskconc_empty());

        let _ = rspace.clear_store();
    }

    #[test]
    fn diskconc_test_consume_persist() {
        let setup = Setup::new();
        let rspace = setup.rspace;

        let commit = Setup::create_commit(
            vec![String::from("friends")],
            vec![setup.city_match_case],
            String::from("I am the continuation, for now..."),
        );
        let retrieve = Setup::create_retrieve(
            String::from("friends"),
            setup.alice.clone(),
            Setup::get_city_field(setup.alice),
        );

        let cres = rspace.put_always_durable_concurrent(commit);

        assert!(cres.is_none());
        assert!(!rspace.is_diskconc_empty());

        let pres = rspace.get_once_durable_concurrent(retrieve);

        assert!(pres.is_some());
        assert!(!rspace.is_diskconc_empty());

        let _ = rspace.clear_store();
    }

    #[test]
    fn diskconc_test_produce_persist() {
        let setup = Setup::new();
        let rspace = setup.rspace;

        let retrieve = Setup::create_retrieve(
            String::from("friends"),
            setup.alice.clone(),
            Setup::get_city_field(setup.alice),
        );
        let commit = Setup::create_commit(
            vec![String::from("friends")],
            vec![setup.city_match_case],
            String::from("I am the continuation, for now..."),
        );

        let pres = rspace.get_always_durable_concurrent(retrieve);

        assert!(pres.is_none());
        assert!(!rspace.is_diskconc_empty());

        let cres = rspace.put_once_durable_concurrent(commit);

        assert!(cres.is_some());
        assert_eq!(cres.unwrap().len(), 1);
        assert!(!rspace.is_diskconc_empty());

        let _ = rspace.clear_store();
    }

    #[test]
    fn diskconc_test_produce_persist_existing_matches() {
        let setup = Setup::new();
        let rspace = setup.rspace;

        let commit1 = Setup::create_commit(
            vec![String::from("friends")],
            vec![setup.city_match_case.clone()],
            String::from("I am the continuation, for now..."),
        );
        let retrieve1 = Setup::create_retrieve(
            String::from("friends"),
            setup.alice.clone(),
            Setup::get_city_field(setup.alice.clone()),
        );
        let retrieve2 = Setup::create_retrieve(
            String::from("friends"),
            setup.alice.clone(),
            Setup::get_city_field(setup.alice),
        );
        let commit2 = Setup::create_commit(
            vec![String::from("friends")],
            vec![setup.city_match_case],
            String::from("I am the continuation, for now..."),
        );

        let cres1 = rspace.put_once_durable_concurrent(commit1);

        assert!(cres1.is_none());
        assert!(!rspace.is_diskconc_empty());

        let pres1 = rspace.get_always_durable_concurrent(retrieve1);

        assert!(pres1.is_some());
        assert!((rspace.is_diskconc_empty()));

        let pres2 = rspace.get_always_durable_concurrent(retrieve2);
        let _cres2 = rspace.put_once_durable_concurrent(commit2);

        assert!(pres2.is_none());
        assert!(!rspace.is_diskconc_empty());

        let _ = rspace.clear_store();
    }

    //diskseq
    #[test]
    fn diskseq_test_consume_persist_existing_matches() {
        let setup = Setup::new();
        let rspace = setup.rspace;

        let retrieve1 = Setup::create_retrieve(
            String::from("friends"),
            setup.alice.clone(),
            Setup::get_city_field(setup.alice.clone()),
        );
        let retrieve2 = Setup::create_retrieve(
            String::from("friends"),
            setup.bob.clone(),
            Setup::get_city_field(setup.bob),
        );
        let commit1 = Setup::create_commit(
            vec![String::from("friends")],
            vec![setup.city_match_case.clone()],
            String::from("I am the continuation, for now..."),
        );
        let commit2 = Setup::create_commit(
            vec![String::from("friends")],
            vec![setup.city_match_case.clone()],
            String::from("I am the continuation, for now..."),
        );
        let commit3 = Setup::create_commit(
            vec![String::from("friends")],
            vec![setup.city_match_case],
            String::from("I am the continuation, for now..."),
        );
        let retrieve3 = Setup::create_retrieve(
            String::from("friends"),
            setup.alice.clone(),
            Setup::get_city_field(setup.alice),
        );

        let _pres1 = rspace.get_once_durable_sequential(retrieve1);
        let _pres2 = rspace.get_once_durable_sequential(retrieve2);
        let cres1 = rspace.put_always_durable_sequential(commit1);

        assert_eq!(cres1.unwrap().len(), 1);
        assert!(!rspace.is_diskseq_empty());

        let cres2 = rspace.put_always_durable_sequential(commit2);

        assert_eq!(cres2.unwrap().len(), 1);
        assert!(rspace.is_diskseq_empty());

        let cres3 = rspace.put_always_durable_sequential(commit3);

        assert!(cres3.is_none());
        assert!(!rspace.is_diskseq_empty());

        let pres3 = rspace.get_once_durable_sequential(retrieve3);

        assert!(pres3.is_some());
        assert!(!rspace.is_diskseq_empty());

        let _ = rspace.clear_store();
    }

    #[test]
    fn diskseq_test_multiple_channels_consume_match() {
        let setup = Setup::new();
        let rspace = setup.rspace;

        let retrieve1 = Setup::create_retrieve(
            String::from("colleagues"),
            setup.dan.clone(),
            Setup::get_state_field(setup.dan),
        );
        let retrieve2 = Setup::create_retrieve(
            String::from("friends"),
            setup.erin.clone(),
            Setup::get_state_field(setup.erin),
        );
        let commit = Setup::create_commit(
            vec![String::from("friends"), String::from("colleagues")],
            vec![setup.state_match_case.clone(), setup.state_match_case],
            String::from("I am the continuation, for now..."),
        );
        let pres1 = rspace.get_once_durable_sequential(retrieve1);
        let pres2 = rspace.get_once_durable_sequential(retrieve2);

        let cres = rspace.put_once_durable_sequential(commit);

        assert!(pres1.is_none());
        assert!(pres2.is_none());
        assert!(cres.is_some());
        assert_eq!(cres.unwrap().len(), 2);
        assert!(rspace.is_diskseq_empty());

        let _ = rspace.clear_store();
    }

    #[test]
    fn diskseq_test_consume_match() {
        let setup = Setup::new();
        let rspace = setup.rspace;

        let retrieve = Setup::create_retrieve(
            String::from("friends"),
            setup.bob.clone(),
            Setup::get_last_name_field(setup.bob),
        );
        let commit = Setup::create_commit(
            vec![String::from("friends")],
            vec![setup.name_match_case],
            String::from("I am the continuation, for now..."),
        );
        let pres = rspace.get_once_durable_sequential(retrieve);
        let cres = rspace.put_once_durable_sequential(commit);

        assert!(pres.is_none());
        assert!(cres.is_some());
        assert!(rspace.is_diskseq_empty());

        let _ = rspace.clear_store();
    }

    #[test]
    fn diskseq_test_produce_match() {
        let setup = Setup::new();
        let rspace = setup.rspace;

        let commit = Setup::create_commit(
            vec![String::from("friends")],
            vec![setup.city_match_case],
            String::from("I am the continuation, for now..."),
        );
        let retrieve = Setup::create_retrieve(
            String::from("friends"),
            setup.alice.clone(),
            Setup::get_city_field(setup.alice),
        );
        let cres = rspace.put_once_durable_sequential(commit);
        let pres = rspace.get_once_durable_sequential(retrieve);

        assert!(cres.is_none());
        assert!(pres.is_some());
        assert!(rspace.is_diskseq_empty());

        let _ = rspace.clear_store();
    }

    #[test]
    fn diskseq_test_produce_no_match() {
        let setup = Setup::new();
        let rspace = setup.rspace;

        let commit = Setup::create_commit(
            vec![String::from("friends")],
            vec![setup.city_match_case],
            String::from("I am the continuation, for now..."),
        );
        let retrieve = Setup::create_retrieve(
            String::from("friends"),
            setup.carol.clone(),
            Setup::get_city_field(setup.carol),
        );

        let cres = rspace.put_once_durable_sequential(commit);
        let pres = rspace.get_once_durable_sequential(retrieve);

        assert!(cres.is_none());
        assert!(pres.is_none());
        assert!(!rspace.is_diskseq_empty());

        let _ = rspace.clear_store();
    }

    #[test]
    fn diskseq_test_consume_persist() {
        let setup = Setup::new();
        let rspace = setup.rspace;

        let commit = Setup::create_commit(
            vec![String::from("friends")],
            vec![setup.city_match_case],
            String::from("I am the continuation, for now..."),
        );
        let retrieve = Setup::create_retrieve(
            String::from("friends"),
            setup.alice.clone(),
            Setup::get_city_field(setup.alice),
        );
        let cres = rspace.put_always_durable_sequential(commit);

        assert!(cres.is_none());
        assert!(!rspace.is_diskseq_empty());

        let pres = rspace.get_once_durable_sequential(retrieve);

        assert!(pres.is_some());
        assert!(!rspace.is_diskseq_empty());

        let _ = rspace.clear_store();
    }

    #[test]
    fn diskseq_test_produce_persist() {
        let setup = Setup::new();
        let rspace = setup.rspace;

        let retrieve = Setup::create_retrieve(
            String::from("friends"),
            setup.alice.clone(),
            Setup::get_city_field(setup.alice),
        );
        let commit = Setup::create_commit(
            vec![String::from("friends")],
            vec![setup.city_match_case],
            String::from("I am the continuation, for now..."),
        );

        let pres = rspace.get_always_durable_sequential(retrieve);

        assert!(pres.is_none());
        assert!(!rspace.is_diskseq_empty());

        let cres = rspace.put_once_durable_sequential(commit);

        assert!(cres.is_some());
        assert_eq!(cres.unwrap().len(), 1);
        assert!(!rspace.is_diskseq_empty());

        let _ = rspace.clear_store();
    }

    #[test]
    fn diskseq_test_produce_persist_existing_matches() {
        let setup = Setup::new();
        let rspace = setup.rspace;

        let commit1 = Setup::create_commit(
            vec![String::from("friends")],
            vec![setup.city_match_case.clone()],
            String::from("I am the continuation, for now..."),
        );
        let retrieve1 = Setup::create_retrieve(
            String::from("friends"),
            setup.alice.clone(),
            Setup::get_city_field(setup.alice.clone()),
        );
        let retrieve2 = Setup::create_retrieve(
            String::from("friends"),
            setup.alice.clone(),
            Setup::get_city_field(setup.alice),
        );
        let commit2 = Setup::create_commit(
            vec![String::from("friends")],
            vec![setup.city_match_case],
            String::from("I am the continuation, for now..."),
        );
        let cres1 = rspace.put_once_durable_sequential(commit1);

        assert!(cres1.is_none());
        assert!(!rspace.is_diskseq_empty());

        let pres1 = rspace.get_always_durable_sequential(retrieve1);

        assert!(pres1.is_some());
        assert!((rspace.is_diskseq_empty()));

        let pres2 = rspace.get_always_durable_sequential(retrieve2);
        let _cres2 = rspace.put_once_durable_sequential(commit2);

        assert!(pres2.is_none());
        assert!(!rspace.is_diskseq_empty());

        let _ = rspace.clear_store();
    }
}

// #[test]
// fn rspace_test_duplicate_keys() {
//     let mut setup = Setup::new();
//     let rspace = setup.rspace;
//     setup.entries = vec![setup.emptyEntry.clone(),setup.emptyEntry.clone(),setup.emptyEntry.clone()];
//     let binding = "chan1".to_string();
//     let channels: Vec<&str> = vec! [&binding,&binding,&binding];

//     let cres = rspace.put_once_non_durable_sequential(channels, vec! [city_match,state_match,name_match], Printer);
//     rspace.print_data(&binding);
//     let pres = rspace.get_once_non_durable_sequential(&binding, setup.emptyEntry);
//     rspace.print_data(&binding);

//     println!("\npres {:?}", pres);

//     assert!(cres.is_none());
//     assert!(pres.is_some());

//     // let _ = rspace.clear();
// }

// #[test]
// fn rspace_test_memory_concurrent() {
//     let mut setup = Setup::new();
//     let rspace = Arc::new(setup.rspace);
//     setup.entries = vec![setup.emptyEntry.clone(),setup.emptyEntry.clone(),setup.emptyEntry.clone()];

//     println!("\n -= -= -= -= starting =- =- =- =- \n");
//     // Create 10 threads that write to the hashmap
//     let mut handles = Vec::new();
//     for i in 0..10 {
//         let binding = "chan1".to_string();
//         let mut e: Entry = setup.emptyEntry.clone();
//         // fn new_match(entry: Entry, i:u8) -> bool {
//         //     entry.pos == i
//         // }
//         e.pos = i;
//         e.posStr = i.to_string();
//         let rspace_clone = Arc::clone(&rspace);
//         let handle = thread::spawn(move || {
//             //putting this one on allows the output to go through but its nonsense
//             //let pres = rspace_clone.put_once_non_durable_concurrent(vec! [&binding], vec! [pos_match],Printer);

//             //this one makes sense to put in the data by calling get knowing it puts the entry in the db
//             //printouts during test show them going as threaded and out of order
//             let pres = rspace_clone.get_once_non_durable_concurrent(&binding, e);
//             //println!("pres value: {:?}", pres);

//             //println!("value: {}", pres.unwrap().data.posStr);
//             //let v= pres.unwrap().data;
//             // println!("i: {}", i);
//             // println!("v pos: {}", v.pos);
//             // println!("v posStr: {}", v.posStr);
//         });
//         handles.push(handle);
//     }

//     // Wait for all the threads to complete
//     println!("\ncompleting puts\n");
//     for handle in handles {
//         handle.join().unwrap();
//     }
//     println!("\nended puts\n");

//     println!("\nbegin print state of db\n");
//     rspace.print_data(&"chan1".to_string());
//     println!("\nend print state of db\n");

//     // Check that all the values were written correctly
//     // for i in 0..10 {
//     //     let binding = "chan1".to_string();
//     //     let mut e: Entry = setup.emptyEntry.clone();
//     //     e.pos = i;
//     //     e.posStr = i.to_string();
//     //     let pres = rspace.get_once_non_durable_sequential(&binding, e);
//     //     // let v= pres.data;
//     //     // println!("i: {}", i);
//     //     // println!("v pos: {}", v.pos);
//     //     // println!("v posStr: {}", v.posStr);
//     //     println!("pres {:?}", pres);
//     //     //assert_eq!(v.pos, i);
//     // }

//     // // Create 10 threads that read from the hashmap
//     let mut handles = Vec::new();
//     for i in 0..10 {
//         let binding = "chan1".to_string();
//         let mut e: Entry = setup.emptyEntry.clone();
//         e.pos = i;
//         e.posStr = i.to_string();
//         let rspace_clone = Arc::clone(&rspace);
//         let handle = thread::spawn(move || {
//             //putting this one on allows the output to go through but its nonsense
//             let pres = rspace_clone.put_once_non_durable_concurrent(vec! [&binding], vec! [pos_match],Printer);

//             //this one makes sense to put in the data by calling get knowing it puts the entry in the db
//             //printouts during test show them going as threaded and out of order
//             //let pres = rspace_clone.get_once_non_durable_concurrent(&binding, e);

//             //println!("pres value: {:?}", pres);
//             // let v= pres.unwrap().data;
//             // println!("i: {}", i);
//             // println!("v pos: {}", v.pos);
//             // println!("v posStr: {}", v.posStr);
//         });
//         handles.push(handle);
//     }

//     // Wait for all the threads to complete
//     println!("\ncompleting gets\n");
//     for handle in handles {
//         handle.join().unwrap();
//     }
//     println!("\nended gets\n");

//     //     let cres = rspace.put_once_non_durable_concurrent(channels, vec! [city_match], Printer);
//     //     rspace.print_data(&binding);
//     //     let pres = rspace.get_once_non_durable_sequential(&binding, setup.emptyEntry);

//     println!("\nbegin print state of db\n");
//     rspace.print_data(&"chan1".to_string());
//     println!("\nend print state of db\n");

// }
// // //cargo test --test rspace_test -- --test-threads=1

// #[test]
// fn rspace_test_default_memory_sequential() {
//     memseq_test_consume_persist_existing_matches();
//     // memseq_test_multiple_channels_consume_match();
//     // memseq_test_consume_match();
//     // memseq_test_produce_match();
//     // memseq_test_produce_no_match();
//     // memseq_test_consume_persist();
//     // memseq_test_produce_persist();
//     // memseq_test_produce_persist_existing_matches();

// }

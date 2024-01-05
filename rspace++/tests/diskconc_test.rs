#[cfg(test)]
mod tests {
    use rspace_plus_plus::setup::Setup;

    #[test]
    fn diskconc_test_produce_match() {
        let setup = Setup::new();
        let diskconc = setup.diskconc;

        let commit = Setup::create_commit(
            vec![String::from("friends")],
            vec![setup.city_match_case],
            String::from("I am the continuation, for now..."),
        );
        let cres = diskconc.consume(commit, false);

        let retrieve = Setup::create_retrieve(
            String::from("friends"),
            setup.alice.clone(),
            Setup::get_city_field(setup.alice),
        );
        let pres = diskconc.produce(retrieve, false);

        assert!(cres.is_none());
        assert!(pres.is_some());
        assert!(diskconc.is_empty());

        let _ = diskconc.clear();
    }

    #[test]
    fn diskconc_test_produce_no_match() {
        let setup = Setup::new();
        let diskconc = setup.diskconc;

        let commit = Setup::create_commit(
            vec![String::from("friends")],
            vec![setup.city_match_case],
            String::from("I am the continuation, for now..."),
        );
        let cres = diskconc.consume(commit, false);

        let retrieve = Setup::create_retrieve(
            String::from("friends"),
            setup.carol.clone(),
            Setup::get_city_field(setup.carol),
        );
        let pres = diskconc.produce(retrieve, false);

        assert!(cres.is_none());
        assert!(pres.is_none());
        assert!(!diskconc.is_empty());

        let _ = diskconc.clear();
    }

    #[test]
    fn diskconc_test_consume_match() {
        let setup = Setup::new();
        let diskconc = setup.diskconc;

        let retrieve = Setup::create_retrieve(
            String::from("friends"),
            setup.bob.clone(),
            Setup::get_last_name_field(setup.bob),
        );
        let pres = diskconc.produce(retrieve, false);

        let commit = Setup::create_commit(
            vec![String::from("friends")],
            vec![setup.name_match_case],
            String::from("I am the continuation, for now..."),
        );
        let cres = diskconc.consume(commit, false);

        assert!(pres.is_none());
        assert!(cres.is_some());
        assert!(diskconc.is_empty());

        let _ = diskconc.clear();
    }

    #[test]
    fn diskconc_test_multiple_channels_consume_match() {
        let setup = Setup::new();
        let diskconc = setup.diskconc;

        let retrieve1 = Setup::create_retrieve(
            String::from("colleagues"),
            setup.dan.clone(),
            Setup::get_state_field(setup.dan),
        );
        let pres1 = diskconc.produce(retrieve1, false);

        let retrieve2 = Setup::create_retrieve(
            String::from("friends"),
            setup.erin.clone(),
            Setup::get_state_field(setup.erin),
        );
        let pres2 = diskconc.produce(retrieve2, false);

        let commit = Setup::create_commit(
            vec![String::from("friends"), String::from("colleagues")],
            vec![setup.state_match_case.clone(), setup.state_match_case],
            String::from("I am the continuation, for now..."),
        );
        let cres = diskconc.consume(commit, false);

        assert!(pres1.is_none());
        assert!(pres2.is_none());
        assert!(cres.is_some());
        assert_eq!(cres.unwrap().len(), 2);
        assert!(diskconc.is_empty());

        let _ = diskconc.clear();
    }

    #[test]
    fn diskconc_test_consume_persist() {
        let setup = Setup::new();
        let diskconc = setup.diskconc;

        let commit = Setup::create_commit(
            vec![String::from("friends")],
            vec![setup.city_match_case],
            String::from("I am the continuation, for now..."),
        );
        let cres = diskconc.consume(commit, true);

        assert!(cres.is_none());
        assert!(!diskconc.is_empty());

        let retrieve = Setup::create_retrieve(
            String::from("friends"),
            setup.alice.clone(),
            Setup::get_city_field(setup.alice),
        );
        let pres = diskconc.produce(retrieve, false);

        assert!(pres.is_some());
        assert!(!diskconc.is_empty());

        let _ = diskconc.clear();
    }

    #[test]
    fn diskconc_test_consume_persist_existing_matches() {
        let setup = Setup::new();
        let diskconc = setup.diskconc;

        let retrieve1 = Setup::create_retrieve(
            String::from("friends"),
            setup.alice.clone(),
            Setup::get_city_field(setup.alice.clone()),
        );
        let _pres1 = diskconc.produce(retrieve1, false);

        let retrieve2 = Setup::create_retrieve(
            String::from("friends"),
            setup.bob.clone(),
            Setup::get_city_field(setup.bob),
        );
        let _pres2 = diskconc.produce(retrieve2, false);

        let commit1 = Setup::create_commit(
            vec![String::from("friends")],
            vec![setup.city_match_case.clone()],
            String::from("I am the continuation, for now..."),
        );
        let cres1 = diskconc.consume(commit1, true);

        assert_eq!(cres1.unwrap().len(), 1);
        assert!(!diskconc.is_empty());

        let commit2 = Setup::create_commit(
            vec![String::from("friends")],
            vec![setup.city_match_case.clone()],
            String::from("I am the continuation, for now..."),
        );
        let cres2 = diskconc.consume(commit2, true);

        assert_eq!(cres2.unwrap().len(), 1);
        assert!(diskconc.is_empty());

        let commit3 = Setup::create_commit(
            vec![String::from("friends")],
            vec![setup.city_match_case],
            String::from("I am the continuation, for now..."),
        );
        let cres3 = diskconc.consume(commit3, true);

        assert!(cres3.is_none());
        assert!(!diskconc.is_empty());

        let retrieve3 = Setup::create_retrieve(
            String::from("friends"),
            setup.alice.clone(),
            Setup::get_city_field(setup.alice),
        );
        let pres3 = diskconc.produce(retrieve3, false);

        assert!(pres3.is_some());
        assert!(!diskconc.is_empty());

        let _ = diskconc.clear();
    }

    #[test]
    fn diskconc_test_produce_persist() {
        let setup = Setup::new();
        let diskconc = setup.diskconc;

        let retrieve = Setup::create_retrieve(
            String::from("friends"),
            setup.alice.clone(),
            Setup::get_city_field(setup.alice),
        );
        let pres = diskconc.produce(retrieve, true);

        assert!(pres.is_none());
        assert!(!diskconc.is_empty());

        let commit = Setup::create_commit(
            vec![String::from("friends")],
            vec![setup.city_match_case],
            String::from("I am the continuation, for now..."),
        );
        let cres = diskconc.consume(commit, false);

        assert!(cres.is_some());
        assert_eq!(cres.unwrap().len(), 1);
        assert!(!diskconc.is_empty());

        let _ = diskconc.clear();
    }

    #[test]
    fn diskconc_test_produce_persist_existing_matches() {
        let setup = Setup::new();
        let diskconc = setup.diskconc;

        let commit1 = Setup::create_commit(
            vec![String::from("friends")],
            vec![setup.city_match_case.clone()],
            String::from("I am the continuation, for now..."),
        );
        let cres1 = diskconc.consume(commit1, false);

        assert!(cres1.is_none());
        assert!(!diskconc.is_empty());

        let retrieve1 = Setup::create_retrieve(
            String::from("friends"),
            setup.alice.clone(),
            Setup::get_city_field(setup.alice.clone()),
        );
        let pres1 = diskconc.produce(retrieve1, true);

        assert!(pres1.is_some());
        assert!((diskconc.is_empty()));

        let retrieve2 = Setup::create_retrieve(
            String::from("friends"),
            setup.alice.clone(),
            Setup::get_city_field(setup.alice),
        );
        let pres2 = diskconc.produce(retrieve2, true);

        let commit2 = Setup::create_commit(
            vec![String::from("friends")],
            vec![setup.city_match_case],
            String::from("I am the continuation, for now..."),
        );
        let _cres2 = diskconc.consume(commit2, false);

        assert!(pres2.is_none());
        assert!(!diskconc.is_empty());

        let _ = diskconc.clear();
    }
}

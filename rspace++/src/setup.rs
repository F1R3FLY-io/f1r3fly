#![allow(dead_code)]

use crate::diskconc::DiskConcDB;
use crate::diskseq::DiskSeqDB;
use crate::memconc::MemConcDB;
use crate::memseq::MemSeqDB;
use crate::rspace::RSpace;
use crate::rtypes::rtypes::{Address, Commit, Entry, Name, Retrieve};

pub struct Setup {
    pub rspace: RSpace<Retrieve, Commit>,
    pub memconc: MemConcDB<Retrieve, Commit>,
    pub memseq: MemSeqDB<Retrieve, Commit>,
    pub diskconc: DiskConcDB<Retrieve, Commit>,
    pub diskseq: DiskSeqDB<Retrieve, Commit>,
    pub city_match_case: String,
    pub name_match_case: String,
    pub state_match_case: String,
    pub email_match_case: String,
    pub phone_match_case: String,
    pub alice: Entry,
    pub bob: Entry,
    pub carol: Entry,
    pub dan: Entry,
    pub erin: Entry,
}

impl Setup {
    pub fn new() -> Self {
        let rspace = RSpace::<Retrieve, Commit>::create().unwrap();
        let memconc = MemConcDB::<Retrieve, Commit>::create().unwrap();
        let memseq = MemSeqDB::<Retrieve, Commit>::create().unwrap();
        let diskconc = DiskConcDB::<Retrieve, Commit>::create().unwrap();
        let diskseq = DiskSeqDB::<Retrieve, Commit>::create().unwrap();

        // Alice
        let mut alice_name = Name::default();
        alice_name.first = "Alice".to_string();
        alice_name.last = "Lincoln".to_string();

        let mut alice_address = Address::default();
        alice_address.street = "777 Ford St".to_string();
        alice_address.city = "Crystal Lake".to_string();
        alice_address.state = "Idaho".to_string();
        alice_address.zip = "223322".to_string();

        let mut alice = Entry::default();
        alice.name = Some(alice_name);
        alice.address = Some(alice_address);
        alice.email = "alicel@ringworld.net".to_string();
        alice.phone = "787-555-1212".to_string();

        // Bob
        let mut bob_name = Name::default();
        bob_name.first = "Bob".to_string();
        bob_name.last = "Lahblah".to_string();

        let mut bob_address = Address::default();
        bob_address.street = "1000 Main St".to_string();
        bob_address.city = "Crystal Lake".to_string();
        bob_address.state = "Idaho".to_string();
        bob_address.zip = "223322".to_string();

        let mut bob = Entry::default();
        bob.name = Some(bob_name);
        bob.address = Some(bob_address);
        bob.email = "blablah@tenex.net".to_string();
        bob.phone = "698-555-1212".to_string();

        // Carol
        let mut carol_name = Name::default();
        carol_name.first = "Carol".to_string();
        carol_name.last = "Lahblah".to_string();

        let mut carol_address = Address::default();
        carol_address.street = "22 Goldwater Way".to_string();
        carol_address.city = "Herbert".to_string();
        carol_address.state = "Nevada".to_string();
        carol_address.zip = "334433".to_string();

        let mut carol = Entry::default();
        carol.name = Some(carol_name);
        carol.address = Some(carol_address);
        carol.email = "carol@blablah.org".to_string();
        carol.phone = "232-555-1212".to_string();

        // Dan
        let mut dan_name = Name::default();
        dan_name.first = "Dan".to_string();
        dan_name.last = "Walters".to_string();

        let mut dan_address = Address::default();
        dan_address.street = "40 Shady Lane".to_string();
        dan_address.city = "Crystal Lake".to_string();
        dan_address.state = "Idaho".to_string();
        dan_address.zip = "223322".to_string();

        let mut dan = Entry::default();
        dan.name = Some(dan_name);
        dan.address = Some(dan_address);
        dan.email = "deejwalters@sdf.lonestar.org".to_string();
        dan.phone = "444-555-1212".to_string();

        // Erin
        let mut erin_name = Name::default();
        erin_name.first = "Erin".to_string();
        erin_name.last = "Rush".to_string();

        let mut erin_address = Address::default();
        erin_address.street = "23 Market St.".to_string();
        erin_address.city = "Peony".to_string();
        erin_address.state = "Idaho".to_string();
        erin_address.zip = "224422".to_string();

        let mut erin = Entry::default();
        erin.name = Some(erin_name);
        erin.address = Some(erin_address);
        erin.email = "erush@lasttraintogoa.net".to_string();
        erin.phone = "333-555-1212".to_string();

        Setup {
            rspace,
            memconc,
            memseq,
            diskconc,
            diskseq,
            city_match_case: String::from("Crystal Lake"),
            name_match_case: String::from("Lahblah"),
            state_match_case: String::from("Idaho"),
            email_match_case: String::from("deejwalters@sdf.lonestar.org"),
            phone_match_case: String::from("333-555-1212"),
            alice,
            bob,
            carol,
            dan,
            erin,
        }
    }

    pub fn get_city_field(entry: Entry) -> String {
        entry.address.unwrap().city
    }

    pub fn get_last_name_field(entry: Entry) -> String {
        entry.name.unwrap().last
    }

    pub fn get_state_field(entry: Entry) -> String {
        entry.address.unwrap().state
    }

    pub fn create_retrieve(_channel: String, _data: Entry, _match_case: String) -> Retrieve {
        let mut retrieve = Retrieve::default();
        retrieve.chan = _channel;
        retrieve.data = Some(_data);
        retrieve.match_case = _match_case;
        retrieve
    }

    pub fn create_commit(
        _channels: Vec<String>,
        _patterns: Vec<String>,
        _continutation: String,
    ) -> Commit {
        let mut commit = Commit::default();
        commit.channels = _channels;
        commit.patterns = _patterns;
        commit.continuation = _continutation;
        commit
    }
}

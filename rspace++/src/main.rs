use rspace_plus_plus::{rtypes::rtypes::OptionResult, setup::Setup};
use std::error::Error;

mod diskconc;
mod diskseq;
mod memconc;
mod memseq;
mod rtypes;

fn run_k(ks: Vec<OptionResult>) {
    for k in ks {
        println!(
            "\nRunning continuation for {:?}...",
            k.data.unwrap().name.unwrap()
        );

        println!("\n{:?}", k.continuation);
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let setup = Setup::new();
    let rspace = setup.rspace;

    println!("\n**** Example 1 ****");

    let commit1 = Setup::create_commit(
        vec![String::from("friends")],
        vec![setup.city_match_case],
        String::from("I am the continuation, for now..."),
    );
    let _cres1 = rspace.put_once_durable_sequential(commit1);

    let _ = rspace.print_store("friends");

    let retrieve1 = Setup::create_retrieve(
        String::from("friends"),
        setup.alice.clone(),
        Setup::get_city_field(setup.alice),
    );
    let pres1 = rspace.get_once_durable_sequential(retrieve1);
    if pres1.is_some() {
        run_k(vec![pres1.unwrap()]);
    }
    let _ = rspace.print_store("friends");

    println!("\n**** Example 2 ****");

    let retrieve2 = Setup::create_retrieve(
        String::from("colleagues"),
        setup.dan.clone(),
        Setup::get_state_field(setup.dan),
    );
    let _pres2 = rspace.get_once_durable_concurrent(retrieve2);

    let retrieve3 = Setup::create_retrieve(
        String::from("friends"),
        setup.bob.clone(),
        Setup::get_state_field(setup.bob),
    );
    let _pres3 = rspace.get_once_durable_concurrent(retrieve3);

    let commit2 = Setup::create_commit(
        vec![String::from("friends"), String::from("colleagues")],
        vec![setup.state_match_case.clone(), setup.state_match_case],
        String::from("I am the continuation, for now..."),
    );
    let cres2 = rspace.put_once_durable_concurrent(commit2);
    if cres2.is_some() {
        run_k(cres2.unwrap());
    }
    let _ = rspace.print_store("friends");

    let _ = rspace.clear_store();
    assert!(rspace.is_empty());

    Ok(())
}

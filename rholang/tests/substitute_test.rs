// See rholang/src/test/scala/coop/rchain/rholang/interpreter/SubstituteTest.scala

use models::rhoapi::{connective::ConnectiveInstance, Connective, ConnectiveBody, Par, VarRef};
// use rand::{seq::SliceRandom, thread_rng, Rng};
use rholang::rust::interpreter::{
    accounting::{costs::Cost, CostManager},
    env::Env,
    matcher::prepend_connective,
    substitute::{Substitute, SubstituteTrait},
};

const DEPTH: i32 = 0;

fn env() -> Env<Par> {
    Env::new()
}

fn substitute_instance() -> Substitute {
    let cost = Cost::create(0, "substitute_test".to_string());
    let cost_manager = CostManager::new(cost, 1);
    let substitute = Substitute { cost: cost_manager };
    substitute
}

// fn generate_random_subsequence<T: Clone>(items: &[T]) -> Vec<T> {
//     let mut rng = thread_rng();
//     let subset: Vec<T> = items
//         .choose_multiple(&mut rng.clone(), rng.gen_range(0..=items.len()))
//         .cloned()
//         .collect();
//     // subset.sort();
//     subset
// }

#[test]
fn substitute_should_retain_all_non_empty_par_ed_connectives() {
    // let sample_connectives: Vec<Connective> = vec![
    //     Connective {
    //         connective_instance: Some(ConnectiveInstance::VarRefBody(VarRef {
    //             index: 0,
    //             depth: 0,
    //         })),
    //     },
    //     Connective {
    //         connective_instance: Some(ConnectiveInstance::ConnAndBody(ConnectiveBody {
    //             ps: vec![],
    //         })),
    //     },
    //     Connective {
    //         connective_instance: Some(ConnectiveInstance::ConnOrBody(ConnectiveBody {
    //             ps: vec![],
    //         })),
    //     },
    //     Connective {
    //         connective_instance: Some(ConnectiveInstance::ConnNotBody(Par::default())),
    //     },
    //     Connective {
    //         connective_instance: Some(ConnectiveInstance::ConnBool(false)),
    //     },
    //     Connective {
    //         connective_instance: Some(ConnectiveInstance::ConnInt(false)),
    //     },
    //     Connective {
    //         connective_instance: Some(ConnectiveInstance::ConnString(true)),
    //     },
    //     Connective {
    //         connective_instance: Some(ConnectiveInstance::ConnUri(true)),
    //     },
    //     Connective {
    //         connective_instance: Some(ConnectiveInstance::ConnByteArray(true)),
    //     },
    //     Connective {
    //         connective_instance: Some(ConnectiveInstance::ConnOrBody(ConnectiveBody {
    //             ps: vec![],
    //         })),
    //     },
    // ];

    // let doubled_connectives: Vec<Connective> = sample_connectives
    //     .clone()
    //     .into_iter()
    //     .chain(sample_connectives.into_iter())
    //     .collect();

    // for _ in 0..10 {
    //     // let random_connectives: Vec<Connective> = generate_random_subsequence(&doubled_connectives);

    //     let par = Par::default();
    //     for connective in &doubled_connectives {
    //         prepend_connective(par.clone(), connective.clone(), DEPTH);
    //     }

    //     let substitution = substitute_instance().substitute(par.clone(), DEPTH, &env());
    //     assert!(substitution.is_ok());
    //     assert_eq!(substitution.unwrap().connectives, par.connectives);
    // }
}

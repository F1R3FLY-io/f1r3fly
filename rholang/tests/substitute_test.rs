// See rholang/src/test/scala/coop/rchain/rholang/interpreter/SubstituteTest.scala

use models::{
    create_bit_vector,
    rhoapi::{
        connective::ConnectiveInstance, var::VarInstance, Connective, ConnectiveBody, Par, Send,
        Var, VarRef,
    },
    rust::{
        rholang::implicits::GPrivateBuilder,
        utils::{new_boundvar_par, new_freevar_var},
    },
};
use rand::{seq::SliceRandom, thread_rng, Rng};
use rholang::rust::interpreter::{
    accounting::{costs::Cost, CostManager},
    env::Env,
    matcher::prepend_connective,
    substitute::{Substitute, SubstituteTrait},
};
use rspace_plus_plus::rspace::history::Either;

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

fn generate_random_subsequence<T: Clone>(items: &[T]) -> Vec<T> {
    let mut rng = thread_rng();
    let subset: Vec<T> = items
        .choose_multiple(&mut rng.clone(), rng.gen_range(0..=items.len()))
        .cloned()
        .collect();
    // subset.sort();
    subset
}

#[test]
fn substitute_should_retain_all_non_empty_par_ed_connectives() {
    let sample_connectives: Vec<Connective> = vec![
        Connective {
            connective_instance: Some(ConnectiveInstance::VarRefBody(VarRef {
                index: 0,
                depth: 0,
            })),
        },
        Connective {
            connective_instance: Some(ConnectiveInstance::ConnAndBody(ConnectiveBody {
                ps: vec![],
            })),
        },
        Connective {
            connective_instance: Some(ConnectiveInstance::ConnOrBody(ConnectiveBody {
                ps: vec![],
            })),
        },
        Connective {
            connective_instance: Some(ConnectiveInstance::ConnNotBody(Par::default())),
        },
        Connective {
            connective_instance: Some(ConnectiveInstance::ConnBool(false)),
        },
        Connective {
            connective_instance: Some(ConnectiveInstance::ConnInt(false)),
        },
        Connective {
            connective_instance: Some(ConnectiveInstance::ConnString(true)),
        },
        Connective {
            connective_instance: Some(ConnectiveInstance::ConnUri(true)),
        },
        Connective {
            connective_instance: Some(ConnectiveInstance::ConnByteArray(true)),
        },
        Connective {
            connective_instance: Some(ConnectiveInstance::ConnOrBody(ConnectiveBody {
                ps: vec![],
            })),
        },
    ];

    let doubled_connectives: Vec<Connective> = sample_connectives
        .clone()
        .into_iter()
        .chain(sample_connectives.into_iter())
        .collect();

    for _ in 0..10 {
        let random_connectives: Vec<Connective> = generate_random_subsequence(&doubled_connectives);

        let par = Par::default();
        for connective in &random_connectives {
            prepend_connective(par.clone(), connective.clone(), DEPTH);
        }

        let substitution = substitute_instance().substitute(par.clone(), DEPTH, &env());
        assert!(substitution.is_ok());
        assert_eq!(substitution.unwrap().connectives, par.connectives);
    }
}

#[test]
fn free_var_substitute_should_throw_an_error() {
    let source = GPrivateBuilder::new_par();
    let mut env = Env::new();
    env = env.put(source);
    let substitution = substitute_instance().maybe_substitute_var(new_freevar_var(0), DEPTH, &env);
    assert!(substitution.is_err());
}

#[test]
fn bound_var_substitute_should_be_substituted_for_process() {
    let source = GPrivateBuilder::new_par();
    let mut env = Env::new();
    env = env.put(source.clone());
    let substitution = substitute_instance().maybe_substitute_var(
        Var {
            var_instance: Some(VarInstance::BoundVar(0)),
        },
        DEPTH,
        &env,
    );
    assert!(substitution.is_ok());
    assert_eq!(substitution.unwrap(), Either::Right(source))
}

#[test]
fn bound_var_substitute_should_be_substituted_with_expression() {
    let new_sends = vec![Send {
        chan: Some(new_boundvar_par(0, Vec::new(), false)),
        data: vec![Par::default()],
        persistent: false,
        locally_free: create_bit_vector(&vec![0]),
        connective_used: false,
    }];
    let source = Par::default().with_sends(new_sends);
    let mut env = Env::new();
    env = env.put(source.clone());
    let substitution = substitute_instance().maybe_substitute_var(
        Var {
            var_instance: Some(VarInstance::BoundVar(0)),
        },
        DEPTH,
        &env,
    );
    assert!(substitution.is_ok());
    assert_eq!(substitution.unwrap(), Either::Right(source))
}

#[test]
fn bound_var_substitute_should_be_left_unchanged() {
    let mut env = Env::new();
    env = env.put(GPrivateBuilder::new_par());
    env = env.put(GPrivateBuilder::new_par());
    env = env.shift(1);
    let substitution = substitute_instance().maybe_substitute_var(
        Var {
            var_instance: Some(VarInstance::BoundVar(0)),
        },
        DEPTH,
        &env,
    );
    assert!(substitution.is_ok());
    assert_eq!(
        substitution.unwrap(),
        Either::Left(Var {
            var_instance: Some(VarInstance::BoundVar(0)),
        })
    )
}

#[test]
fn send_substitute_should_leave_variables_not_in_the_environment_alone() {
    let mut env = Env::new();
    env = env.put(GPrivateBuilder::new_par());
    env = env.put(GPrivateBuilder::new_par());
    env = env.shift(1);

    let send = Send {
        chan: Some(new_boundvar_par(0, create_bit_vector(&vec![0]), false)),
        data: vec![Par::default()],
        persistent: false,
        locally_free: create_bit_vector(&vec![0]),
        connective_used: false,
    };

    let substitution = substitute_instance().substitute(send.clone(), DEPTH, &env);
    assert!(substitution.is_ok());
    assert_eq!(substitution.unwrap(), send)
}

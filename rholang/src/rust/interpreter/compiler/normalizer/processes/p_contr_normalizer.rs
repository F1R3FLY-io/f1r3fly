use super::exports::*;
use crate::rust::interpreter::compiler::normalize::{
    normalize_match_proc, NameVisitInputs, ProcVisitInputs, ProcVisitOutputs, VarSort,
};
use crate::rust::interpreter::compiler::normalizer::processes::utils::fail_on_invalid_connective;
use crate::rust::interpreter::compiler::normalizer::remainder_normalizer_matcher::normalize_match_name;
use crate::rust::interpreter::compiler::rholang_ast::{Block, Name, Names};
use crate::rust::interpreter::errors::InterpreterError;
use crate::rust::interpreter::matcher::has_locally_free::HasLocallyFree;
use crate::rust::interpreter::util::filter_and_adjust_bitset;
use models::rhoapi::{Par, Receive, ReceiveBind};
use models::rust::utils::union;
use std::collections::HashMap;

pub fn normalize_p_contr(
    name: &Name,
    formals: &Names,
    proc: &Box<Block>,
    input: ProcVisitInputs,
    env: &HashMap<String, Par>,
) -> Result<ProcVisitOutputs, InterpreterError> {
    let name_match_result = normalize_name(
        name,
        NameVisitInputs {
            bound_map_chain: input.bound_map_chain.clone(),
            free_map: input.free_map.clone(),
        },
        env,
    )?;

    let mut init_acc = (vec![], FreeMap::<VarSort>::default(), Vec::new());

    for name in formals.names.clone() {
        let res = normalize_name(
            &name,
            NameVisitInputs {
                bound_map_chain: input.clone().bound_map_chain.push(),
                free_map: init_acc.1.clone(),
            },
            env,
        )?;

        let result = fail_on_invalid_connective(&input, &res)?;

        // Accumulate the result
        init_acc.0.insert(0, result.par.clone());
        init_acc.1 = result.free_map.clone();
        init_acc.2 = union(
            init_acc.clone().2,
            result.par.locally_free(
                result.par.clone(),
                (input.bound_map_chain.depth() + 1) as i32,
            ),
        );
    }

    let remainder_result = normalize_match_name(&formals.cont, init_acc.1.clone())?;

    let new_enw = input
        .bound_map_chain
        .absorb_free(remainder_result.1.clone());
    let bound_count = remainder_result.1.count_no_wildcards();

    let body_result = normalize_match_proc(
        &proc.proc,
        ProcVisitInputs {
            par: Par::default(),
            bound_map_chain: new_enw,
            free_map: name_match_result.free_map.clone(),
        },
        env,
    )?;

    let receive = Receive {
        binds: vec![ReceiveBind {
            patterns: init_acc.0.clone().into_iter().rev().collect(),
            source: Some(name_match_result.par.clone()),
            remainder: remainder_result.0.clone(),
            free_count: bound_count as i32,
        }],
        body: Some(body_result.par.clone()),
        persistent: true,
        peek: false,
        bind_count: bound_count as i32,
        locally_free: union(
            name_match_result.par.locally_free(
                name_match_result.par.clone(),
                input.bound_map_chain.depth() as i32,
            ),
            union(
                init_acc.2,
                filter_and_adjust_bitset(body_result.par.clone().locally_free, bound_count),
            ),
        ),
        connective_used: name_match_result
            .par
            .connective_used(name_match_result.par.clone())
            || body_result.par.connective_used(body_result.par.clone()),
    };
    //I should create new Expr for prepend_expr and provide it instead of receive.clone().into
    let updated_par = input.clone().par.prepend_receive(receive);
    Ok(ProcVisitOutputs {
        par: updated_par,
        free_map: body_result.free_map,
    })
}

// See rholang/src/test/scala/coop/rchain/rholang/interpreter/compiler/normalizer/ProcMatcherSpec.scala
#[cfg(test)]
mod tests {

    use models::{
        create_bit_vector,
        rhoapi::{expr::ExprInstance, EPlus, Expr, Par, Receive, ReceiveBind},
        rust::utils::{new_boundvar_par, new_freevar_par, new_gint_par, new_send_par},
    };

    use crate::rust::interpreter::{
        compiler::{
            compiler::Compiler,
            normalize::{normalize_match_proc, VarSort},
            rholang_ast::{Block, Name, Names, ProcList, SendType},
        },
        errors::InterpreterError,
        test_utils::utils::proc_visit_inputs_and_env,
    };

    use super::{Proc, SourcePosition};

    #[test]
    fn p_contr_should_handle_a_basic_contract() {
        /*  new add in {
             contract add(ret, @x, @y) = {
               ret!(x + y)
             }
           }
           // new is simulated by bindings.
        */
        let p_contract = Proc::Contract {
            name: Name::new_name_var("add", 0, 0),
            formals: Names::create(
                vec![
                    Name::new_name_var("ret", 0, 0),
                    Name::new_name_quote_var("x", 0, 0),
                    Name::new_name_quote_var("y", 0, 0),
                ],
                None,
                0,
                0,
            ),
            proc: Box::new(Block {
                proc: Proc::Send {
                    name: Name::new_name_var("ret", 0, 0),
                    send_type: SendType::new_single(0, 0),
                    inputs: ProcList::create(
                        vec![Proc::Add {
                            left: Box::new(Proc::new_proc_var("x", 0, 0)),
                            right: Box::new(Proc::new_proc_var("y", 0, 0)),
                            line_num: 0,
                            col_num: 0,
                        }],
                        0,
                        0,
                    ),
                    line_num: 0,
                    col_num: 0,
                },
                line_num: 0,
                col_num: 0,
            }),
            line_num: 0,
            col_num: 0,
        };

        let (mut inputs, env) = proc_visit_inputs_and_env();
        inputs.bound_map_chain = inputs.bound_map_chain.put((
            "add".to_string(),
            VarSort::NameSort,
            SourcePosition::new(0, 0),
        ));

        let result = normalize_match_proc(&p_contract, inputs.clone(), &env);
        assert!(result.is_ok());

        let expected_result = inputs.par.prepend_receive(Receive {
            binds: vec![ReceiveBind {
                patterns: vec![
                    new_freevar_par(0, Vec::new()),
                    new_freevar_par(1, Vec::new()),
                    new_freevar_par(2, Vec::new()),
                ],
                source: Some(new_boundvar_par(0, create_bit_vector(&vec![0]), false)),
                remainder: None,
                free_count: 3,
            }],
            body: Some(new_send_par(
                new_boundvar_par(2, create_bit_vector(&vec![2]), false),
                vec![{
                    let mut par = Par::default().with_exprs(vec![Expr {
                        expr_instance: Some(ExprInstance::EPlusBody(EPlus {
                            p1: Some(new_boundvar_par(1, create_bit_vector(&vec![1]), false)),
                            p2: Some(new_boundvar_par(0, create_bit_vector(&vec![0]), false)),
                        })),
                    }]);
                    par.locally_free = create_bit_vector(&vec![0, 1]);
                    par
                }],
                false,
                create_bit_vector(&vec![0, 1, 2]),
                false,
                create_bit_vector(&vec![0, 1, 2]),
                false,
            )),
            persistent: true,
            peek: false,
            bind_count: 3,
            locally_free: create_bit_vector(&vec![0]),
            connective_used: false,
        });

        assert_eq!(result.clone().unwrap().par, expected_result);
        assert_eq!(result.unwrap().free_map, inputs.free_map);
    }

    #[test]
    fn p_contr_should_not_count_ground_values_in_the_formals_towards_the_bind_count() {
        /*  new ret5 in {
             contract ret5(ret, @5) = {
               ret!(5)
             }
           }
           // new is simulated by bindings.
        */
        let p_contract = Proc::Contract {
            name: Name::new_name_var("ret5", 0, 0),
            formals: Names::create(
                vec![
                    Name::new_name_var("ret", 0, 0),
                    Name::new_name_quote_ground_long_literal(5, 0, 0),
                ],
                None,
                0,
                0,
            ),
            proc: Box::new(Block {
                proc: Proc::Send {
                    name: Name::new_name_var("ret", 0, 0),
                    send_type: SendType::new_single(0, 0),
                    inputs: ProcList::create(vec![Proc::new_proc_int(5, 0, 0)], 0, 0),
                    line_num: 0,
                    col_num: 0,
                },
                line_num: 0,
                col_num: 0,
            }),
            line_num: 0,
            col_num: 0,
        };

        let (mut inputs, env) = proc_visit_inputs_and_env();
        inputs.bound_map_chain = inputs.bound_map_chain.put((
            "ret5".to_string(),
            VarSort::NameSort,
            SourcePosition::new(0, 0),
        ));

        let result = normalize_match_proc(&p_contract, inputs.clone(), &env);
        assert!(result.is_ok());

        let expected_result = inputs.par.prepend_receive(Receive {
            binds: vec![ReceiveBind {
                patterns: vec![
                    new_freevar_par(0, Vec::new()),
                    new_gint_par(5, Vec::new(), false),
                ],
                source: Some(new_boundvar_par(0, create_bit_vector(&vec![0]), false)),
                remainder: None,
                free_count: 1,
            }],
            body: Some(new_send_par(
                new_boundvar_par(0, create_bit_vector(&vec![0]), false),
                vec![new_gint_par(5, Vec::new(), false)],
                false,
                create_bit_vector(&vec![0]),
                false,
                create_bit_vector(&vec![0]),
                false,
            )),
            persistent: true,
            peek: false,
            bind_count: 1,
            locally_free: create_bit_vector(&vec![0]),
            connective_used: false,
        });

        assert_eq!(result.clone().unwrap().par, expected_result);
        assert_eq!(result.unwrap().free_map, inputs.free_map);
    }

    #[test]
    fn p_contr_should_not_compile_when_logical_or_or_not_is_used_in_the_pattern_of_the_receive() {
        let result1 =
            Compiler::source_to_adt(r#"new x in { contract x(@{ y /\ {Nil \/ Nil}}) = { Nil } }"#);
        assert!(result1.is_err());
        assert_eq!(
            result1,
            Err(InterpreterError::PatternReceiveError(format!(
                "\\/ (disjunction) at {:?}",
                SourcePosition { row: 0, column: 31 }
            )))
        );

        let result2 =
            Compiler::source_to_adt(r#"new x in { contract x(@{ y /\ ~Nil}) = { Nil } }"#);
        assert!(result2.is_err());
        assert_eq!(
            result2,
            Err(InterpreterError::PatternReceiveError(format!(
                "~ (negation) at {:?}",
                SourcePosition { row: 0, column: 30 }
            )))
        );
    }

    #[test]
    fn p_contr_should_compile_when_logical_and_is_used_in_the_pattern_of_the_receive() {
        let result1 =
            Compiler::source_to_adt(r#"new x in { contract x(@{ y /\ {Nil /\ Nil}}) = { Nil } }"#);
        assert!(result1.is_ok());
    }
}

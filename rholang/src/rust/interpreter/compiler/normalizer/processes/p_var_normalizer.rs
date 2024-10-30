use models::rust::utils::{new_boundvar_expr, new_freevar_expr, new_wildcard_expr};

use crate::rust::interpreter::compiler::exports::{BoundContext, FreeContext};
use crate::rust::interpreter::compiler::normalize::VarSort;

use super::exports::*;
use std::result::Result;

pub fn normalize_p_var(
    p: &Proc,
    mut input: ProcVisitInputs,
) -> Result<ProcVisitOutputs, InterpreterError> {
    match p {
        Proc::Var(var) => {
            let var_name = var.name.clone();
            let row = var.line_num;
            let column = var.col_num;

            match input.bound_map_chain.get(&var_name) {
                Some(BoundContext {
                    index,
                    typ,
                    source_position,
                }) => match typ {
                    VarSort::ProcSort => Ok(ProcVisitOutputs {
                        par: prepend_expr(
                            input.par,
                            new_boundvar_expr(index as i32),
                            input.bound_map_chain.depth() as i32,
                        ),
                        free_map: input.free_map,
                    }),
                    VarSort::NameSort => Err(InterpreterError::UnexpectedProcContext {
                        var_name,
                        name_var_source_position: source_position,
                        process_source_position: SourcePosition { row, column },
                    }),
                },

                None => match input.free_map.get(&var_name) {
                    Some(FreeContext {
                        source_position, ..
                    }) => Err(InterpreterError::UnexpectedReuseOfProcContextFree {
                        var_name,
                        first_use: source_position,
                        second_use: SourcePosition { row, column },
                    }),

                    None => {
                        let new_bindings_pair = input.free_map.put((
                            var_name,
                            VarSort::ProcSort,
                            SourcePosition { row, column },
                        ));

                        Ok(ProcVisitOutputs {
                            par: prepend_expr(
                                input.par,
                                new_freevar_expr(input.free_map.next_level as i32),
                                input.bound_map_chain.depth() as i32,
                            ),
                            free_map: new_bindings_pair,
                        })
                    }
                },
            }
        }

        Proc::Wildcard { line_num, col_num } => Ok(ProcVisitOutputs {
            par: {
                let mut par = prepend_expr(
                    input.par,
                    new_wildcard_expr(),
                    input.bound_map_chain.depth() as i32,
                );
                par.connective_used = true;
                par
            },
            free_map: input.free_map.add_wildcard(SourcePosition {
                row: *line_num,
                column: *col_num,
            }),
        }),

        _ => Err(InterpreterError::NormalizerError(format!(
            "Expected Proc::Var or Proc::Wildcard, found {:?}",
            p,
        ))),
    }
}

#[cfg(test)]
mod tests {
    use crate::rust::interpreter::compiler::exports::BoundMapChain;
    use crate::rust::interpreter::compiler::rholang_ast::Var;

    use super::*;
    use models::create_bit_vector;
    use models::rhoapi::Par;

    fn inputs() -> ProcVisitInputs {
        ProcVisitInputs {
            par: Par::default(),
            bound_map_chain: BoundMapChain::new(),
            free_map: FreeMap::new(),
        }
    }

    fn p_var() -> Proc {
        Proc::Var(Var {
            name: "x".to_string(),
            line_num: 0,
            col_num: 0,
        })
    }

    #[test]
    fn p_var_should_compile_as_bound_var_if_its_in_env() {
        let bound_inputs = {
            let mut inputs = inputs();
            inputs.bound_map_chain = inputs.bound_map_chain.put((
                "x".to_string(),
                VarSort::ProcSort,
                SourcePosition::new(0, 0),
            ));
            inputs
        };

        let result = normalize_p_var(&p_var(), bound_inputs);
        assert!(result.is_ok());
        assert_eq!(
            result.clone().unwrap().par,
            prepend_expr(inputs().par, new_boundvar_expr(0), 0)
        );

        assert_eq!(result.clone().unwrap().free_map, inputs().free_map);
        assert_eq!(
            result.unwrap().par.locally_free,
            create_bit_vector(&vec![0])
        );
    }

    #[test]
    fn p_var_should_compile_as_free_var_if_its_not_in_env() {
        let result = normalize_p_var(&p_var(), inputs());
        assert!(result.is_ok());
        assert_eq!(
            result.clone().unwrap().par,
            prepend_expr(inputs().par, new_freevar_expr(0), 0)
        );

        assert_eq!(
            result.clone().unwrap().free_map,
            inputs().free_map.put((
                "x".to_string(),
                VarSort::ProcSort,
                SourcePosition::new(0, 0)
            ))
        );
    }

    #[test]
    fn p_var_should_not_compile_if_its_in_env_of_the_wrong_sort() {
        let bound_inputs = {
            let mut inputs = inputs();
            inputs.bound_map_chain = inputs.bound_map_chain.put((
                "x".to_string(),
                VarSort::NameSort,
                SourcePosition::new(0, 0),
            ));
            inputs
        };

        let result = normalize_p_var(&p_var(), bound_inputs);
        assert!(result.is_err());
        assert_eq!(
            result,
            Err(InterpreterError::UnexpectedProcContext {
                var_name: "x".to_string(),
                name_var_source_position: SourcePosition::new(0, 0),
                process_source_position: SourcePosition::new(0, 0),
            })
        )
    }

    #[test]
    fn p_var_should_not_compile_if_its_used_free_somewhere_else() {
        let bound_inputs = {
            let mut inputs = inputs();
            inputs.free_map = inputs.free_map.put((
                "x".to_string(),
                VarSort::ProcSort,
                SourcePosition::new(0, 0),
            ));
            inputs
        };

        let result = normalize_p_var(&p_var(), bound_inputs);
        assert!(result.is_err());
        assert_eq!(
            result,
            Err(InterpreterError::UnexpectedReuseOfProcContextFree {
                var_name: "x".to_string(),
                first_use: SourcePosition::new(0, 0),
                second_use: SourcePosition::new(0, 0)
            })
        )
    }
}

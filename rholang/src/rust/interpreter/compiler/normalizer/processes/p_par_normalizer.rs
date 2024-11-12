use super::exports::*;
use crate::rust::interpreter::compiler::normalize::{
    normalize_match_proc, ProcVisitInputs, ProcVisitOutputs,
};
use crate::rust::interpreter::errors::InterpreterError;
use models::rhoapi::Par;
use std::collections::HashMap;

pub fn normalize_p_par(
    left: &Proc,
    right: &Proc,
    input: ProcVisitInputs,
    env: &HashMap<String, Par>,
) -> Result<ProcVisitOutputs, InterpreterError> {
    let result = normalize_match_proc(&left, input.clone(), env)?;
    let chained_input = ProcVisitInputs {
        par: result.par.clone(),
        free_map: result.free_map.clone(),
        ..input.clone()
    };

    let chained_res = normalize_match_proc(&right, chained_input, env)?;
    Ok(chained_res)
}

// See rholang/src/test/scala/coop/rchain/rholang/interpreter/compiler/normalizer/ProcMatcherSpec.scala
#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use models::{
        create_bit_vector,
        rhoapi::Par,
        rust::utils::{new_boundvar_expr, new_freevar_expr, new_gint_expr},
    };

    use crate::rust::interpreter::{
        compiler::normalize::{normalize_match_proc, ProcVisitInputs, VarSort},
        errors::InterpreterError,
        test_utils::utils::proc_visit_inputs_and_env,
    };

    use super::{Proc, SourcePosition};

    #[test]
    fn p_par_should_compile_both_branches_into_a_par_object() {
        let par_ground = Proc::Par {
            left: Box::new(Proc::new_proc_int(7)),
            right: Box::new(Proc::new_proc_int(8)),
            line_num: 0,
            col_num: 0,
        };

        let result = normalize_match_proc(&par_ground, ProcVisitInputs::new(), &HashMap::new());
        assert!(result.is_ok());
        assert_eq!(
            result.clone().unwrap().par,
            Par::default().with_exprs(vec![new_gint_expr(8), new_gint_expr(7)])
        );
        assert_eq!(result.unwrap().free_map, ProcVisitInputs::new().free_map);
    }

    #[test]
    fn p_par_should_compile_both_branches_with_the_same_environment() {
        let par_double_bound = Proc::Par {
            left: Box::new(Proc::new_proc_var("x")),
            right: Box::new(Proc::new_proc_var("x")),
            line_num: 0,
            col_num: 0,
        };

        let (mut inputs, env) = proc_visit_inputs_and_env();
        inputs.bound_map_chain = inputs.bound_map_chain.put((
            "x".to_string(),
            VarSort::ProcSort,
            SourcePosition::new(0, 0),
        ));

        let result = normalize_match_proc(&par_double_bound, inputs, &env);
        assert!(result.is_ok());
        assert_eq!(result.clone().unwrap().par, {
            let mut par =
                Par::default().with_exprs(vec![new_boundvar_expr(0), new_boundvar_expr(0)]);
            par.locally_free = create_bit_vector(&vec![0]);
            par
        });
        assert_eq!(result.unwrap().free_map, ProcVisitInputs::new().free_map);
    }

    #[test]
    fn p_par_should_not_compile_if_both_branches_use_the_same_free_variable() {
        let par_double_free = Proc::Par {
            left: Box::new(Proc::new_proc_var("x")),
            right: Box::new(Proc::new_proc_var("x")),
            line_num: 0,
            col_num: 0,
        };

        let result =
            normalize_match_proc(&par_double_free, ProcVisitInputs::new(), &HashMap::new());
        assert!(result.is_err());
        assert_eq!(
            result,
            Err(InterpreterError::UnexpectedReuseOfProcContextFree {
                var_name: "x".to_string(),
                first_use: SourcePosition::new(0, 0),
                second_use: SourcePosition::new(0, 0)
            })
        );
    }

    #[test]
    fn p_par_should_accumulate_free_counts_from_both_branches() {
        let par_double_free = Proc::Par {
            left: Box::new(Proc::new_proc_var("x")),
            right: Box::new(Proc::new_proc_var("y")),
            line_num: 0,
            col_num: 0,
        };

        let result =
            normalize_match_proc(&par_double_free, ProcVisitInputs::new(), &HashMap::new());
        assert!(result.is_ok());
        assert_eq!(result.clone().unwrap().par, {
            let mut par = Par::default().with_exprs(vec![new_freevar_expr(1), new_freevar_expr(0)]);
            par.connective_used = true;
            par
        });
        assert_eq!(
            result.unwrap().free_map,
            ProcVisitInputs::new().free_map.put_all(vec![
                ("x".to_owned(), VarSort::ProcSort, SourcePosition::new(0, 0)),
                ("y".to_owned(), VarSort::ProcSort, SourcePosition::new(0, 0))
            ])
        )
    }

    /*
     * In this test case, 'huge_p_par' should iterate up to '50000'
     * Without passing 'RUST_MIN_STACK' env variable, this test case will fail with StackOverflowError
     * To test this correctly, change '50' to '50000' and run test with this command: 'RUST_MIN_STACK=2147483648 cargo test'
     *
     * 'RUST_MIN_STACK=2147483648' sets stack size to 2GB for rust program
     * 'RUST_MIN_STACK=1073741824' sets stack size to 1GB
     * 'RUST_MIN_STACK=536870912' sets stack size to 512MB
     */
    #[test]
    fn p_par_should_normalize_without_stack_overflow_error_even_for_huge_program() {
        let huge_p_par = (1..=50)
            .map(|x| Proc::new_proc_int(x as i64))
            .reduce(|l, r| Proc::Par {
                left: Box::new(l),
                right: Box::new(r),
                line_num: 0,
                col_num: 0,
            })
            .expect("Failed to create huge Proc::Par");

        let result = normalize_match_proc(&huge_p_par, ProcVisitInputs::new(), &HashMap::new());
        assert!(result.is_ok());
    }
}

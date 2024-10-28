use models::{
    create_bit_vector,
    rhoapi::Par,
    rust::utils::{new_boundvar_expr, new_freevar_expr},
};
use rholang::rust::interpreter::{
    compiler::{
        exports::{BoundMapChain, FreeMap, SourcePosition},
        normalize::{ProcVisitInputs, VarSort},
        normalizer::processes::p_var_normalizer::normalize_p_var,
        rholang_ast::{Proc, Var},
    },
    errors::InterpreterError,
    util::prepend_expr,
};

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

    let result = normalize_p_var(p_var(), bound_inputs);
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
    let result = normalize_p_var(p_var(), inputs());
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

    let result = normalize_p_var(p_var(), bound_inputs);
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

    let result = normalize_p_var(p_var(), bound_inputs);
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

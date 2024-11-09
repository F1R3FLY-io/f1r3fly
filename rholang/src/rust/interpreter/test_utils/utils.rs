use crate::rust::interpreter::compiler::bound_map_chain::BoundMapChain;
use crate::rust::interpreter::compiler::free_map::FreeMap;
use crate::rust::interpreter::compiler::normalize::VarSort::{NameSort, ProcSort};
use crate::rust::interpreter::compiler::normalize::{NameVisitInputs, ProcVisitInputs};
use crate::rust::interpreter::compiler::source_position::SourcePosition;
use models::rhoapi::Par;
use std::collections::HashMap;

pub fn name_visit_inputs_and_env() -> (NameVisitInputs, HashMap<String, Par>) {
    let input: NameVisitInputs = NameVisitInputs {
        bound_map_chain: BoundMapChain::default(),
        free_map: FreeMap::default(),
    };
    let env: HashMap<String, Par> = HashMap::new();

    (input, env)
}

pub fn proc_visit_inputs_and_env() -> (ProcVisitInputs, HashMap<String, Par>) {
    let proc_inputs = ProcVisitInputs {
        par: Default::default(),
        bound_map_chain: BoundMapChain::new(),
        free_map: Default::default(),
    };
    let env: HashMap<String, Par> = HashMap::new();

    (proc_inputs, env)
}

pub fn collection_proc_visit_inputs_and_env() -> (ProcVisitInputs, HashMap<String, Par>) {
    let proc_inputs = ProcVisitInputs {
        par: Default::default(),
        bound_map_chain: {
            let bound_map_chain = BoundMapChain::new();
            bound_map_chain.put_all(vec![
                (
                    "P".to_string(),
                    ProcSort,
                    SourcePosition { row: 0, column: 0 },
                ),
                (
                    "x".to_string(),
                    NameSort,
                    SourcePosition { row: 0, column: 0 },
                ),
            ])
        },
        free_map: Default::default(),
    };
    let env: HashMap<String, Par> = HashMap::new();

    (proc_inputs, env)
}

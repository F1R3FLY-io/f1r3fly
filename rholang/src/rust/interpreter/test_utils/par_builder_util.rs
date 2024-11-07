use crate::rust::interpreter::compiler::compiler::Compiler;
use crate::rust::interpreter::errors::InterpreterError;
use models::rhoapi::Par;
use std::collections::HashMap;

pub struct ParBuilderUtil;

impl ParBuilderUtil {
    pub fn mk_term(rho: &str) -> Result<Par, InterpreterError> {
        Compiler::source_to_adt_with_normalizer_env(rho, HashMap::new())
    }

    pub fn assert_compiled_equal(s: &str, t: &str) {
        let par_s = ParBuilderUtil::mk_term(s).expect("Compilation failed for the first string");
        let par_t = ParBuilderUtil::mk_term(t).expect("Compilation failed for the second string");
        assert_eq!(par_s, par_t, "Compiled Par values are not equal");
    }
}

// See rholang/src/main/scala/coop/rchain/rholang/interpreter/compiler/Compiler.scala

use models::{
    rhoapi::Par,
    rust::rholang::sorter::{par_sort_matcher::ParSortMatcher, sortable::Sortable},
};
use std::collections::HashMap;

use crate::rust::interpreter::{
    compiler::normalizer::parser::parse_rholang_code_to_proc, errors::InterpreterError,
};

use super::rholang_ast::Proc;

pub struct Compiler;

impl Compiler {
    pub fn source_to_adt(source: &str) -> Result<Par, InterpreterError> {
        Self::source_to_adt_with_normalizer_env(source, HashMap::new())
    }

    pub fn source_to_adt_with_normalizer_env(
        source: &str,
        normalizer_env: HashMap<String, Par>,
    ) -> Result<Par, InterpreterError> {
        let proc = Self::source_to_ast(source)?;
        Self::ast_to_adt_with_normalizer_env(proc, normalizer_env)
    }

    pub fn ast_to_adt(proc: Proc) -> Result<Par, InterpreterError> {
        Self::ast_to_adt_with_normalizer_env(proc, HashMap::new())
    }

    pub fn ast_to_adt_with_normalizer_env(
        proc: Proc,
        normalizer_env: HashMap<String, Par>,
    ) -> Result<Par, InterpreterError> {
        let par = Self::normalize_term(proc, normalizer_env)?;
        let sorted_par = ParSortMatcher::sort_match(&par);
        Ok(sorted_par.term)
    }

    pub fn source_to_ast(source: &str) -> Result<Proc, InterpreterError> {
        parse_rholang_code_to_proc(source)
    }

    fn normalize_term(
        term: Proc,
        normalizer_env: HashMap<String, Par>,
    ) -> Result<Par, InterpreterError> {
        todo!()
    }
}

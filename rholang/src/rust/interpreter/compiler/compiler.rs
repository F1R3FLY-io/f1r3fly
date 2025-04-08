// See rholang/src/main/scala/coop/rchain/rholang/interpreter/compiler/Compiler.scala

use std::collections::{BTreeMap, HashMap};

use crate::rust::interpreter::{
    errors::InterpreterError,
    normal_forms::Par,
    sort_matcher::{Sortable, Sorted},
};

use super::{
    bound_map_chain::BoundMapChain,
    normalize::normalize_match_proc,
    normalizer::{
        parser,
        processes::exports::{FreeMap, SourcePosition},
    },
    rholang_ast::{ASTBuilder, Proc},
};

pub struct Compiler<'src, 'env> {
    ast_builder: ASTBuilder<'src>,
    normalizer_env: BTreeMap<String, &'env Sorted<Par>>,
}

impl<'src, 'env> Compiler<'src, 'env> {
    pub fn new(source: &'src str) -> Compiler<'src, 'env> {
        Self::new_with_normalizer_env(source, HashMap::new())
    }

    pub fn new_with_normalizer_env<Env, K>(source: &'src str, env: Env) -> Compiler<'src, 'env>
    where
        Env: IntoIterator<Item = (K, &'env Sorted<Par>)>,
        K: ToOwned<Owned = String>,
    {
        let mut normalizer_env = BTreeMap::new();
        for (k, ref_par) in env {
            normalizer_env.insert(k.to_owned(), ref_par);
        }
        Compiler {
            ast_builder: ASTBuilder::new(source),
            normalizer_env,
        }
    }

    pub fn compile_to_adt(&'src self) -> Result<Sorted<Par>, InterpreterError> {
        let proc = self.parse_to_ast()?;
        let par = normalize_term(proc, &self.normalizer_env)?;
        let sorted_par = par.sort_match();
        Ok(sorted_par.term)
    }

    pub fn parse_to_ast(&'src self) -> Result<&Proc, InterpreterError> {
        parser::parse_rholang_code_to_proc(&self.ast_builder)
    }
}

fn normalize_term(
    term: &Proc,
    normalizer_env: &HashMap<String, Par>,
) -> Result<Par, InterpreterError> {
    let mut result = Par::default();
    let mut free_map = FreeMap::new();
    let mut bound_map_chain = BoundMapChain::new();
    normalize_match_proc(
        &term,
        &mut result,
        &mut free_map,
        &mut bound_map_chain,
        normalizer_env,
        SourcePosition::default(),
    )
    .and_then(|_| {
        if free_map.is_empty() {
            return Ok(result);
        }
        if !free_map.has_wildcards() && !free_map.has_connectives() {
            return Err(InterpreterError::TopLevelFreeVariablesNotAllowedError(
                free_map.iter_free_vars().collect(),
            ));
        }
        if free_map.has_connectives() {
            return Err(InterpreterError::TopLevelLogicalConnectivesNotAllowedError(
                free_map.iter_connectives().collect(),
            ));
        }
        return Err(InterpreterError::TopLevelWildcardsNotAllowedError(
            free_map.iter_wildcards().collect(),
        ));
    })
}

// See rholang/src/main/scala/coop/rchain/rholang/interpreter/compiler/Compiler.scala

use models::rhoapi::Par;

use crate::{
    aliases::EnvHashMap,
    errors::InterpreterError,
    sort_matcher::{Sortable, Sorted},
};

use super::{
    bound_map_chain::BoundMapChain,
    exports::{FreeMap, SourcePosition},
    normalizer::{normalize_match_proc, parser},
    rholang_ast::{ASTBuilder, Proc},
};

pub struct Compiler<'src> {
    ast_builder: ASTBuilder<'src>,
    normalizer_env: EnvHashMap<String, crate::normal_forms::Par>,
}

impl<'src> Compiler<'src> {
    pub fn new(source: &'src str) -> Compiler<'src> {
        Self::new_with_normalizer_env(
            source,
            EnvHashMap::<String, crate::normal_forms::Par>::new(),
        )
    }

    pub fn new_with_normalizer_env(
        source: &'src str,
        env: EnvHashMap<String, crate::normal_forms::Par>,
    ) -> Compiler<'src> {
        let mut normalizer_env = EnvHashMap::new();
        for (k, ref_par) in env {
            normalizer_env.insert(k.to_owned(), ref_par);
        }
        Compiler {
            ast_builder: ASTBuilder::new(source),
            normalizer_env,
        }
    }

    pub fn compile_to_adt(
        &'src self,
    ) -> Result<Sorted<crate::normal_forms::Par>, InterpreterError> {
        let proc = self.parse_to_ast()?;
        let par = normalize_term(proc, &self.normalizer_env)?;
        let sorted_par = crate::normal_forms::Par::from(par).sort_match();

        Ok(sorted_par.term)
    }

    pub fn parse_to_ast(&self) -> Result<&'src Proc, InterpreterError> {
        parser::parse_rholang_code_to_proc(&self.ast_builder)
    }
}

fn normalize_term(
    term: &Proc,
    normalizer_env: &EnvHashMap<String, crate::normal_forms::Par>,
) -> Result<crate::normal_forms::Par, InterpreterError> {
    let mut result: crate::normal_forms::Par = Par::default().into();
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

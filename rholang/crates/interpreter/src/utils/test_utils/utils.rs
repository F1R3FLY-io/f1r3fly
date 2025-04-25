use crate::aliases::EnvHashMap;
use crate::compiler::bound_map_chain::BoundMapChain;
use crate::compiler::compiler::Compiler;
use crate::compiler::free_map::FreeMap;
use crate::compiler::normalizer::normalize_match_proc;
use crate::compiler::rholang_ast::Proc;
use crate::compiler::source_position::SourcePosition;
use crate::errors::InterpreterError;
use crate::normal_forms::Par;
use crate::sort_matcher::{ScoredTerm, Sortable, Sorted};
use pretty_assertions::assert_eq;
use std::collections::{BTreeMap, BTreeSet};

pub(crate) fn test_normalize_match_proc<A, B>(proc: &Proc, before: A, after: B)
where
    A: FnOnce(&mut FreeMap, &mut BoundMapChain, &mut EnvHashMap<String, Par>) -> Par,
    B: FnOnce(Result<(&Par, &FreeMap), InterpreterError>),
{
    let mut free_map = FreeMap::new();
    let mut bound_map_chain = BoundMapChain::new();
    let mut env = EnvHashMap::new();

    let mut input_par = before(&mut free_map, &mut bound_map_chain, &mut env);

    let result = normalize_match_proc(
        proc,
        &mut input_par,
        &mut free_map,
        &mut bound_map_chain,
        &env,
        SourcePosition::default(),
    );

    after(result.and(Ok((&input_par, &free_map))));
}

pub(crate) fn with_bindings<F>(
    fun: F,
) -> impl FnOnce(&mut FreeMap, &mut BoundMapChain, &mut BTreeMap<String, Par>) -> Par
where
    F: FnOnce(&mut BoundMapChain),
{
    |_, bindings, _| {
        fun(bindings);
        Par::NIL
    }
}

pub(crate) fn with_par(
    par: Par,
) -> impl FnOnce(&mut FreeMap, &mut BoundMapChain, &mut BTreeMap<String, Par>) -> Par {
    |_, _, _| par
}

pub(crate) fn defaults()
-> impl FnOnce(&mut FreeMap, &mut BoundMapChain, &mut BTreeMap<String, Par>) -> Par {
    with_par(Par::NIL)
}

pub(crate) fn test<T>(
    name: &'static str,
    fun: T,
) -> impl FnOnce(Result<(&Par, &FreeMap), InterpreterError>)
where
    T: FnOnce(&Par, &FreeMap),
{
    |actual| {
        let temp = actual.expect(name);
        fun(temp.0, temp.1)
    }
}

pub(crate) fn set<I>(pars: I) -> BTreeSet<ScoredTerm<Par>>
where
    I: IntoIterator<Item = Par>,
{
    let mut result = BTreeSet::new();
    pars.into_iter().fold(&mut result, |acc, par| {
        acc.insert(par.sort_match());
        acc
    });
    return result;
}

pub(crate) fn map<I>(elems: I) -> BTreeMap<ScoredTerm<Par>, Sorted<Par>>
where
    I: IntoIterator<Item = (Par, Par)>,
{
    let mut result = BTreeMap::new();
    elems.into_iter().fold(&mut result, |acc, (k, v)| {
        acc.insert(k.sort_match(), v.sort_match().term);
        acc
    });
    return result;
}

pub(crate) fn assert_equal_normalized(lhs: &str, rhs: &str) {
    let c1 = Compiler::new(lhs);
    let c2 = Compiler::new(rhs);

    let r1 = c1.compile_to_adt();
    let r2 = c2.compile_to_adt();

    r1.and_then(|par1| {
        r2.inspect(|par2| {
            assert_eq!(&par1, par2);
        })
    })
    .expect("Normalization expected not to fail");
}

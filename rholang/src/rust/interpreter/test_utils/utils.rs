use crate::rust::interpreter::compiler::bound_map_chain::BoundMapChain;
use crate::rust::interpreter::compiler::compiler::Compiler;
use crate::rust::interpreter::compiler::free_map::FreeMap;
use crate::rust::interpreter::compiler::normalize::normalize_match_proc;
use crate::rust::interpreter::compiler::normalizer::processes::exports::{InterpreterError, Proc};
use crate::rust::interpreter::compiler::source_position::SourcePosition;
use crate::rust::interpreter::normal_forms::Par;
use crate::rust::interpreter::sort_matcher::{ScoredTerm, Sortable, Sorted};
use pretty_assertions::assert_eq;
use std::collections::{BTreeMap, BTreeSet, HashMap};

pub(crate) fn test_normalize_match_proc<B, A>(proc: &Proc, before: B, after: A)
where
    B: FnOnce(&mut FreeMap, &mut BoundMapChain, &mut HashMap<String, Par>) -> Par,
    A: FnOnce(Result<(&Par, &FreeMap), InterpreterError>),
{
    let mut free_map = FreeMap::new();
    let mut bound_map_chain = BoundMapChain::new();
    let mut env = HashMap::new();

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
) -> impl FnOnce(&mut FreeMap, &mut BoundMapChain, &mut HashMap<String, Par>) -> Par
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
) -> impl FnOnce(&mut FreeMap, &mut BoundMapChain, &mut HashMap<String, Par>) -> Par {
    |_, _, _| par
}

pub(crate) fn defaults(
) -> impl FnOnce(&mut FreeMap, &mut BoundMapChain, &mut HashMap<String, Par>) -> Par {
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

#[macro_export]
macro_rules! assert_matches {
    ($expression:expr, $pattern:pat => $assertions:block) => {
        match $expression {
            $pattern => $assertions,
            other => panic!(
                "assertion failed: `{:?}` does not match pattern `{}`",
                other,
                stringify!($pattern)
            ),
        }
    };
}

pub fn assert_matches<T, F>(result: Result<T, InterpreterError>, matcher: F)
where
    F: FnOnce(InterpreterError),
{
    match result {
        Ok(_) => panic!("Expected an error, but got Ok"),
        Err(e) => matcher(e),
    }
}

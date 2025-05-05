// See rholang/src/main/scala/coop/rchain/rholang/interpreter/compiler/normalizer/processes/PLetNormalizer.scala

use bitvec::vec::BitVec;
use smallvec::SmallVec;
use uuid::Uuid;

use crate::{
    aliases::EnvHashMap,
    compiler::{
        exports::{BoundMapChain, FreeMap, SourcePosition},
        normalizer::{
            name_normalize_matcher::normalize_name, normalize_match_proc,
            remainder_normalizer_matcher::handle_name_remainder,
        },
        rholang_ast::{
            ASTBuilder, AnnName, AnnProc, Binding, LinearBind, Name, NameDecl, NameRemainder, Proc,
            Receipt, SendType, Source,
        },
    },
    errors::InterpreterError,
    normal_forms::{EListBody, Match, MatchCase, Par, adjust_bitset, union, union_inplace},
};

pub fn normalize_p_let(
    decls: &[Binding],
    body: AnnProc,
    input_par: &mut Par,
    free_map: &mut FreeMap,
    bound_map_chain: &mut BoundMapChain,
    env: &EnvHashMap,
) -> Result<(), InterpreterError> {
    /*
    Let processes with a single bind or with sequential binds ";" are converted into match processes rather
    than input processes, so that each sequential bind doesn't add a new unforgeable name to the tuplespace.
    The Rholang 1.1 spec defines them as the latter. Because the Rholang 1.1 spec defines let processes in terms
    of a output process in concurrent composition with an input process, the let process appears to quote the
    process on the RHS of "<-" and bind it to the pattern on LHS. For example, in
        let x <- 1 in { Nil }
    the process (value) "1" is quoted and bound to "x" as a name. There is no way to perform an AST transformation
    of sequential let into a match process and still preserve these semantics, so we have to do an ADT transformation.
     */

    assert!(!decls.is_empty());
    let mut continuation_stack = SmallVec::<[DeclCont; 1]>::new();
    let mut total_free: u32 = 0;
    for decl in decls {
        let proc_list = &decl.procs;
        let mut ps = Vec::with_capacity(proc_list.len());
        let mut locally_free = BitVec::new();
        let mut connective_used = false;

        for proc in proc_list {
            let mut par = Par::default();

            normalize_match_proc(
                proc.proc,
                &mut par,
                free_map,
                bound_map_chain,
                env,
                proc.pos,
            )?;

            union_inplace(&mut locally_free, &par.locally_free);
            connective_used = connective_used | par.connective_used;
            ps.push(par);
        }

        let value_list_body = EListBody {
            ps,
            locally_free,
            connective_used,
            remainder: None,
        };

        let name_list = &decl.names;
        let mut pattern_known_free = FreeMap::new();
        let pattern_list_body = names_to_e_list(
            &name_list.names,
            &name_list.remainder,
            &mut pattern_known_free,
            bound_map_chain,
        )?;

        let next_locally_free = union(
            &value_list_body.locally_free,
            &pattern_list_body.locally_free,
        );
        let free_count = pattern_known_free.count_no_wildcards();
        total_free += free_count;
        let output = DeclCont {
            output: Match {
                target: Par::elist(value_list_body),
                cases: vec![MatchCase {
                    pattern: Par::elist(pattern_list_body),
                    source: Par::default(), // continuation
                    free_count,
                }],
                locally_free: next_locally_free,
                connective_used,
            },
            total_free,
        };
        continuation_stack.push(output);

        bound_map_chain.absorb_free(pattern_known_free);
    }

    let mut body_par = Par::default();
    normalize_match_proc(
        body.proc,
        &mut body_par,
        free_map,
        bound_map_chain,
        env,
        body.pos,
    )?;
    let result = continuation_stack
        .into_iter()
        .rfold(body_par, |continuation_par, accum| {
            let mut output = accum.output;
            let case = &mut output.cases[0];

            output.connective_used = output.connective_used || continuation_par.connective_used;
            union_inplace(
                &mut output.locally_free,
                adjust_bitset(&continuation_par.locally_free, accum.total_free as usize),
            );
            case.source = continuation_par;

            bound_map_chain.shift(case.free_count as usize);
            let mut result = Par::default();
            result.push_match(output);
            result
        });

    input_par.concat_with(result);
    Ok(())
}

pub fn normalize_p_concurrent_let(
    decls: &[Binding],
    body: AnnProc,
    input_par: &mut Par,
    free_map: &mut FreeMap,
    bound_map_chain: &mut BoundMapChain,
    env: &EnvHashMap,
    pos: SourcePosition,
) -> Result<(), InterpreterError> {
    assert!(!decls.is_empty());

    let n = decls.len();
    let variable_names =
        Vec::from_iter(std::iter::repeat_with(|| Uuid::new_v4().to_string()).take(n));
    let mut ps = Vec::with_capacity(n + 1);
    let mut linear_binds = Vec::with_capacity(n);

    variable_names
        .iter()
        .zip(decls)
        .for_each(|(variable_name, Binding { names, procs })| {
            let mut inputs = Vec::with_capacity(procs.len());
            inputs.extend(procs);
            ps.push(Proc::Send {
                name: Name::declare(variable_name),
                send_type: SendType::Single,
                inputs,
            });
            linear_binds.push(LinearBind {
                lhs: names.clone(), // cloning names is fast - see the custom `clone()` implementation
                rhs: Source::Simple {
                    name: AnnName::declare(variable_name, SourcePosition::default()),
                },
            });
        });
    ps.push(Proc::ForComprehension {
        receipts: vec![Receipt::Linear(linear_binds)],
        proc: body,
    });

    let builder = ASTBuilder::with_capacity(n);
    let ppar = builder.fold_procs_into_par(&ps, std::iter::empty());
    let pnew = Proc::New {
        decls: variable_names
            .iter()
            .map(|var| NameDecl::id(var, SourcePosition::default()))
            .collect(),
        proc: ppar,
    };

    normalize_match_proc(&pnew, input_par, free_map, bound_map_chain, env, pos)
}

fn names_to_e_list(
    names: &[AnnName],
    name_remainder: &Option<NameRemainder>,
    free_map: &mut FreeMap,
    bound_map_chain: &mut BoundMapChain,
) -> Result<EListBody, InterpreterError> {
    let temp_env = EnvHashMap::new();
    let mut result = EListBody {
        ps: Vec::with_capacity(names.len()),
        connective_used: true,
        ..Default::default()
    };

    for name in names {
        let par = bound_map_chain
            .descend(|bound_map| normalize_name(name.0, free_map, bound_map, &temp_env, name.1))?;

        union_inplace(&mut result.locally_free, &par.locally_free);
        result.ps.push(par);
    }

    if let Some(remainder) = name_remainder {
        let optional_var = handle_name_remainder(*remainder, free_map)?;
        result.remainder = Some(optional_var);
    }

    Ok(result)
}

struct DeclCont {
    output: Match,
    total_free: u32,
}

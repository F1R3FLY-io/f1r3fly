use core::panic;
use std::fmt::{self, Debug, Formatter};

use itertools::Itertools;
use tree_sitter::{Node, Parser, Tree, TreeCursor};

use crate::rust::interpreter::{
    compiler::rholang_ast::{
        ASTBuilder, AnnName, AnnProc, Binding, Branch, BundleType, Case, Id, KeyValuePair,
        LinearBind, NameDecl, Names, PeekBind, Proc, Receipt, RepeatedBind, SendType, Source,
        VarRef, VarRefKind, GFALSE, GTRUE, NIL, TYPE_BOOL, TYPE_BYTEA, TYPE_INT, TYPE_STRING,
        TYPE_URI, WILD,
    },
    errors::InterpreterError,
};

use super::processes::exports::SourcePosition;

pub fn parse_rholang_code(code: impl AsRef<[u8]>) -> Tree {
    let mut parser = Parser::new();
    parser
        .set_language(&tree_sitter_rholang::LANGUAGE.into())
        .expect("Error loading Rholang grammar");
    parser.parse(code, None).expect("Failed to parse code")
}

pub fn parse_rholang_code_to_proc<'a>(
    ast_builder: &'a ASTBuilder<'a>,
) -> Result<&'a Proc<'a>, InterpreterError> {
    let tree = parse_rholang_code(ast_builder.get_source_code());
    let root_node = tree.root_node();
    let start_node = match root_node.named_child(0) {
        Some(node) => Ok(node),
        None => Err(InterpreterError::parser_error_from_node(
            "The code does not contain any valid Rholang process. Expected a child node",
            &root_node,
        )),
    }?;

    parse_proc_iter(&start_node, ast_builder)
}

enum K<'tree, 'ast> {
    EvalArrows {
        context: TreeCursor<'tree>,
        lenient: bool,
    },
    EvalDelayed {
        context: Node<'tree>,
    },
    EvalLet {
        bindings: TreeCursor<'tree>,
    },
    EvalList {
        context: TreeCursor<'tree>,
    },
    EvalNamedPairs {
        context: TreeCursor<'tree>,
        fst_selector: &'tree str,
        snd_selector: &'tree str,
    },
    ConsumeAdd {
        pos: SourcePosition,
    },
    ConsumeAnd {
        pos: SourcePosition,
    },
    ConsumeBundle {
        pos: SourcePosition,
        typ: BundleType,
    },
    ConsumeChoice {
        pos: SourcePosition,
        patterns: Vec<usize>,
    },
    ConsumeConcat {
        pos: SourcePosition,
    },
    ConsumeConjunction {
        pos: SourcePosition,
    },
    ConsumeContract {
        pos: SourcePosition,
        arity: usize,
        has_cont: bool,
    },
    ConsumeDiff {
        pos: SourcePosition,
    },
    ConsumeDisjunction {
        pos: SourcePosition,
    },
    ConsumeDiv {
        pos: SourcePosition,
    },
    ConsumeEq {
        pos: SourcePosition,
    },
    ConsumeEval {
        pos: SourcePosition,
    },
    ConsumeFor {
        pos: SourcePosition,
        arities: Vec<usize>,
    },
    ConsumeGt {
        pos: SourcePosition,
    },
    ConsumeGte {
        pos: SourcePosition,
    },
    ConsumeIfThen {
        pos: SourcePosition,
    },
    ConsumeIfThenElse {
        pos: SourcePosition,
    },
    ConsumeInterpolation {
        pos: SourcePosition,
    },
    ConsumeLet {
        pos: SourcePosition,
        arity: usize,
        concurrent: bool,
    },
    ConsumeLetBinding {
        rhs_len: usize,
    },
    ConsumeList {
        pos: SourcePosition,
        arity: usize,
    },
    ConsumeListWithRemainder {
        pos: SourcePosition,
        arity: usize,
    },
    ConsumeLt {
        pos: SourcePosition,
    },
    ConsumeLte {
        pos: SourcePosition,
    },
    ConsumeMap {
        pos: SourcePosition,
        arity: usize,
    },
    ConsumeMapWithRemainder {
        pos: SourcePosition,
        arity: usize,
    },
    ConsumeMatch {
        pos: SourcePosition,
        arity: usize,
    },
    ConsumeMatches {
        pos: SourcePosition,
    },
    ConsumeMethod {
        pos: SourcePosition,
        id: Id<'ast>,
        arity: usize,
    },
    ConsumeMod {
        pos: SourcePosition,
    },
    ConsumeMult {
        pos: SourcePosition,
    },
    ConsumeNames {
        arity: usize,
        has_cont: bool,
    },
    ConsumeNeg {
        pos: SourcePosition,
    },
    ConsumeNegation {
        pos: SourcePosition,
    },
    ConsumeNeq {
        pos: SourcePosition,
    },
    ConsumeNew {
        pos: SourcePosition,
        decls: Vec<NameDecl<'ast>>,
    },
    ConsumeNot {
        pos: SourcePosition,
    },
    ConsumeOr {
        pos: SourcePosition,
    },
    ConsumePar {
        pos: SourcePosition,
    },
    ConsumePeek,
    ConsumeQuote {
        pos: SourcePosition,
    },
    ConsumeReceiveSendBind,
    ConsumeRepeatedBind,
    ConsumeSendMultiple {
        pos: SourcePosition,
        arity: usize,
    },
    ConsumeSendReceiveBind {
        arity: usize,
    },
    ConsumeSendSingle {
        pos: SourcePosition,
        arity: usize,
    },
    ConsumeSendSync {
        pos: SourcePosition,
        arity: usize,
    },
    ConsumeSendSyncWithCont {
        pos: SourcePosition,
        arity: usize,
    },
    ConsumeSet {
        pos: SourcePosition,
        arity: usize,
    },
    ConsumeSetWithRemainder {
        pos: SourcePosition,
        arity: usize,
    },
    ConsumeSimpleBind,
    ConsumeSub {
        pos: SourcePosition,
    },
    ConsumeTuple {
        pos: SourcePosition,
        arity: usize,
    },
}

impl K<'_, '_> {
    fn expected_proc_stack_min_len(&self) -> usize {
        match self {
            K::ConsumeBundle { .. }
            | K::ConsumeEval { .. }
            | K::ConsumeNew { .. }
            | K::ConsumeNeg { .. }
            | K::ConsumeNegation { .. }
            | K::ConsumeNot { .. }
            | K::ConsumeQuote { .. } => 1,
            K::ConsumeAdd { .. }
            | K::ConsumeAnd { .. }
            | K::ConsumeConcat { .. }
            | K::ConsumeConjunction { .. }
            | K::ConsumeDiff { .. }
            | K::ConsumeDisjunction { .. }
            | K::ConsumeDiv { .. }
            | K::ConsumeEq { .. }
            | K::ConsumeGt { .. }
            | K::ConsumeGte { .. }
            | K::ConsumeIfThen { .. }
            | K::ConsumeInterpolation { .. }
            | K::ConsumeLt { .. }
            | K::ConsumeLte { .. }
            | K::ConsumeMatches { .. }
            | K::ConsumeMod { .. }
            | K::ConsumeMult { .. }
            | K::ConsumeNeq { .. }
            | K::ConsumeOr { .. }
            | K::ConsumePar { .. }
            | K::ConsumePeek
            | K::ConsumeRepeatedBind
            | K::ConsumeReceiveSendBind
            | K::ConsumeSimpleBind
            | K::ConsumeSub { .. } => 2,
            K::ConsumeIfThenElse { .. } => 3,
            K::ConsumeList { arity, .. }
            | K::ConsumeListWithRemainder { arity, .. }
            | K::ConsumeNames { arity, .. }
            | K::ConsumeSet { arity, .. }
            | K::ConsumeSetWithRemainder { arity, .. }
            | K::ConsumeTuple { arity, .. } => *arity,
            K::ConsumeLet { arity, .. }
            | K::ConsumeMethod { arity, .. }
            | K::ConsumeSendMultiple { arity, .. }
            | K::ConsumeSendSingle { arity, .. }
            | K::ConsumeSendSync { arity, .. } => arity + 1,
            K::ConsumeContract { arity, .. }
            | K::ConsumeSendSyncWithCont { arity, .. }
            | K::ConsumeSendReceiveBind { arity } => arity + 2,
            K::ConsumeLetBinding { rhs_len } => rhs_len + 1,
            K::ConsumeMap { arity, .. } => arity * 2,
            K::ConsumeMapWithRemainder { arity, .. } | K::ConsumeMatch { arity, .. } => {
                arity * 2 + 1
            }
            K::ConsumeFor { arities, .. } => {
                let bindings_count: usize = arities.iter().sum();
                bindings_count + 1
            }
            K::ConsumeChoice { patterns, .. } => {
                let pattern_count: usize = patterns.iter().sum();
                let branch_count = patterns.len();
                branch_count + pattern_count
            }
            _ => 0,
        }
    }
}

enum Step<'a> {
    Done,
    Continue(Node<'a>),
    Error(InterpreterError),
}

fn apply_cont<'tree, 'ast>(
    stack: &mut Vec<K<'tree, 'ast>>,
    procs: &mut Vec<StackItem<'ast>>,
    ast_builder: &'ast ASTBuilder<'ast>,
) -> Step<'tree> {
    loop {
        let cc;
        match stack.last_mut() {
            None => return Step::Done,
            Some(k) => cc = k,
        }

        match cc {
            K::EvalDelayed { context } => {
                let node = *context;
                stack.pop();
                return Step::Continue(node);
            }
            K::EvalList { context } => {
                if move_cursor_to_named(context) {
                    return Step::Continue(context.node());
                }
                stack.pop();
            }
            K::EvalNamedPairs {
                context,
                fst_selector,
                snd_selector,
            } => {
                if move_cursor_to_named(context) {
                    let child = context.node();
                    match get_named_pair(&child, fst_selector, snd_selector) {
                        Err(_) => {
                            stack.pop(); //on error we end the current continuation and try to apply the next one
                        }
                        Ok((fst, snd)) => {
                            stack.push(K::EvalDelayed { context: snd });
                            return Step::Continue(fst);
                        }
                    }
                } else {
                    stack.pop();
                };
            }
            K::EvalArrows { context, lenient } => {
                if !move_cursor_to_named(context) {
                    stack.pop();
                } else {
                    let binding_node = context.node();
                    match get_named_pair(&binding_node, "names", "input") {
                        Err(_) if *lenient => {
                            stack.pop();
                        }
                        Err(err) => return Step::Error(err),
                        Ok((lhs, rhs)) => {
                            let rhs_node_or_error = match binding_node.kind() {
                                "linear_bind" => match rhs.kind() {
                                    "simple_source" => {
                                        stack.push(K::ConsumeSimpleBind);
                                        rhs.named_child(0).ok_or(
                                            InterpreterError::parser_expects_named_node_at(
                                                0,
                                                &rhs,
                                                &binding_node,
                                            ),
                                        )
                                    }
                                    "receive_send_source" => {
                                        stack.push(K::ConsumeReceiveSendBind);
                                        rhs.named_child(0).ok_or(
                                            InterpreterError::parser_expects_named_node_at(
                                                0,
                                                &rhs,
                                                &binding_node,
                                            ),
                                        )
                                    }
                                    "send_receive_source" => {
                                        let some_inputs_or_error =
                                            get_child_by_field_name(&rhs, "inputs");
                                        some_inputs_or_error.and_then(|inputs| {
                                            stack.push(K::ConsumeSendReceiveBind {
                                                arity: inputs.named_child_count(),
                                            });
                                            stack.push(K::EvalList {
                                                context: inputs.walk(),
                                            });
                                            rhs.named_child(0).ok_or(
                                                InterpreterError::parser_expects_named_node_at(
                                                    0,
                                                    &rhs,
                                                    &binding_node,
                                                ),
                                            )
                                        })
                                    }
                                    what => Err(InterpreterError::parser_unexpected_node_kind(
                                        what, "source", &rhs,
                                    )),
                                },
                                "repeated_bind" => {
                                    stack.push(K::ConsumeRepeatedBind);
                                    Ok(rhs)
                                }
                                "peek_bind" => {
                                    stack.push(K::ConsumePeek);
                                    Ok(rhs)
                                }
                                what => Err(InterpreterError::parser_unexpected_node_kind(
                                    what,
                                    "receipt",
                                    &binding_node,
                                )),
                            };

                            match rhs_node_or_error {
                                Err(err) => return Step::Error(err),
                                Ok(rhs) => {
                                    stack.push(K::ConsumeNames {
                                        arity: lhs.named_child_count(),
                                        has_cont: lhs.child_by_field_name("cont").is_some(),
                                    });
                                    stack.push(K::EvalList {
                                        context: lhs.walk(),
                                    });
                                    return Step::Continue(rhs);
                                }
                            }
                        }
                    }
                }
            }
            K::EvalLet { bindings } => {
                if move_cursor_to_named(bindings) {
                    let decl_node = bindings.node();
                    match get_named_pair(&decl_node, "names", "procs") {
                        Err(err) => return Step::Error(err),
                        Ok((names, procs)) => {
                            stack.push(K::ConsumeLetBinding {
                                rhs_len: procs.named_child_count(),
                            });
                            stack.push(K::EvalList {
                                context: procs.walk(),
                            });
                            stack.push(K::ConsumeNames {
                                arity: names.named_child_count(),
                                has_cont: names.child_by_field_name("cont").is_some(),
                            });
                            stack.push(K::EvalList {
                                context: names.walk(),
                            });
                        }
                    }
                } else {
                    stack.pop();
                }
            }
            _ => {
                consume_cont(stack, procs, ast_builder);
            }
        }
    }
}

fn consume_cont<'tree, 'ast>(
    cont_stack: &mut Vec<K<'tree, 'ast>>,
    stack: &mut Vec<StackItem<'ast>>,
    ast_builder: &'ast ASTBuilder<'ast>,
) {
    let k;
    unsafe {
        expect_stack_len(
            cont_stack
                .last()
                .unwrap_unchecked()
                .expected_proc_stack_min_len(),
            stack,
            cont_stack,
        );
        k = cont_stack.pop().unwrap_unchecked();
    }

    match k {
        K::ConsumePar { pos } => {
            let left_right = consume_2proc_unchecked(stack);
            stack.push(StackItem::new_proc(
                ast_builder.alloc_par(left_right[0], left_right[1]),
                pos,
            ));
        }
        K::ConsumeSendSync { pos, arity } => {
            consume_procs_and_name_unchecked(stack, arity, |messages, name| {
                StackItem::new_proc(ast_builder.alloc_send_sync(name.0, messages), pos)
            });
        }
        K::ConsumeSendSyncWithCont { pos, arity } => {
            let cont = consume_top_as_proc_unchecked(stack);
            consume_procs_and_name_unchecked(stack, arity, |messages, name| {
                StackItem::new_proc(
                    ast_builder.alloc_send_sync_with_cont(name.0, messages, cont),
                    pos,
                )
            });
        }
        K::ConsumeNew { pos, mut decls } => {
            // We do this here because the sorting affects the numbering of variables inside the body.
            decls.sort_by(|a, b| a.uri.cmp(&b.uri));
            replace_top_proc_unchecked(stack, |top| {
                StackItem::new_proc(ast_builder.alloc_new(top.proc, decls), pos)
            });
        }
        K::ConsumeIfThen { pos } => {
            let condition_branch = consume_2proc_unchecked(stack);
            stack.push(StackItem::new_proc(
                ast_builder.alloc_if_then(condition_branch[0], condition_branch[1]),
                pos,
            ));
        }
        K::ConsumeIfThenElse { pos } => {
            let condition_branch_alternative = consume_3proc_unchecked(stack);
            stack.push(StackItem::new_proc(
                ast_builder.alloc_if_then_else(
                    condition_branch_alternative[0],
                    condition_branch_alternative[1],
                    condition_branch_alternative[2],
                ),
                pos,
            ));
        }
        K::ConsumeNames { arity, has_cont } => {
            let names = consume_name_vec_unchecked(stack, arity);
            stack.push(StackItem::new_names(
                Names::from_slice(&names, has_cont).expect("the parser expected names"),
            ));
        }
        K::ConsumeLetBinding { rhs_len } => {
            let procs = consume_proc_vec_unchecked(stack, rhs_len);
            let names = consume_top_as_names_unchecked(stack);
            stack.push(StackItem::new_let_binding(names, procs));
        }
        K::ConsumeLet {
            pos,
            arity,
            concurrent,
        } => {
            let bottom = stack.len() - arity;
            let bindings: Vec<_> = stack
                .drain(bottom..)
                .map(|item| {
                    if let StackItem::SomeLetBinding(binding) = item {
                        return binding;
                    };
                    panic_no_let(&item);
                })
                .collect();
            replace_top_proc_unchecked(stack, |body| {
                StackItem::new_proc(ast_builder.alloc_let(bindings, body, concurrent), pos)
            });
        }
        K::ConsumeBundle { pos, typ } => {
            replace_top_proc_unchecked(stack, |top| {
                StackItem::new_proc(ast_builder.alloc_bundle(typ, top.proc), pos)
            });
        }
        K::ConsumeMatch { pos, arity } => {
            let cases = consume_pairs(stack, arity, |pattern, proc| Case { pattern, proc });
            replace_top_proc_unchecked(stack, |expression| {
                StackItem::new_proc(ast_builder.alloc_match(expression, cases), pos)
            });
        }
        K::ConsumeContract {
            pos,
            arity,
            has_cont,
        } => {
            let body = consume_top_as_proc_unchecked(stack);
            let decls = consume_name_vec_unchecked(stack, arity);
            replace_top_proc_unchecked(stack, |name| {
                let formals = Names::from_slice(&decls, has_cont)
                    .expect("the parser expected correctly formed formal declarations");
                StackItem::new_proc(
                    ast_builder.alloc_contract(
                        name.try_into().expect("contract name expected"),
                        formals,
                        body,
                    ),
                    pos,
                )
            });
        }
        K::ConsumePeek => {
            consume_binding_from_top_unchecked(stack, |from, to| StackItem::new_peek(to, from));
        }
        K::ConsumeRepeatedBind => {
            consume_binding_from_top_unchecked(stack, |from, to| {
                StackItem::new_repeated_bind(to, from)
            });
        }
        K::ConsumeSimpleBind => {
            consume_binding_from_top_unchecked(stack, |from, to| {
                StackItem::new_linear_bind(to, Source::Simple { name: from })
            });
        }
        K::ConsumeReceiveSendBind => {
            consume_binding_from_top_unchecked(stack, |from, to| {
                StackItem::new_linear_bind(to, Source::ReceiveSend { name: from })
            });
        }
        K::ConsumeSendReceiveBind { arity } => {
            let inputs = consume_proc_vec_unchecked(stack, arity);
            consume_binding_from_top_unchecked(stack, |from, to| {
                StackItem::new_linear_bind(to, Source::SendReceive { name: from, inputs })
            });
        }
        K::ConsumeChoice { pos, patterns } => {
            let mut branches = Vec::with_capacity(patterns.len());
            for pat_size in patterns {
                let bottom = stack.len() - pat_size;
                let patterns = stack
                    .drain(bottom..)
                    .map(|binding| {
                        if let StackItem::SomeLinearBind(lin) = binding {
                            return lin;
                        }
                        panic_no_linear(&binding);
                    })
                    .collect_vec();
                let proc = consume_top_as_proc_unchecked(stack);
                branches.push(Branch { patterns, proc });
            }
            stack.push(StackItem::new_proc(ast_builder.alloc_choice(branches), pos));
        }
        K::ConsumeFor { pos, arities } => {
            let mut receipts = Vec::with_capacity(arities.len());
            for count in arities {
                receipts.push(consume_receipt(stack, count));
            }
            replace_top_proc_unchecked(stack, |body| {
                StackItem::new_proc(ast_builder.alloc_for(receipts, body), pos)
            })
        }
        K::ConsumeSendSingle { pos, arity } => {
            consume_procs_and_name_unchecked(stack, arity, |inputs, name| {
                StackItem::new_proc(
                    ast_builder.alloc_send(SendType::Single, name.0, inputs),
                    pos,
                )
            })
        }
        K::ConsumeSendMultiple { pos, arity } => {
            consume_procs_and_name_unchecked(stack, arity, |inputs, name| {
                StackItem::new_proc(
                    ast_builder.alloc_send(SendType::Multiple, name.0, inputs),
                    pos,
                )
            })
        }
        K::ConsumeMethod { pos, id, arity } => {
            let args = consume_proc_vec_unchecked(stack, arity);
            replace_top_proc_unchecked(stack, |receiver| {
                StackItem::new_proc(ast_builder.alloc_method(id, receiver.proc, args), pos)
            })
        }
        K::ConsumeEval { pos } => replace_top_proc_unchecked(stack, |proc| {
            StackItem::new_proc(
                ast_builder.alloc_eval(
                    proc.try_into()
                        .expect("the parser expected a name to be evaluated"),
                ),
                pos,
            )
        }),
        K::ConsumeQuote { pos } => replace_top_proc_unchecked(stack, |top| {
            StackItem::new_proc(ast_builder.alloc_quote(top.proc), pos)
        }),
        K::ConsumeOr { pos } => {
            let left_right = consume_2proc_unchecked(stack);
            stack.push(StackItem::new_proc(
                ast_builder.alloc_or(left_right[0], left_right[1]),
                pos,
            ));
        }
        K::ConsumeAnd { pos } => {
            let left_right = consume_2proc_unchecked(stack);
            stack.push(StackItem::new_proc(
                ast_builder.alloc_and(left_right[0], left_right[1]),
                pos,
            ));
        }
        K::ConsumeMatches { pos } => {
            let left_right = consume_2proc_unchecked(stack);
            stack.push(StackItem::new_proc(
                ast_builder.alloc_matches(left_right[0], left_right[1]),
                pos,
            ));
        }
        K::ConsumeEq { pos } => {
            let left_right = consume_2proc_unchecked(stack);
            stack.push(StackItem::new_proc(
                ast_builder.alloc_eq(left_right[0], left_right[1]),
                pos,
            ));
        }
        K::ConsumeNeq { pos } => {
            let left_right = consume_2proc_unchecked(stack);
            stack.push(StackItem::new_proc(
                ast_builder.alloc_neq(left_right[0], left_right[1]),
                pos,
            ));
        }
        K::ConsumeLt { pos } => {
            let left_right = consume_2proc_unchecked(stack);
            stack.push(StackItem::new_proc(
                ast_builder.alloc_lt(left_right[0], left_right[1]),
                pos,
            ));
        }
        K::ConsumeLte { pos } => {
            let left_right = consume_2proc_unchecked(stack);
            stack.push(StackItem::new_proc(
                ast_builder.alloc_lte(left_right[0], left_right[1]),
                pos,
            ));
        }
        K::ConsumeGt { pos } => {
            let left_right = consume_2proc_unchecked(stack);
            stack.push(StackItem::new_proc(
                ast_builder.alloc_gt(left_right[0], left_right[1]),
                pos,
            ));
        }
        K::ConsumeGte { pos } => {
            let left_right = consume_2proc_unchecked(stack);
            stack.push(StackItem::new_proc(
                ast_builder.alloc_gte(left_right[0], left_right[1]),
                pos,
            ));
        }
        K::ConsumeConcat { pos } => {
            let left_right = consume_2proc_unchecked(stack);
            stack.push(StackItem::new_proc(
                ast_builder.alloc_concat(left_right[0], left_right[1]),
                pos,
            ));
        }
        K::ConsumeDiff { pos } => {
            let left_right = consume_2proc_unchecked(stack);
            stack.push(StackItem::new_proc(
                ast_builder.alloc_diff(left_right[0], left_right[1]),
                pos,
            ));
        }
        K::ConsumeSub { pos } => {
            let left_right = consume_2proc_unchecked(stack);
            stack.push(StackItem::new_proc(
                ast_builder.alloc_sub(left_right[0], left_right[1]),
                pos,
            ));
        }
        K::ConsumeAdd { pos } => {
            let left_right = consume_2proc_unchecked(stack);
            stack.push(StackItem::new_proc(
                ast_builder.alloc_add(left_right[0], left_right[1]),
                pos,
            ));
        }
        K::ConsumeInterpolation { pos } => {
            let left_right = consume_2proc_unchecked(stack);
            stack.push(StackItem::new_proc(
                ast_builder.alloc_interpolation(left_right[0], left_right[1]),
                pos,
            ));
        }
        K::ConsumeMult { pos } => {
            let left_right = consume_2proc_unchecked(stack);
            stack.push(StackItem::new_proc(
                ast_builder.alloc_mult(left_right[0], left_right[1]),
                pos,
            ));
        }
        K::ConsumeDiv { pos } => {
            let left_right = consume_2proc_unchecked(stack);
            stack.push(StackItem::new_proc(
                ast_builder.alloc_div(left_right[0], left_right[1]),
                pos,
            ));
        }
        K::ConsumeMod { pos } => {
            let left_right = consume_2proc_unchecked(stack);
            stack.push(StackItem::new_proc(
                ast_builder.alloc_mod(left_right[0], left_right[1]),
                pos,
            ));
        }
        K::ConsumeNot { pos } => {
            replace_top_proc_unchecked(stack, |top| {
                StackItem::new_proc(ast_builder.alloc_not(top.proc), pos)
            });
        }
        K::ConsumeNeg { pos } => {
            replace_top_proc_unchecked(stack, |top| {
                StackItem::new_proc(ast_builder.alloc_neg(top.proc), pos)
            });
        }
        K::ConsumeConjunction { pos } => {
            let left_right = consume_2proc_unchecked(stack);
            stack.push(StackItem::new_proc(
                ast_builder.alloc_conjunction(left_right[0], left_right[1]),
                pos,
            ));
        }
        K::ConsumeDisjunction { pos } => {
            let left_right = consume_2proc_unchecked(stack);
            stack.push(StackItem::new_proc(
                ast_builder.alloc_disjunction(left_right[0], left_right[1]),
                pos,
            ));
        }
        K::ConsumeNegation { pos } => {
            replace_top_proc_unchecked(stack, |top| {
                StackItem::new_proc(ast_builder.alloc_negation(top.proc), pos)
            });
        }
        K::ConsumeList { pos, arity } => {
            let procs = consume_proc_vec_unchecked(stack, arity);
            stack.push(StackItem::new_proc(ast_builder.alloc_list(procs), pos));
        }
        K::ConsumeListWithRemainder { pos, arity } => {
            let remainder = consume_top_as_proc_unchecked(stack);
            let procs = consume_proc_vec_unchecked(stack, arity - 1);
            stack.push(StackItem::new_proc(
                ast_builder.alloc_list_with_remainder(
                    procs,
                    remainder.try_into().expect("proc var expected"),
                ),
                pos,
            ));
        }
        K::ConsumeSet { pos, arity } => {
            let procs = consume_proc_vec_unchecked(stack, arity);
            stack.push(StackItem::new_proc(ast_builder.alloc_set(procs), pos));
        }
        K::ConsumeSetWithRemainder { pos, arity } => {
            let remainder = consume_top_as_proc_unchecked(stack);
            let procs = consume_proc_vec_unchecked(stack, arity - 1);
            stack.push(StackItem::new_proc(
                ast_builder.alloc_set_with_remainder(
                    procs,
                    remainder.try_into().expect("proc var expected"),
                ),
                pos,
            ));
        }
        K::ConsumeTuple { pos, arity } => {
            let procs = consume_proc_vec_unchecked(stack, arity);
            stack.push(StackItem::new_proc(ast_builder.alloc_tuple(procs), pos));
        }
        K::ConsumeMap { pos, arity } => {
            let pairs = consume_pairs(stack, arity, |key, value| KeyValuePair { key, value });
            stack.push(StackItem::new_proc(ast_builder.alloc_map(pairs), pos));
        }
        K::ConsumeMapWithRemainder { pos, arity } => {
            let remainder = consume_top_as_proc_unchecked(stack);
            let pairs = consume_pairs(stack, arity, |key, value| KeyValuePair { key, value });
            stack.push(StackItem::new_proc(
                ast_builder.alloc_map_with_remainder(
                    pairs,
                    remainder.try_into().expect("proc var expected"),
                ),
                pos,
            ));
        }

        _ => unreachable!("Only remaining continuations should be consumes at this point"),
    }
}

fn parse_proc_iter<'tree, 'adt>(
    start_node: &Node<'tree>,
    ast_builder: &'adt ASTBuilder<'adt>,
) -> Result<&'adt Proc<'adt>, InterpreterError> {
    let mut cont_stack = Vec::with_capacity(16);
    let mut proc_stack = Vec::with_capacity(32);

    let mut node = *start_node;

    'parse: loop {
        let pos = node.start_position().into();
        match node.kind() {
            "par" => {
                let (left, right) = get_left_and_right(&node)?;
                cont_stack.push(K::ConsumePar { pos });
                cont_stack.push(K::EvalDelayed { context: right });
                node = left;
                continue 'parse;
            }

            "send_sync" => {
                let name = get_child_by_field_name(&node, "name")?;
                let messages = get_child_by_field_name(&node, "messages")?;
                let arity = messages.named_child_count();
                let sync_send_cont_node = get_child_by_field_name(&node, "cont")?;
                match sync_send_cont_node.named_child(0) {
                    Some(choice_node) => match choice_node.kind() {
                        "empty_cont" => {
                            cont_stack.push(K::ConsumeSendSync { pos, arity });
                            cont_stack.push(K::EvalList {
                                context: messages.walk(),
                            });
                        }
                        "non_empty_cont" => {
                            let proc_node = choice_node.named_child(0).ok_or(
                                InterpreterError::parser_expects_named_node_at(
                                    0,
                                    &choice_node,
                                    &node,
                                ),
                            )?;
                            cont_stack.push(K::ConsumeSendSyncWithCont { pos, arity });
                            cont_stack.push(K::EvalDelayed { context: proc_node });
                            cont_stack.push(K::EvalList {
                                context: messages.walk(),
                            });
                        }
                        what => {
                            return Err(InterpreterError::parser_unexpected_node_kind(
                                what,
                                "sync_send_cont",
                                &node,
                            ));
                        }
                    },
                    None => {
                        return Err(InterpreterError::parser_expects_child(
                            &sync_send_cont_node,
                            &node,
                        ));
                    }
                };
                node = name;
                continue 'parse;
            }

            "new" => {
                let decls_node = get_child_by_field_name(&node, "decls")?;
                let proc_node = get_child_by_field_name(&node, "proc")?;

                let decls = parse_decls(&decls_node, ast_builder)?;
                cont_stack.push(K::ConsumeNew { pos, decls });
                node = proc_node;
                continue 'parse;
            }

            "ifElse" => {
                let condition_node = get_child_by_field_name(&node, "condition")?;
                let if_true_node = get_child_by_field_name(&node, "ifTrue")?;
                match node.named_child(2) {
                    Some(alternative_proc_node) => {
                        cont_stack.push(K::ConsumeIfThenElse { pos });
                        cont_stack.push(K::EvalDelayed {
                            context: alternative_proc_node,
                        });
                    }
                    None => {
                        cont_stack.push(K::ConsumeIfThen { pos });
                    }
                };
                cont_stack.push(K::EvalDelayed {
                    context: if_true_node,
                });
                node = condition_node;
                continue 'parse;
            }

            "let" => {
                let decls_node = get_child_by_field_name(&node, "decls")?;
                let body_node = get_child_by_field_name(&node, "body")?;

                let concurrent = match decls_node.kind() {
                    "linear_decls" => false,
                    "conc_decls" => true,
                    what => {
                        return Err(InterpreterError::parser_unexpected_node_kind(
                            what, "decls", &node,
                        ))
                    }
                };
                cont_stack.push(K::ConsumeLet {
                    pos,
                    concurrent,
                    arity: decls_node.named_child_count(),
                });
                cont_stack.push(K::EvalLet {
                    bindings: decls_node.walk(),
                });

                node = body_node;
                continue 'parse;
            }

            "bundle" => {
                let bundle_node = get_child_by_field_name(&node, "bundle_type")?;

                let bundle = match bundle_node.kind() {
                    "bundle_write" => BundleType::BundleWrite,
                    "bundle_read" => BundleType::BundleRead,
                    "bundle_equiv" => BundleType::BundleEquiv,
                    "bundle_read_write" => BundleType::BundleReadWrite,

                    what => {
                        return Err(InterpreterError::parser_unexpected_node_kind(
                            what, "bundle", &node,
                        ))
                    }
                };

                let proc_node = get_child_by_field_name(&node, "proc")?;
                cont_stack.push(K::ConsumeBundle { pos, typ: bundle });
                node = proc_node;
                continue 'parse;
            }

            "match" => {
                let expression_node = get_child_by_field_name(&node, "expression")?;
                let cases_node = get_child_by_field_name(&node, "cases")?;

                cont_stack.push(K::ConsumeMatch {
                    pos,
                    arity: cases_node.named_child_count(),
                });
                cont_stack.push(K::EvalNamedPairs {
                    context: cases_node.walk(),
                    fst_selector: "pattern",
                    snd_selector: "proc",
                });

                node = expression_node;
                continue 'parse;
            }

            "choice" => {
                let branches_node = get_child_by_field_name(&node, "branches")?;

                let child_count = branches_node.named_child_count();
                let mut per_branch_continuations = Vec::with_capacity(child_count);
                let mut patterns = Vec::with_capacity(child_count);

                for branch_node in branches_node.named_children(&mut branches_node.walk()) {
                    let proc_node = get_child_by_field_name(&branch_node, "proc")?;

                    per_branch_continuations.push(K::EvalDelayed { context: proc_node });
                    per_branch_continuations.push(K::EvalArrows {
                        context: branch_node.walk(),
                        lenient: true,
                    });

                    let pat_size = branch_node
                        .children_by_field_name("pattern", &mut branch_node.walk())
                        .count();
                    patterns.push(pat_size);
                }

                cont_stack.push(K::ConsumeChoice { pos, patterns });

                cont_stack.extend(per_branch_continuations.into_iter().rev())
            }

            "contract" => {
                let name_node = get_child_by_field_name(&node, "name")?;
                let formals_node = get_child_by_field_name(&node, "formals")?;
                let proc_node = get_child_by_field_name(&node, "proc")?;

                cont_stack.push(K::ConsumeContract {
                    pos,
                    arity: formals_node.named_child_count(),
                    has_cont: formals_node.child_by_field_name("cont").is_some(),
                });
                cont_stack.push(K::EvalDelayed { context: proc_node });
                cont_stack.push(K::EvalList {
                    context: formals_node.walk(),
                });
                node = name_node;
                continue 'parse;
            }

            "input" => {
                let formals = get_child_by_field_name(&node, "formals")?;
                let proc_node = get_child_by_field_name(&node, "proc")?;

                let receipts_count = formals.named_child_count();
                let mut per_receipt_continuations = Vec::with_capacity(receipts_count);
                let mut receipts = Vec::with_capacity(receipts_count);

                for receipt_node in formals.named_children(&mut formals.walk()) {
                    per_receipt_continuations.push(K::EvalArrows {
                        context: receipt_node.walk(),
                        lenient: false,
                    });

                    let size = receipt_node.named_child_count();
                    receipts.push(size);
                }

                cont_stack.push(K::ConsumeFor {
                    pos,
                    arities: receipts,
                });
                cont_stack.extend(per_receipt_continuations.into_iter().rev());

                node = proc_node;
                continue 'parse;
            }

            "send" => {
                let name_node = get_child_by_field_name(&node, "name")?;
                let inputs_node = get_child_by_field_name(&node, "inputs")?;
                let arity = inputs_node.named_child_count();
                let send_type_node = get_child_by_field_name(&node, "send_type")?;
                match send_type_node.kind() {
                    "send_single" => cont_stack.push(K::ConsumeSendSingle { pos, arity }),
                    "send_multiple" => cont_stack.push(K::ConsumeSendMultiple { pos, arity }),
                    what => {
                        return Err(InterpreterError::parser_unexpected_node_kind(
                            what,
                            "send_type",
                            &node,
                        ))
                    }
                };

                cont_stack.push(K::EvalList {
                    context: inputs_node.walk(),
                });
                node = name_node;
                continue 'parse;
            }

            "method" => {
                let receiver_node = get_child_by_field_name(&node, "receiver")?;

                let name_node = get_child_by_field_name(&node, "name")?;
                let id = parse_var(&name_node, ast_builder)?;

                let args_node = get_child_by_field_name(&node, "args")?;

                cont_stack.push(K::ConsumeMethod {
                    pos,
                    id,
                    arity: args_node.named_child_count(),
                });

                cont_stack.push(K::EvalList {
                    context: args_node.walk(),
                });
                node = receiver_node;
                continue 'parse;
            }

            "eval" => {
                cont_stack.push(K::ConsumeEval { pos });
                node =
                    node.named_child(0)
                        .ok_or(InterpreterError::parser_expects_named_node_at(
                            0, &node, &node,
                        ))?;
                continue 'parse;
            }

            "quote" => {
                cont_stack.push(K::ConsumeQuote { pos });
                node =
                    node.named_child(0)
                        .ok_or(InterpreterError::parser_expects_named_node_at(
                            0, &node, &node,
                        ))?;
                continue 'parse;
            }

            "or" => {
                let (left, right) = get_left_and_right(&node)?;
                cont_stack.push(K::ConsumeOr { pos });
                cont_stack.push(K::EvalDelayed { context: right });
                node = left;
                continue 'parse;
            }

            "and" => {
                let (left, right) = get_left_and_right(&node)?;
                cont_stack.push(K::ConsumeAnd { pos });
                cont_stack.push(K::EvalDelayed { context: right });
                node = left;
                continue 'parse;
            }

            "matches" => {
                let (left, right) = get_left_and_right(&node)?;
                cont_stack.push(K::ConsumeMatches { pos });
                cont_stack.push(K::EvalDelayed { context: right });
                node = left;
                continue 'parse;
            }

            "eq" => {
                let (left, right) = get_left_and_right(&node)?;
                cont_stack.push(K::ConsumeEq { pos });
                cont_stack.push(K::EvalDelayed { context: right });
                node = left;
                continue 'parse;
            }

            "neq" => {
                let (left, right) = get_left_and_right(&node)?;
                cont_stack.push(K::ConsumeNeq { pos });
                cont_stack.push(K::EvalDelayed { context: right });
                node = left;
                continue 'parse;
            }

            "lt" => {
                let (left, right) = get_left_and_right(&node)?;
                cont_stack.push(K::ConsumeLt { pos });
                cont_stack.push(K::EvalDelayed { context: right });
                node = left;
                continue 'parse;
            }

            "lte" => {
                let (left, right) = get_left_and_right(&node)?;
                cont_stack.push(K::ConsumeLte { pos });
                cont_stack.push(K::EvalDelayed { context: right });
                node = left;
                continue 'parse;
            }

            "gt" => {
                let (left, right) = get_left_and_right(&node)?;
                cont_stack.push(K::ConsumeGt { pos });
                cont_stack.push(K::EvalDelayed { context: right });
                node = left;
                continue 'parse;
            }

            "gte" => {
                let (left, right) = get_left_and_right(&node)?;
                cont_stack.push(K::ConsumeGte { pos });
                cont_stack.push(K::EvalDelayed { context: right });
                node = left;
                continue 'parse;
            }

            "concat" => {
                let (left, right) = get_left_and_right(&node)?;
                cont_stack.push(K::ConsumeConcat { pos });
                cont_stack.push(K::EvalDelayed { context: right });
                node = left;
                continue 'parse;
            }

            "minus_minus" => {
                let (left, right) = get_left_and_right(&node)?;
                cont_stack.push(K::ConsumeDiff { pos });
                cont_stack.push(K::EvalDelayed { context: right });
                node = left;
                continue 'parse;
            }

            "minus" => {
                let (left, right) = get_left_and_right(&node)?;
                cont_stack.push(K::ConsumeSub { pos });
                cont_stack.push(K::EvalDelayed { context: right });
                node = left;
                continue 'parse;
            }

            "add" => {
                let (left, right) = get_left_and_right(&node)?;
                cont_stack.push(K::ConsumeAdd { pos });
                cont_stack.push(K::EvalDelayed { context: right });
                node = left;
                continue 'parse;
            }

            "percent_percent" => {
                let (left, right) = get_left_and_right(&node)?;
                cont_stack.push(K::ConsumeInterpolation { pos });
                cont_stack.push(K::EvalDelayed { context: right });
                node = left;
                continue 'parse;
            }

            "mult" => {
                let (left, right) = get_left_and_right(&node)?;
                cont_stack.push(K::ConsumeMult { pos });
                cont_stack.push(K::EvalDelayed { context: right });
                node = left;
                continue 'parse;
            }

            "div" => {
                let (left, right) = get_left_and_right(&node)?;
                cont_stack.push(K::ConsumeDiv { pos });
                cont_stack.push(K::EvalDelayed { context: right });
                node = left;
                continue 'parse;
            }

            "mod" => {
                let (left, right) = get_left_and_right(&node)?;
                cont_stack.push(K::ConsumeMod { pos });
                cont_stack.push(K::EvalDelayed { context: right });
                node = left;
                continue 'parse;
            }

            "not" => {
                let proc_node = get_child_by_field_name(&node, "proc")?;
                cont_stack.push(K::ConsumeNot { pos });
                node = proc_node;
                continue 'parse;
            }

            "neg" => {
                let proc_node = get_child_by_field_name(&node, "proc")?;
                cont_stack.push(K::ConsumeNeg { pos });
                node = proc_node;
                continue 'parse;
            }

            "disjunction" => {
                let (left, right) = get_left_and_right(&node)?;
                cont_stack.push(K::ConsumeDisjunction { pos });
                cont_stack.push(K::EvalDelayed { context: right });
                node = left;
                continue 'parse;
            }

            "conjunction" => {
                let (left, right) = get_left_and_right(&node)?;
                cont_stack.push(K::ConsumeConjunction { pos });
                cont_stack.push(K::EvalDelayed { context: right });
                node = left;
                continue 'parse;
            }

            "negation" => {
                let proc_node = get_child_by_field_name(&node, "proc")?;
                cont_stack.push(K::ConsumeNegation { pos });
                node = proc_node;
                continue 'parse;
            }

            "block" => {
                let body_node = get_child_by_field_name(&node, "body")?;
                node = body_node;
                continue 'parse;
            }

            "simple_type" => {
                let simple_type_value = ast_builder.get_node_value(&node)?;
                let simple_type = match simple_type_value {
                    "Bool" => Ok(&TYPE_BOOL),
                    "Int" => Ok(&TYPE_INT),
                    "String" => Ok(&TYPE_STRING),
                    "Uri" => Ok(&TYPE_URI),
                    "ByteArray" => Ok(&TYPE_BYTEA),
                    what => Err(InterpreterError::parser_unexpected_value(what, &node)),
                };
                proc_stack.push(StackItem::new_proc(simple_type?, pos));
            }
            "bool_literal" => {
                let bool_value = ast_builder.get_node_value(&node)?;
                let value = match bool_value {
                    "true" => Ok(&GTRUE),
                    "false" => Ok(&GFALSE),
                    what => Err(InterpreterError::parser_unexpected_value(what, &node)),
                };
                proc_stack.push(StackItem::new_proc(value?, pos));
            }
            "long_literal" => {
                let i64_string = ast_builder.get_node_value(&node)?;
                let value = i64_string.parse::<i64>().or_else(|e| {
                    Err(InterpreterError::from_parse_int_error(
                        i64_string, &e, &node,
                    ))
                });
                proc_stack.push(StackItem::new_proc(
                    ast_builder.alloc_long_literal(value?),
                    pos,
                ));
            }
            "string_literal" => {
                let value = ast_builder.get_node_value(&node)?;
                proc_stack.push(StackItem::new_proc(
                    ast_builder.alloc_string_literal(value),
                    pos,
                ));
            }
            "uri_literal" => {
                let value = ast_builder.get_node_value(&node)?;
                proc_stack.push(StackItem::new_proc(
                    ast_builder.alloc_uri_literal(value.into()),
                    pos,
                ));
            }
            "nil" => proc_stack.push(StackItem::new_proc(&NIL, pos)),

            "wildcard" => proc_stack.push(StackItem::new_proc(&WILD, pos)),
            "var" => {
                let id = parse_var(&node, ast_builder)?;
                proc_stack.push(StackItem::new_proc(ast_builder.alloc_var(id), pos))
            }

            "collection" => {
                let current_node = node
                    .child(0)
                    .ok_or(InterpreterError::parser_expects_child(&node, &node))?;
                let cont_proc = current_node
                    .child_by_field_name("cont")
                    .map(|cont_node| {
                        // 'cont_node' is '...' so it's sibling is the actual rule we want
                        cont_node.next_named_sibling().ok_or(
                            InterpreterError::parser_expects_named_node_inside(
                                &cont_node,
                                &current_node,
                            ),
                        )
                    })
                    .transpose()?;
                let arity = current_node.named_child_count();

                if cont_proc.is_some() {
                    match current_node.kind() {
                        "list" => cont_stack.push(K::ConsumeListWithRemainder { pos, arity }),
                        "set" => cont_stack.push(K::ConsumeSetWithRemainder { pos, arity }),
                        "map" => {
                            cont_stack.push(K::ConsumeMapWithRemainder { pos, arity });
                            cont_stack.push(K::EvalDelayed {
                                context: cont_proc.unwrap(),
                            });
                        }
                        what => {
                            return Err(InterpreterError::parser_unexpected_node_kind(
                                what,
                                "collection",
                                &node,
                            ));
                        }
                    }
                } else {
                    match current_node.kind() {
                        "list" => cont_stack.push(K::ConsumeList { pos, arity }),
                        "set" => cont_stack.push(K::ConsumeSet { pos, arity }),
                        "map" => cont_stack.push(K::ConsumeMap { pos, arity }),
                        "tuple" => cont_stack.push(K::ConsumeTuple { pos, arity }),
                        what => {
                            return Err(InterpreterError::parser_unexpected_node_kind(
                                what,
                                "collection",
                                &current_node,
                            ))
                        }
                    }
                }
                if current_node.kind() == "map" {
                    cont_stack.push(K::EvalNamedPairs {
                        context: current_node.walk(),
                        fst_selector: "key",
                        snd_selector: "value",
                    })
                } else {
                    cont_stack.push(K::EvalList {
                        context: current_node.walk(),
                    })
                }
            }

            "var_ref" => {
                let (var_ref_kind_node, var_node) = get_left_and_right(&node)?;

                let kind = var_ref_kind_node.child(0).map(|n| n.kind()).ok_or(
                    InterpreterError::parser_expects_child(&var_ref_kind_node, &node),
                )?;

                let var_ref_kind = match kind {
                    "=" => VarRefKind::Proc,
                    "=*" => VarRefKind::Name,
                    what => {
                        return Err(InterpreterError::parser_unexpected_node_kind(
                            what,
                            "var_ref_kind",
                            &node,
                        ))
                    }
                };
                let var = parse_var(&var_node, ast_builder)?;
                let var_ref = VarRef {
                    kind: var_ref_kind,
                    var,
                };

                proc_stack.push(StackItem::new_proc(ast_builder.alloc_var_ref(var_ref), pos))
            }

            _ => return Err(InterpreterError::parser_did_not_recognize(&node)),
        }

        let step = apply_cont(&mut cont_stack, &mut proc_stack, &ast_builder);
        match step {
            Step::Done => {
                assert!(proc_stack.len() == 1, "interpreter bug: parsing finished prematurely\n.Remaining process stack: {proc_stack:#?}");
                let last = proc_stack.last().unwrap();
                if let StackItem::SomeProc(result) = last {
                    return Ok(result.proc);
                }
                panic!("interpreter bug: parsing did not produce a process but {last:#?}")
            }
            Step::Error(err) => return Err(err),
            Step::Continue(n) => node = n,
        }
    }
}

fn parse_var<'a>(node: &Node, ast_builder: &'a ASTBuilder<'a>) -> Result<Id<'a>, InterpreterError> {
    let var_value = ast_builder.get_node_value(node)?;
    Ok(Id {
        name: var_value,
        pos: node.start_position().into(),
    })
}

fn parse_decls<'a>(
    decls_node: &Node,
    ast_builder: &'a ASTBuilder<'a>,
) -> Result<Vec<NameDecl<'a>>, InterpreterError> {
    let mut name_decls = Vec::with_capacity(decls_node.named_child_count());

    for child in decls_node.named_children(&mut decls_node.walk()) {
        let var_node = child
            .named_child(0)
            .filter(|n| n.kind() == "var")
            .ok_or(InterpreterError::parser_expects_kind("var", &child))?;
        let id = parse_var(&var_node, ast_builder)?;

        let uri = child
            .child_by_field_name("uri")
            .map(|uri_literal_node| {
                uri_literal_node
                    .next_named_sibling() // 'uri_literal_node' is '(' so it's sibling is the actual rule we want
                    .ok_or(InterpreterError::parser_expects_rule("uri_literal", &child))
                    .and_then(|rule_node| ast_builder.get_node_value(&rule_node))
                    .map(|value| value.into())
            })
            .transpose()?;

        name_decls.push(NameDecl { id, uri });
    }

    Ok(name_decls)
}

fn get_child_by_field_name<'a>(node: &Node<'a>, name: &str) -> Result<Node<'a>, InterpreterError> {
    node.child_by_field_name(name)
        .ok_or(InterpreterError::parser_expects_field(name, node))
}

fn get_named_pair<'a>(
    node: &Node<'a>,
    fst_selector: &str,
    snd_selector: &str,
) -> Result<(Node<'a>, Node<'a>), InterpreterError> {
    get_child_by_field_name(&node, fst_selector).and_then(|fst| {
        let snd_or_err = get_child_by_field_name(&node, snd_selector);
        snd_or_err.map(|snd| (fst, snd))
    })
}

fn get_left_and_right<'a>(node: &Node<'a>) -> Result<(Node<'a>, Node<'a>), InterpreterError> {
    let left = node
        .named_child(0)
        .ok_or(InterpreterError::parser_expects_named_node_at(
            0, node, node,
        ))?;
    let right = node
        .named_child(1)
        .ok_or(InterpreterError::parser_expects_named_node_at(
            1, node, node,
        ))?;
    Ok((left, right))
}

fn move_cursor_to_named(cursor: &mut TreeCursor) -> bool {
    let mut has_more = if cursor.depth() == 0 {
        cursor.goto_first_child()
    } else {
        cursor.goto_next_sibling()
    };
    while has_more && !cursor.node().is_named() {
        has_more = cursor.goto_next_sibling();
    }
    return has_more;
}

#[derive(Debug)]
enum StackItem<'ast> {
    SomeProc(AnnProc<'ast>),
    SomeNames(Names<'ast>),
    SomeLetBinding(Binding<'ast>),
    SomePeekBind(PeekBind<'ast>),
    SomeLinearBind(LinearBind<'ast>),
    SomeRepeatedBind(RepeatedBind<'ast>),
}

impl<'ast> StackItem<'ast> {
    fn new_proc(proc: &'ast Proc<'ast>, pos: SourcePosition) -> StackItem<'ast> {
        StackItem::SomeProc(AnnProc { proc, pos })
    }

    fn new_names(names: Names<'ast>) -> StackItem<'ast> {
        StackItem::SomeNames(names)
    }

    fn new_let_binding(names: Names<'ast>, procs: Vec<AnnProc<'ast>>) -> StackItem<'ast> {
        StackItem::SomeLetBinding(Binding { names, procs })
    }

    fn new_peek(names: Names<'ast>, input: AnnName<'ast>) -> StackItem<'ast> {
        StackItem::SomePeekBind(PeekBind {
            lhs: names,
            rhs: input,
        })
    }

    fn new_repeated_bind(names: Names<'ast>, input: AnnName<'ast>) -> StackItem<'ast> {
        StackItem::SomeRepeatedBind(RepeatedBind {
            lhs: names,
            rhs: input,
        })
    }

    fn new_linear_bind(names: Names<'ast>, input: Source<'ast>) -> StackItem<'ast> {
        StackItem::SomeLinearBind(LinearBind {
            lhs: names,
            rhs: input,
        })
    }
}

impl Debug for K<'_, '_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            K::ConsumeAdd { pos } => f.debug_struct("ConsumeAdd").field("pos", pos).finish(),
            K::ConsumeAnd { pos } => f.debug_struct("ConsumeAnd").field("pos", pos).finish(),
            K::ConsumeBundle { pos, typ } => f
                .debug_struct("ConsumeBundle")
                .field("type", typ)
                .field("pos", pos)
                .finish(),
            K::ConsumeChoice { pos, patterns } => f
                .debug_struct("ConsumeChoice")
                .field("pos", pos)
                .field("patterns", patterns)
                .finish(),
            K::ConsumeConcat { pos } => f.debug_struct("ConsumeConcat").field("pos", pos).finish(),
            K::ConsumeConjunction { pos } => f
                .debug_struct("ConsumeConjunction")
                .field("pos", pos)
                .finish(),
            K::ConsumeContract { pos, arity, .. } => f
                .debug_struct("ConsumeContract")
                .field("pos", pos)
                .field("arity", arity)
                .finish(),
            K::ConsumeDiff { pos } => f.debug_struct("ConsumeDiff").field("pos", pos).finish(),
            K::ConsumeDisjunction { pos } => f
                .debug_struct("ConsumeDisjunction")
                .field("pos", pos)
                .finish(),
            K::ConsumeDiv { pos } => f.debug_struct("ConsumeDiv").field("pos", pos).finish(),
            K::ConsumeEq { pos } => f.debug_struct("ConsumeEq").field("pos", pos).finish(),
            K::ConsumeEval { pos } => f.debug_struct("ConsumeEval").field("pos", pos).finish(),
            K::ConsumeFor { pos, arities } => f
                .debug_struct("ConsumeFor")
                .field("pos", pos)
                .field("arities", arities)
                .finish(),
            K::ConsumeGt { pos } => f.debug_struct("ConsumeGt").field("pos", pos).finish(),
            K::ConsumeGte { pos } => f.debug_struct("ConsumeGte").field("pos", pos).finish(),
            K::ConsumeIfThen { pos } => f.debug_struct("ConsumeIfThen").field("pos", pos).finish(),
            K::ConsumeIfThenElse { pos } => f
                .debug_struct("ConsumeIfThenElse")
                .field("pos", pos)
                .finish(),
            K::ConsumeInterpolation { pos } => f
                .debug_struct("ConsumeInteprolation")
                .field("pos", pos)
                .finish(),
            K::ConsumeLet {
                pos,
                arity,
                concurrent,
            } => f
                .debug_struct("ConsumeLet")
                .field("pos", pos)
                .field("arity", arity)
                .field("concurrent", concurrent)
                .finish(),
            K::ConsumeLetBinding { rhs_len } => f
                .debug_struct("ConsumeLetBinding")
                .field("rhs_len", rhs_len)
                .finish(),
            K::ConsumeList { pos, arity } => f
                .debug_struct("ConsumeList")
                .field("pos", pos)
                .field("arity", arity)
                .finish(),
            K::ConsumeListWithRemainder { pos, arity } => f
                .debug_struct("ConsumeListWithRemainder")
                .field("pos", pos)
                .field("arity", arity)
                .finish(),
            K::ConsumeLt { pos } => f.debug_struct("ConsumeLt").field("pos", pos).finish(),
            K::ConsumeLte { pos } => f.debug_struct("ConsumeLte").field("pos", pos).finish(),
            K::ConsumeMap { pos, arity } => f
                .debug_struct("ConsumeMap")
                .field("pos", pos)
                .field("arity", arity)
                .finish(),
            K::ConsumeMapWithRemainder { pos, arity } => f
                .debug_struct("ConsumeMapWithRemainder")
                .field("pos", pos)
                .field("arity", arity)
                .finish(),
            K::ConsumeMatch { pos, arity } => f
                .debug_struct("ConsumeMatch")
                .field("pos", pos)
                .field("arity", arity)
                .finish(),
            K::ConsumeMatches { pos } => {
                f.debug_struct("ConsumeMatches").field("pos", pos).finish()
            }
            K::ConsumeMethod { pos, id, arity } => f
                .debug_struct("ConsumeMethod")
                .field("pos", pos)
                .field("id", id)
                .field("arity", arity)
                .finish(),
            K::ConsumeMod { pos } => f.debug_struct("ConsumeMod").field("pos", pos).finish(),
            K::ConsumeMult { pos } => f.debug_struct("ConsumeMult").field("pos", pos).finish(),
            K::ConsumeNames { arity, .. } => f
                .debug_struct("ConsumeNames")
                .field("arity", arity)
                .finish(),
            K::ConsumeNeg { pos } => f.debug_struct("ConsumeNeg").field("pos", pos).finish(),
            K::ConsumeNegation { pos } => {
                f.debug_struct("ConsumeNegation").field("pos", pos).finish()
            }
            K::ConsumeNeq { pos } => f.debug_struct("ConsumeNeq").field("pos", pos).finish(),
            K::ConsumeNew { pos, decls } => f
                .debug_struct("ConsumeNew")
                .field("pos", pos)
                .field("decls", decls)
                .finish(),
            K::ConsumeNot { pos } => f.debug_struct("ConsumeNot").field("pos", pos).finish(),
            K::ConsumeOr { pos } => f.debug_struct("ConsumeOr").field("pos", pos).finish(),
            K::ConsumePar { pos } => f.debug_struct("ConsumePar").field("pos", pos).finish(),
            K::ConsumePeek => f.debug_struct("ConsumePeek").finish(),
            K::ConsumeQuote { pos } => f.debug_struct("ConsumeQuote").field("pos", pos).finish(),
            K::ConsumeReceiveSendBind => f.debug_struct("ConsumeReceiveSendBind").finish(),
            K::ConsumeRepeatedBind => f.debug_struct("ConsumeRepeatedBind").finish(),
            K::ConsumeSendMultiple { pos, arity } => f
                .debug_struct("ConsumeSendMultiple")
                .field("pos", pos)
                .field("arity", arity)
                .finish(),
            K::ConsumeSendReceiveBind { arity } => f
                .debug_struct("ConsumeSendReceiveBind")
                .field("arity", arity)
                .finish(),
            K::ConsumeSendSingle { pos, arity } => f
                .debug_struct("ConsumeSendSingle")
                .field("pos", pos)
                .field("arity", arity)
                .finish(),
            K::ConsumeSendSync { pos, arity } => f
                .debug_struct("ConsumeSendSync")
                .field("pos", pos)
                .field("arity", arity)
                .finish(),
            K::ConsumeSendSyncWithCont { pos, arity } => f
                .debug_struct("ConsumeSendSyncWithCont")
                .field("pos", pos)
                .field("arity", arity)
                .finish(),
            K::ConsumeSet { pos, arity } => f
                .debug_struct("ConsumeSet")
                .field("pos", pos)
                .field("arity", arity)
                .finish(),
            K::ConsumeSetWithRemainder { pos, arity } => f
                .debug_struct("ConsumeSetWithRemainder")
                .field("pos", pos)
                .field("arity", arity)
                .finish(),
            K::ConsumeSimpleBind => f.debug_struct("ConsumeSimpleBind").finish(),
            K::ConsumeSub { pos } => f.debug_struct("ConsumeSub").field("pos", pos).finish(),
            K::ConsumeTuple { pos, arity } => f
                .debug_struct("ConsumeTuple")
                .field("pos", pos)
                .field("arity", arity)
                .finish(),
            K::EvalArrows { context, .. } => f
                .debug_struct("EvalArrows")
                .field("at", &context.node())
                .finish(),
            K::EvalDelayed { context } => f
                .debug_struct("EvalDelayed")
                .field("context", context)
                .finish(),
            K::EvalLet { bindings } => f
                .debug_struct("EvalLet")
                .field("at", &bindings.node())
                .finish(),
            K::EvalList { context } => f
                .debug_struct("EvalList")
                .field("at", &context.node())
                .finish(),
            K::EvalNamedPairs {
                context,
                fst_selector,
                snd_selector,
            } => f
                .debug_struct("EvalNamedPairs")
                .field("at", &context.node())
                .field("fst_selector", fst_selector)
                .field("snd_selector", snd_selector)
                .finish(),
        }
    }
}

// some helpers for stack manipulation
#[inline]
fn expect_stack_len(n: usize, stack: &[StackItem], cont_stack: &[K]) {
    let len = stack.len();
    assert!(n <= len, "interpreter bug: process stack underflow!!!\nProcess stack: {stack:#?}\nContinuation stack: {cont_stack:#?}");
}

#[inline]
fn consume_top_as_proc_unchecked<'a>(stack: &mut Vec<StackItem<'a>>) -> AnnProc<'a> {
    unsafe {
        let top = stack.pop().unwrap_unchecked();
        if let StackItem::SomeProc(proc) = top {
            return proc;
        }
        panic_no_process(&top);
    }
}

#[inline]
fn consume_top_as_names_unchecked<'a>(stack: &mut Vec<StackItem<'a>>) -> Names<'a> {
    unsafe {
        let top = stack.pop().unwrap_unchecked();
        if let StackItem::SomeNames(names) = top {
            return names;
        }
        panic_no_names(&top);
    }
}

#[inline]
fn replace_top_proc_unchecked<'a, F>(stack: &mut Vec<StackItem<'a>>, replace: F)
where
    F: FnOnce(AnnProc<'a>) -> StackItem<'a>,
{
    unsafe {
        let elem = stack.last_mut().unwrap_unchecked();
        if let StackItem::SomeProc(top) = elem {
            *elem = replace(*top);
            return;
        }
        panic_no_process(elem);
    }
}

#[inline]
fn consume_2proc_unchecked<'a>(stack: &mut Vec<StackItem<'a>>) -> [AnnProc<'a>; 2] {
    unsafe {
        let b = stack.pop().unwrap_unchecked();
        let a = stack.pop().unwrap_unchecked();
        if let StackItem::SomeProc(top) = b {
            if let StackItem::SomeProc(top_1) = a {
                return [top_1, top];
            }
        }
        panic_no_processes(&[a, b]);
    }
}

#[inline]
fn consume_3proc_unchecked<'a>(stack: &mut Vec<StackItem<'a>>) -> [AnnProc<'a>; 3] {
    unsafe {
        let c = stack.pop().unwrap_unchecked();
        let b = stack.pop().unwrap_unchecked();
        let a = stack.pop().unwrap_unchecked();
        if let StackItem::SomeProc(top) = c {
            if let StackItem::SomeProc(top_1) = b {
                if let StackItem::SomeProc(top_2) = a {
                    return [top_2, top_1, top];
                }
            }
        }
        panic_no_processes(&[a, b, c]);
    }
}

#[inline]
fn proc_iterator<'a, 'b>(
    stack: &'b [StackItem<'a>],
    n: usize,
) -> impl Iterator<Item = &'b AnnProc<'a>> {
    let len = stack.len();
    let slice = &stack[(len - n)..];
    slice.iter().map(move |item| {
        if let StackItem::SomeProc(proc) = item {
            return proc;
        }
        panic_no_processes(slice);
    })
}

#[inline]
fn consume_proc_vec_unchecked<'a>(
    stack: &mut Vec<StackItem<'a>>,
    arity: usize,
) -> Vec<AnnProc<'a>> {
    let result: Vec<AnnProc<'a>> = proc_iterator(stack, arity).copied().collect();
    stack.truncate(stack.len() - arity);
    return result;
}

fn consume_procs_and_name_unchecked<'a, F>(
    stack: &mut Vec<StackItem<'a>>,
    arity: usize,
    consumer: F,
) where
    F: FnOnce(Vec<AnnProc<'a>>, AnnName<'a>) -> StackItem<'a>,
{
    let procs = consume_proc_vec_unchecked(stack, arity);
    replace_top_proc_unchecked(stack, |top| consumer(procs, force_name(top)))
}

fn consume_name_vec_unchecked<'a>(
    stack: &mut Vec<StackItem<'a>>,
    arity: usize,
) -> Vec<AnnName<'a>> {
    let result: Vec<AnnName<'a>> = proc_iterator(stack, arity)
        .map(|proc| force_name(*proc))
        .collect();
    stack.truncate(stack.len() - arity);
    return result;
}

fn consume_pairs<'a, Con, T: Copy>(stack: &mut Vec<StackItem<'a>>, count: usize, con: Con) -> Vec<T>
where
    Con: Fn(AnnProc<'a>, AnnProc<'a>) -> T,
{
    let consumed = count * 2;
    let mut result = Vec::with_capacity(consumed);
    result.extend(
        proc_iterator(stack, consumed)
            .tuples::<(_, _)>()
            .map(|(fst, snd)| con(*fst, *snd)),
    );
    stack.truncate(stack.len() - consumed);
    return result;
}

fn consume_binding_from_top_unchecked<'a, F>(stack: &mut Vec<StackItem<'a>>, consumer: F)
where
    F: FnOnce(AnnName<'a>, Names<'a>) -> StackItem<'a>,
{
    let names = consume_top_as_names_unchecked(stack);
    replace_top_proc_unchecked(stack, |input| {
        consumer(input.try_into().expect("source name expected"), names)
    })
}

fn consume_receipt<'a>(stack: &mut Vec<StackItem<'a>>, count: usize) -> Receipt<'a> {
    let bottom = stack.len() - count;
    let mut drain = stack.drain(bottom..);
    unsafe {
        let fst = drain.next().unwrap_unchecked();
        if let StackItem::SomeLinearBind(bind) = fst {
            let mut bindings = Vec::with_capacity(count);
            bindings.push(bind);
            bindings.extend(drain.map(|item| {
                if let StackItem::SomeLinearBind(bind) = item {
                    return bind;
                }
                panic_no_linear(&item)
            }));
            Receipt::Linear(bindings)
        } else if let StackItem::SomePeekBind(bind) = fst {
            let mut bindings = Vec::with_capacity(count);
            bindings.push(bind);
            bindings.extend(drain.map(|item| {
                if let StackItem::SomePeekBind(bind) = item {
                    return bind;
                }
                panic_no_peek(&item)
            }));
            Receipt::Peek(bindings)
        } else if let StackItem::SomeRepeatedBind(bind) = fst {
            let mut bindings = Vec::with_capacity(count);
            bindings.push(bind);
            bindings.extend(drain.map(|item| {
                if let StackItem::SomeRepeatedBind(bind) = item {
                    return bind;
                }
                panic_no_repeated(&item)
            }));
            Receipt::Repeated(bindings)
        } else {
            panic_no_binder(&fst)
        }
    }
}

#[inline]
fn force_name(proc: AnnProc) -> AnnName {
    proc.try_into().expect("a name expected")
}

fn panic_no_process(item: &StackItem) -> ! {
    panic!("interpreter bug: expected to find a process on the top of the process stack, but found {item:#?}");
}

fn panic_no_names(item: &StackItem) -> ! {
    panic!("interpreter bug: expected to find names on the top of the process stack, but found {item:#?}");
}

fn panic_no_processes(stack: &[StackItem]) -> ! {
    panic!("interpreter bug: expected to find process(es) on the top of the process stack, but found ... {stack:#?} <--- top")
}

fn panic_no_let(item: &StackItem) -> ! {
    panic!("interpreter bug: expected to find let binding on the top of the process stack, but found {item:#?}");
}

fn panic_no_binder(item: &StackItem) -> ! {
    panic!("interpreter bug: expected to find a binder on the top of the process stack, but found {item:#?}");
}

fn panic_no_linear(item: &StackItem) -> ! {
    panic!("unable to downcast {item:#?} into linear_bind");
}

fn panic_no_peek(item: &StackItem) -> ! {
    panic!("nable to downcast {item:#?} into peek_bind");
}

fn panic_no_repeated(item: &StackItem) -> ! {
    panic!("nable to downcast {item:#?} into repeated_bind");
}

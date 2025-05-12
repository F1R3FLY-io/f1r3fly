use crate::normal_forms::{self, EListBody, EMethodBody, ESetBody, ETupleBody, Expr, Par};
use crate::utils::{prepend_connective, prepend_expr};
use bitvec::vec::BitVec;
use models::rhoapi::connective::ConnectiveInstance;
use models::rhoapi::expr::ExprInstance::{self, EListBody, EMatchesBody, EMethodBody};
use models::rhoapi::var::VarInstance;
use models::rhoapi::{
    Bundle, Connective, ConnectiveBody, EAnd, EDiv, EEq, EGt, EGte, EList, ELt, ELte, EMatches,
    EMethod, EMinus, EMinusMinus, EMod, EMult, ENeg, ENeq, ENot, EOr, EPercentPercent, EPlus,
    EPlusPlus, ESet, ETuple, EVar, Match, MatchCase, New, Receive, ReceiveBind, Send, Var, VarRef,
};
use models::rust::bundle_ops::BundleOps;
use models::rust::par_map::ParMap;
use models::rust::par_map_type_mapper::ParMapTypeMapper;
use models::rust::par_set::ParSet;
use models::rust::par_set_type_mapper::ParSetTypeMapper;
use models::rust::rholang::implicits::{concatenate_pars, single_bundle};
use models::rust::rholang::sorter::match_sort_matcher::MatchSortMatcher;
use models::rust::rholang::sorter::new_sort_matcher::NewSortMatcher;
use models::rust::rholang::sorter::par_sort_matcher::ParSortMatcher;
use models::rust::rholang::sorter::receive_sort_matcher::ReceiveSortMatcher;
use models::rust::rholang::sorter::send_sort_matcher::SendSortMatcher;
use models::rust::rholang::sorter::sortable::Sortable;
use models::rust::sorted_par_hash_set::SortedParHashSet;
use models::rust::sorted_par_map::SortedParMap;
use rspace_plus_plus::rspace::history::Either;

use super::accounting::CostManager;
use super::accounting::costs::Cost;
use super::env::Env;
use super::errors::InterpreterError;

// See rholang/src/main/scala/coop/rchain/rholang/interpreter/Substitute.scala
pub trait SubstituteTrait<T> {
    fn substitute(&self, term: T, depth: u32, env: &Env<Par>) -> T;

    fn substitute_no_sort(&self, term: T, depth: u32, env: &Env<Par>) -> T;
}

#[derive(Clone)]
pub struct Substitute {
    pub cost: CostManager,
}

impl Substitute {
    pub fn substitute_and_charge<T>(&self, term: &T, depth: u32, env: &Env<Par>) -> T
    where
        Self: SubstituteTrait<T>,
        T: Clone + prost::Message,
    {
        let substitute = self.substitute(term.clone(), depth, env);
        self.cost.charge(Cost::create_from_generic(
            substitute.clone(),
            "substitution".to_string(),
        ));
        substitute
    }

    pub fn substitute_no_sort_and_charge<T>(&self, term: &A, depth: u32, env: &Env<Par>) -> T
    where
        Self: SubstituteTrait<T>,
        A: Clone + prost::Message,
    {
        let subst_term = self.substitute_no_sort(term.clone(), depth, env);
        self.cost.charge(Cost::create_from_generic(
            subst_term.clone(),
            "substitution".to_string(),
        ));
        subst_term
    }

    // pub here for testing purposes
    pub fn maybe_substitute_var(
        &self,
        term: normal_forms::Var,
        depth: u32,
        env: &Env<Par>,
    ) -> Result<Either<normal_forms::Var, Par>, InterpreterError> {
        if depth != 0 {
            Ok(Either::Left(term))
        } else {
            match term {
                normal_forms::Var::BoundVar(index) => match env.get(index as u32) {
                    Some(p) => Ok(Either::Right(p.clone())),
                    None => Ok(Either::Left(term)),
                },
                _ => Err(InterpreterError::SubstituteError(format!(
                    "Illegal Substitution [{:?}]",
                    term
                ))),
            }
        }
    }

    fn maybe_substitute_evar(
        &self,
        term: crate::normal_forms::Var,
        depth: u32,
        env: &Env<Par>,
    ) -> Result<Either<EVar, Par>, InterpreterError> {
        match self.maybe_substitute_var(term.into(), depth, env)? {
            Either::Left(v) => Ok(Either::Left(EVar { v: Some(v) })),
            Either::Right(p) => Ok(Either::Right(p)),
        }
    }

    fn maybe_substitute_var_ref(
        &self,
        term: crate::normal_forms::VarRef,
        depth: u32,
        env: &Env<Par>,
    ) -> Either<VarRef, Par> {
        if term.depth != depth {
            Either::Left(term.into())
        } else {
            match env.get(term.index as u32) {
                Some(p) => Either::Right(p.clone()),
                None => Either::Left(term.into()),
            }
        }
    }
}

impl SubstituteTrait<normal_forms::Bundle> for Substitute {
    fn substitute(
        &self,
        term: normal_forms::Bundle,
        depth: u32,
        env: &Env<Par>,
    ) -> normal_forms::Bundle {
        let sub_bundle = self.substitute(term.body.clone(), depth, env).into();
        let term_clone = term.clone();

        match single_bundle(&sub_bundle) {
            None => {
                let mut term_mut = term_clone;
                term_mut.body = sub_bundle.into();
                term_mut
            }
            Some(b) => BundleOps::merge(&term.into(), &b).into(),
        }
    }

    fn substitute_no_sort(
        &self,
        term: normal_forms::Bundle,
        depth: u32,
        env: &Env<Par>,
    ) -> normal_forms::Bundle {
        let sub_bundle = self
            .substitute_no_sort(term.clone().body, depth, env)
            .into();
        let term_clone = term.clone();

        match single_bundle(&sub_bundle) {
            Some(b) => BundleOps::merge(&term.into(), &b).into(),
            None => {
                let mut term_mut = term_clone;
                term_mut.body = sub_bundle.into();
                term_mut
            }
        }
    }
}

impl Substitute {
    fn sub_exp(
        &self,
        exprs: Vec<Expr>,
        depth: u32,
        env: &Env<crate::normal_forms::Par>,
    ) -> Result<Par, InterpreterError> {
        exprs
            .into_iter()
            .try_fold(Par::default(), |par, expr| match &expr {
                &Expr::EVar(var) => match self.maybe_substitute_evar(var, depth, env)? {
                    Either::Left(e_var) => Ok(prepend_expr(par.into(), expr, depth)),
                    Either::Right(right_par) => {
                        Ok(concatenate_pars(right_par.into(), par.into()).into())
                    }
                },
                _ => self
                    .substitute_no_sort(expr, depth, env)
                    .map(|e| prepend_expr(par, e, depth)),
            })
    }

    fn sub_conn(
        &self,
        conns: Vec<crate::normal_forms::Connective>,
        depth: u32,
        env: &Env<Par>,
    ) -> Par {
        conns
            .into_iter()
            .try_fold(Par::default(), |par, conn| match conn {
                crate::normal_forms::Connective::VarRef(v) => {
                    match self.maybe_substitute_var_ref(v.clone().into(), depth, env) {
                        Either::Left(_) => prepend_connective(par, conn, depth),
                        Either::Right(new_par) => {
                            concatenate_pars(new_par.into(), par.into()).into()
                        }
                    }
                }
                crate::normal_forms::Connective::ConnAnd(ps) => {
                    prepend_connective(par, conn, depth)
                }
                crate::normal_forms::Connective::ConnOr(ps) => prepend_connective(par, conn, depth),
                crate::normal_forms::Connective::ConnNot(p) => self
                    .substitute_no_sort(par, depth, env)
                    .map(|p| prepend_connective(p, conn, depth)),
                crate::normal_forms::Connective::ConnBool(c) => {
                    prepend_connective(par, conn, depth)
                }
                crate::normal_forms::Connective::ConnInt(c) => prepend_connective(par, conn, depth),
                crate::normal_forms::Connective::ConnString(c) => {
                    prepend_connective(par, conn, depth)
                }
                crate::normal_forms::Connective::ConnUri(c) => prepend_connective(par, conn, depth),
                crate::normal_forms::Connective::ConnByteArray(c) => {
                    prepend_connective(par, conn, depth)
                }
            })
    }
}

impl SubstituteTrait<Par> for Substitute {
    fn substitute_no_sort(&self, term: Par, depth: u32, env: &Env<Par>) -> Par {
        let exprs = self.sub_exp(term.exprs, depth, env);
        let connectives = self.sub_conn(term.connectives, depth, env);

        let sends = term
            .sends
            .into_iter()
            .map(|s| self.substitute_no_sort(s, depth, env))
            .map(Into::into)
            .collect::<Vec<Send>>();

        let bundles = term
            .bundles
            .iter()
            .map(|b| self.substitute_no_sort(b.clone(), depth, env))
            .map(Into::into)
            .collect::<Vec<Bundle>>();

        let receives = term
            .receives
            .iter()
            .map(|r| self.substitute_no_sort(r.clone(), depth, env))
            .map(Into::into)
            .collect::<Vec<Receive>>();

        let news = term
            .news
            .iter()
            .map(|n| self.substitute_no_sort(n.clone(), depth, env))
            .map(Into::into)
            .collect::<Vec<New>>();

        let matches = term
            .matches
            .iter()
            .map(|m| self.substitute_no_sort(m.clone(), depth, env))
            .map(Into::into)
            .collect::<Vec<normal_forms::Match>>();

        let right = Par {
            sends,
            receives,
            news,
            exprs: Vec::new(),
            matches,
            unforgeables: term.unforgeables,
            bundles,
            connectives: Vec::new(),
            locally_free: { set_bits_until(term.locally_free, env.shift) },
            connective_used: term.connective_used,
        };
        let right = concatenate_pars(connectives, right);
        concatenate_pars(exprs, right)
    }

    fn substitute(&self, term: Par, depth: u32, env: &Env<Par>) -> Par {
        self.substitute_no_sort(term, depth, env)
            .map(|p| ParSortMatcher::sort_match(&p))
            .map(|st| st.term)
    }
}

impl SubstituteTrait<normal_forms::Send> for Substitute {
    fn substitute_no_sort(
        &self,
        term: normal_forms::Send,
        depth: u32,
        env: &Env<Par>,
    ) -> normal_forms::Send {
        let channels_sub = self
            .substitute_no_sort(term.clone().chan, depth, env)
            .into();

        let f = |p: &Par| self.substitute_no_sort(p.clone(), depth, env);
        let data = term.data.iter().map(f).collect();

        let locally_free = set_bits_until(
            term.locally_free.into_iter().map(|v| v as u8).collect(),
            env.shift,
        );
        Send {
            chan: Some(channels_sub),
            data,
            persistent: term.persistent,
            locally_free,
            connective_used: term.connective_used,
        }
        .into()
    }

    fn substitute(&self, term: Send, depth: i32, env: &Env<Par>) -> Result<Send, InterpreterError> {
        self.substitute_no_sort(term, depth, env)
            .map(|s| SendSortMatcher::sort_match(&s))
            .map(|st| st.term)
    }
}

impl SubstituteTrait<normal_forms::Receive> for Substitute {
    fn substitute_no_sort(&self, term: Receive, depth: u32, env: &Env<Par>) -> Receive {
        let binds_sub = term
            .binds
            .into_iter()
            .map(
                |ReceiveBind {
                     patterns,
                     source,
                     remainder,
                     free_count,
                 }| {
                    let sub_channel =
                        self.substitute_no_sort(unwrap_option_safe(source)?, depth, env)?;
                    let sub_patterns = patterns
                        .iter()
                        .map(|p| self.substitute_no_sort(p.clone(), depth + 1, env))
                        .collect::<Result<Vec<Par>, InterpreterError>>()?;

                    Ok(ReceiveBind {
                        patterns: sub_patterns,
                        source: Some(sub_channel),
                        remainder,
                        free_count,
                    })
                },
            )
            .collect::<Result<Vec<ReceiveBind>, InterpreterError>>()?;

        let body_sub = self.substitute_no_sort(
            unwrap_option_safe(term.body)?,
            depth,
            &env.shift(term.bind_count),
        )?;

        normal_forms::Receive {
            binds: binds_sub,
            body: Some(body_sub),
            persistent: term.persistent,
            peek: term.peek,
            bind_count: term.bind_count,
            locally_free: set_bits_until(term.locally_free, env.shift),
            connective_used: term.connective_used,
        }
    }

    fn substitute(
        &self,
        term: Receive,
        depth: u32,
        env: &Env<Par>,
    ) -> Result<Receive, InterpreterError> {
        self.substitute_no_sort(term, depth, env)
            .map(|r| ReceiveSortMatcher::sort_match(&r))
            .map(|st| st.term)
    }
}

impl SubstituteTrait<normal_forms::New> for Substitute {
    fn substitute_no_sort(
        &self,
        term: New,
        depth: u32,
        env: &Env<Par>,
    ) -> Result<New, InterpreterError> {
        self.substitute_no_sort(
            unwrap_option_safe(term.p)?,
            depth,
            &env.shift(term.bind_count),
        )
        .map(|new_sub| New {
            bind_count: term.bind_count,
            p: Some(new_sub),
            uri: term.uri,
            injections: term.injections,
            locally_free: set_bits_until(term.locally_free, env.shift),
        })
    }

    fn substitute(&self, term: New, depth: i32, env: &Env<Par>) -> Result<New, InterpreterError> {
        self.substitute_no_sort(term, depth, env)
            .map(|n| NewSortMatcher::sort_match(&n))
            .map(|st| st.term)
    }
}

impl SubstituteTrait<normal_forms::Match> for Substitute {
    fn substitute_no_sort(
        &self,
        term: normal_forms::Match,
        depth: u32,
        env: &Env<Par>,
    ) -> normal_forms::Match {
        let target_sub = self.substitute_no_sort(term.target, depth, env);

        let cases = term
            .cases
            .iter()
            .map(
                |normal_forms::MatchCase {
                     pattern,
                     source,
                     free_count,
                 }| {
                    let par =
                        self.substitute_no_sort(source.clone(), depth, &env.shift(*free_count));

                    let sub_case = self.substitute_no_sort(pattern.clone(), depth + 1, env);

                    normal_forms::MatchCase {
                        pattern: sub_case,
                        source: par,
                        free_count: *free_count,
                    }
                },
            )
            .collect::<Vec<normal_forms::MatchCase>>();

        let locally_free = set_bits_until(term.locally_free.into(), env.shift).into();

        normal_forms::Match {
            target: target_sub,
            cases,
            locally_free,
            connective_used: term.connective_used,
        }
    }

    fn substitute(
        &self,
        term: normal_forms::Match,
        depth: u32,
        env: &Env<Par>,
    ) -> normal_forms::Match {
        self.substitute_no_sort(term, depth, env)
            .map(|m| MatchSortMatcher::sort_match(&m))
            .map(|st| st.term)
    }
}

impl SubstituteTrait<normal_forms::Expr> for Substitute {
    fn substitute(
        &self,
        term: normal_forms::Expr,
        depth: u32,
        env: &Env<Par>,
    ) -> normal_forms::Expr {
        match term {
            normal_forms::Expr::ENot(p) => normal_forms::Expr::ENot(self.substitute(p, depth, env)),

            normal_forms::Expr::ENeg(p) => Expr::ENeg(self.substitute(p, depth, env)),

            normal_forms::Expr::EMult(p1, p2) => {
                let p1 = self.substitute(p1, depth, env);
                let p2 = self.substitute(p2, depth, env);

                normal_forms::Expr::EMult(p1, p2)
            }

            normal_forms::Expr::EDiv(p1, p2) => {
                let p1 = self.substitute(p1, depth, env);
                let p2 = self.substitute(p2, depth, env);

                normal_forms::Expr::EDiv(p1, p2)
            }

            normal_forms::Expr::EMod(p1, p2) => {
                let p1 = self.substitute(p1, depth, env);
                let p2 = self.substitute(p2, depth, env);

                normal_forms::Expr::EMod(p1, p2)
            }

            normal_forms::Expr::EPercentPercent(p1, p2) => {
                let p1 = self.substitute(p1, depth, env);
                let p2 = self.substitute(p2, depth, env);

                normal_forms::Expr::EPercentPercent(p1, p2)
            }

            normal_forms::Expr::EPlus(p1, p2) => {
                let p1 = self.substitute(p1, depth, env);
                let p2 = self.substitute(p2, depth, env);

                normal_forms::Expr::EPlus(p1, p2)
            }

            normal_forms::Expr::EMinus(p1, p2) => {
                let p1 = self.substitute(p1, depth, env);
                let p2 = self.substitute(p2, depth, env);

                normal_forms::Expr::EPlus(p1, p2)
            }

            normal_forms::Expr::EPlusPlus(p1, p2) => {
                let p1 = self.substitute(p1, depth, env);
                let p2 = self.substitute(p2, depth, env);

                normal_forms::Expr::EPlusPlus(p1, p2)
            }

            normal_forms::Expr::EMinusMinus(p1, p2) => {
                let p1 = self.substitute(p1, depth, env);
                let p2 = self.substitute(p2, depth, env);

                normal_forms::Expr::EMinusMinus(p1, p2)
            }

            normal_forms::Expr::ELt(p1, p2) => {
                let p1 = self.substitute(p1, depth, env);
                let p2 = self.substitute(p2, depth, env);

                normal_forms::Expr::ELt(p1, p2)
            }

            normal_forms::Expr::ELte(p1, p2) => {
                let p1 = self.substitute(p1, depth, env);
                let p2 = self.substitute(p2, depth, env);

                normal_forms::Expr::ELte(p1, p2)
            }

            normal_forms::Expr::EGt(p1, p2) => {
                let p1 = self.substitute(p1, depth, env);
                let p2 = self.substitute(p2, depth, env);

                normal_forms::Expr::EGt(p1, p2)
            }

            normal_forms::Expr::EGte(p1, p2) => {
                let p1 = self.substitute(p1, depth, env);
                let p2 = self.substitute(p2, depth, env);

                normal_forms::Expr::EGte(p1, p2)
            }

            normal_forms::Expr::EEq(p1, p2) => {
                let p1 = self.substitute(p1, depth, env);
                let p2 = self.substitute(p2, depth, env);

                normal_forms::Expr::EEq(p1, p2)
            }

            normal_forms::Expr::ENeq(p1, p2) => {
                let p1 = self.substitute(p1, depth, env);
                let p2 = self.substitute(p2, depth, env);

                normal_forms::Expr::ENeq(p1, p2)
            }

            normal_forms::Expr::EAnd(p1, p2) => {
                let p1 = self.substitute(p1, depth, env);
                let p2 = self.substitute(p2, depth, env);

                normal_forms::Expr::EAnd(p1, p2)
            }

            normal_forms::Expr::EOr(p1, p2) => {
                let p1 = self.substitute(p1, depth, env);
                let p2 = self.substitute(p2, depth, env);

                normal_forms::Expr::EOr(p1, p2)
            }

            normal_forms::Expr::EMatches(normal_forms::EMatchesBody { target, pattern }) => {
                let target = self.substitute(target, depth, env);
                let pattern = self.substitute(pattern, depth, env);

                normal_forms::Expr::ELt(target, pattern)
            }

            normal_forms::Expr::EList(normal_forms::EListBody {
                ps,
                locally_free,
                connective_used,
                remainder,
            }) => {
                let ps = ps
                    .iter()
                    .map(|p| self.substitute(p.clone(), depth, env))
                    .collect::<Vec<Par>>();

                let locally_free = set_bits_until(locally_free.into(), env.shift).into();

                normal_forms::Expr::EList(normal_forms::EListBody {
                    ps,
                    locally_free,
                    connective_used,
                    remainder,
                })
            }

            normal_forms::Expr::ETuple(normal_forms::ETupleBody {
                ps,
                locally_free,
                connective_used,
            }) => {
                let ps = ps
                    .iter()
                    .map(|p| self.substitute(p.clone(), depth, env))
                    .collect::<Vec<Par>>();

                let new_locally_free = set_bits_until(locally_free.into(), env.shift);

                normal_forms::Expr::ETuple(normal_forms::ETupleBody {
                    ps,
                    locally_free,
                    connective_used,
                })
            }

            normal_forms::Expr::ESet(eset) => {
                let ps = eset
                    .ps
                    .into_iter()
                    .map(|p| self.substitute(p, depth, env))
                    .map(Into::into)
                    .collect();

                let ps = SortedParHashSet::create_from_vec(ps);

                let eset_body = ParSetTypeMapper::par_set_to_eset(ParSet {
                    ps,
                    connective_used: eset.connective_used,
                    locally_free: set_bits_until(eset.locally_free.into(), env.shift).into(),
                    remainder: eset.remainder.map(Into::into),
                })
                .into();

                normal_forms::Expr::ESet(eset_body)
            }

            normal_forms::Expr::EMap(emap) => {
                let par_map = ParMapTypeMapper::emap_to_par_map(emap.into());
                let ps = par_map
                    .ps
                    .sorted_list
                    .into_iter()
                    .map(|(p1, p2)| {
                        let p1: Par = p1.into();
                        let p2 = p2.into();
                        let p1 = self.substitute(p1, depth, env);
                        let p2 = self.substitute(p2, depth, env);
                        (p1, p2)
                    })
                    .collect::<Vec<(Par, Par)>>();

                let emap_body = ParMapTypeMapper::par_map_to_emap(ParMap {
                    ps: SortedParMap::create_from_vec(ps),
                    connective_used: par_map.connective_used,
                    locally_free: set_bits_until(par_map.locally_free, env.shift),
                    remainder: par_map.remainder,
                })
                .into();

                normal_forms::Expr::EMap(emap_body)
            }

            normal_forms::Expr::EMethod(normal_forms::EMethodBody {
                method_name,
                target,
                arguments,
                locally_free,
                connective_used,
            }) => {
                let sub_target = self.substitute(unwrap_option_safe(target)?, depth, env)?;
                let sub_arguments = arguments
                    .iter()
                    .map(|p| self.substitute(p.clone(), depth, env))
                    .collect::<Result<Vec<Par>, InterpreterError>>()?;

                normal_forms::Expr::EMethod(normal_forms::EMethodBody {
                    method_name,
                    target: Some(sub_target),
                    arguments: sub_arguments,
                    locally_free: set_bits_until(locally_free, env.shift),
                    connective_used,
                })
            }

            _ => term,
        }
    }

    fn substitute_no_sort(&self, term: Expr, depth: u32, env: &Env<Par>) -> Expr {
        match term {
            Expr::ENot(p) => Expr::ENot(self.substitute_no_sort(p, depth, env)),
            Expr::ENeg(p) => Expr::ENeg(self.substitute_no_sort(p, depth, env)),

            Expr::EMult(p1, p2) => {
                let p1 = self.substitute_no_sort(p1, depth, env);
                let p2 = self.substitute_no_sort(p2, depth, env);

                Expr::EMult(p1, p2)
            }

            Expr::EDiv(p1, p2) => {
                let p1 = self.substitute_no_sort(p1, depth, env);
                let p2 = self.substitute_no_sort(p2, depth, env);

                Expr::EDiv(p1, p2)
            }

            Expr::EMod(p1, p2) => {
                let p1 = self.substitute_no_sort(p1, depth, env);
                let p2 = self.substitute_no_sort(p2, depth, env);

                Expr::EMod(p1, p2)
            }

            Expr::EPercentPercent(p1, p2) => {
                let p1 = self.substitute_no_sort(p1, depth, env);
                let p2 = self.substitute_no_sort(p2, depth, env);

                Expr::EPercentPercent(p1, p2)
            }

            Expr::EPlus(p1, p2) => {
                let p1 = self.substitute_no_sort(p1, depth, env);
                let p2 = self.substitute_no_sort(p2, depth, env);

                Expr::EPlus(p1, p2)
            }

            Expr::EMinus(p1, p2) => {
                let p1 = self.substitute_no_sort(p1, depth, env);
                let p2 = self.substitute_no_sort(p2, depth, env);

                Expr::EMinus(p1, p2)
            }

            Expr::EPlusPlus(p1, p2) => {
                let p1 = self.substitute_no_sort(p1, depth, env);
                let p2 = self.substitute_no_sort(p2, depth, env);

                Expr::EPlusPlus(p1, p2)
            }

            Expr::EMinusMinus(p1, p2) => {
                let p1 = self.substitute_no_sort(p1, depth, env);
                let p2 = self.substitute_no_sort(p2, depth, env);

                Expr::EMinusMinus(p1, p2)
            }

            Expr::ELt(p1, p2) => {
                let p1 = self.substitute_no_sort(p1, depth, env);
                let p2 = self.substitute_no_sort(p2, depth, env);

                Expr::ELt(p1, p2)
            }

            Expr::ELte(p1, p2) => {
                let p1 = self.substitute_no_sort(p1, depth, env);
                let p2 = self.substitute_no_sort(p2, depth, env);

                Expr::ELte(p1, p2)
            }

            Expr::EGt(p1, p2) => {
                let p1 = self.substitute_no_sort(p1, depth, env);
                let p2 = self.substitute_no_sort(p2, depth, env);

                Expr::EGt(p1, p2)
            }

            Expr::EGte(p1, p2) => {
                let p1 = self.substitute_no_sort(p1, depth, env);
                let p2 = self.substitute_no_sort(p2, depth, env);

                Expr::EGte(p1, p2)
            }

            Expr::EEq(p1, p2) => {
                let p1 = self.substitute_no_sort(p1, depth, env);
                let p2 = self.substitute_no_sort(p2, depth, env);

                Expr::EEq(p1, p2)
            }

            Expr::ENeq(p1, p2) => {
                let p1 = self.substitute_no_sort(p1, depth, env);
                let p2 = self.substitute_no_sort(p2, depth, env);

                Expr::ENeq(p1, p2)
            }

            Expr::EAnd(p1, p2) => {
                let p1 = self.substitute_no_sort(p1, depth, env);
                let p2 = self.substitute_no_sort(p2, depth, env);

                Expr::EAnd(p1, p2)
            }

            Expr::EOr(p1, p2) => {
                let p1 = self.substitute_no_sort(p1, depth, env);
                let p2 = self.substitute_no_sort(p2, depth, env);

                Expr::EOr(p1, p2)
            }

            Expr::EMatches(normal_forms::EMatchesBody { target, pattern }) => {
                let target = self.substitute_no_sort(target, depth, env);
                let pattern = self.substitute_no_sort(pattern, depth, env);

                Expr::ELt(target, pattern)
            }

            Expr::EList(normal_forms::EListBody {
                ps,
                locally_free,
                connective_used,
                remainder,
            }) => {
                let ps = ps
                    .iter()
                    .map(|p| self.substitute_no_sort(p.clone(), depth, env))
                    .collect();

                let locally_free = set_bits_until(
                    locally_free.into_iter().map(|v| v as u8).collect(),
                    env.shift,
                );

                let locally_free = BitVec::from_iter(locally_free.into_iter().map(|v| v as usize));

                Expr::EList(normal_forms::EListBody {
                    ps,
                    locally_free,
                    connective_used,
                    remainder,
                })
            }

            Expr::ETuple(ETupleBody {
                ps,
                locally_free,
                connective_used,
            }) => {
                let ps = ps
                    .iter()
                    .map(|p| self.substitute_no_sort(p.clone(), depth, env))
                    .collect();

                let locally_free = set_bits_until(
                    locally_free.into_iter().map(|v| v as u8).collect(),
                    env.shift,
                );

                let locally_free = BitVec::from_iter(locally_free.into_iter().map(|v| v as usize));

                Expr::ETuple(ETupleBody {
                    ps,
                    locally_free,
                    connective_used,
                })
            }

            Expr::ESet(eset) => {
                let ps = eset
                    .ps
                    .into_iter()
                    .map(|p| self.substitute(p, depth, env))
                    .collect();

                let iter = set_bits_until(
                    eset.locally_free
                        .into_vec()
                        .into_iter()
                        .map(|v| v as u8)
                        .collect(),
                    env.shift,
                )
                .into_iter()
                .map(|v| v as usize)
                .collect();

                let locally_free = BitVec::from_iter::<Vec<usize>>(iter);
                let ps =
                    SortedParHashSet::create_from_vec(ps.into_iter().map(Into::into).collect())
                        .ps
                        .into_iter()
                        .map(Into::into)
                        .collect();

                Expr::ESet(ESetBody {
                    ps,
                    locally_free,
                    connective_used: eset.connective_used,
                    remainder: eset.remainder.into(),
                })
            }

            Expr::EMap(emap) => {
                let par_map = ParMapTypeMapper::emap_to_par_map(emap.into());
                let ps = par_map
                    .ps
                    .sorted_list
                    .into_iter()
                    .map(|(p1, p2)| {
                        let p1 = self.substitute(p1.into(), depth, env).into();
                        let p2 = self.substitute(p2.into(), depth, env).into();
                        (p1, p2)
                    })
                    .collect();

                let emap_body = ParMapTypeMapper::par_map_to_emap(ParMap {
                    ps: SortedParMap::create_from_vec(ps),
                    connective_used: par_map.connective_used,
                    locally_free: set_bits_until(par_map.locally_free, env.shift),
                    remainder: par_map.remainder,
                });

                Expr::EMap(emap_body.into())
            }

            Expr::EMethod(normal_forms::EMethodBody {
                method_name,
                target,
                arguments,
                locally_free,
                connective_used,
            }) => {
                let target = self.substitute_no_sort(target, depth, env);
                let arguments = arguments
                    .into_iter()
                    .map(|p| self.substitute_no_sort(p, depth, env))
                    .collect();

                let locally_free = set_bits_until(
                    locally_free
                        .into_vec()
                        .into_iter()
                        .map(|v| v as u8)
                        .collect(),
                    env.shift,
                );

                Expr::EMethod(normal_forms::EMethodBody {
                    method_name,
                    target,
                    arguments,
                    locally_free: locally_free.into(),
                    connective_used,
                })
            }
            _ => term,
        }
    }
}

fn set_bits_until(bits: Vec<u8>, until: u32) -> Vec<u8> {
    if until <= 0 {
        return Vec::new();
    }
    let until_usize = until as usize;

    bits.iter()
        .enumerate()
        .filter(|&(i, &bit)| i < until_usize && bit == 1)
        .map(|(_, &bit)| bit)
        .collect()
}

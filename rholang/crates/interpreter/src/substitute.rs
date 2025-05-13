use crate::normal_forms::{self, Connective, ESetBody, ETupleBody, Expr, MyBitVec, Par, Var};
use crate::utils::{prepend_connective, prepend_expr};
use models::rhoapi::{EVar, VarRef};
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

pub trait SubstituteTrait<A> {
    fn substitute(&self, term: A, depth: u32, env: &Env<Par>) -> Result<A, InterpreterError>;

    fn substitute_no_sort(
        &self,
        term: A,
        depth: u32,
        env: &Env<Par>,
    ) -> Result<A, InterpreterError>;
}

#[derive(Clone)]
pub struct Substitute {
    pub cost: CostManager,
}

impl Substitute {
    pub fn substitute_and_charge<T>(
        &self,
        term: &T,
        depth: u32,
        env: &Env<Par>,
    ) -> Result<T, InterpreterError>
    where
        Self: SubstituteTrait<T>,
        T: Clone + prost::Message,
    {
        match self.substitute(term.clone(), depth, env) {
            Ok(subst_term) => {
                self.cost.charge(Cost::create_from_generic(
                    &subst_term.clone(),
                    "substitution".to_string(),
                ))?;
                Ok(subst_term)
            }
            Err(th) => {
                self.cost
                    .charge(Cost::create_from_generic(&term.clone(), "".to_string()))?;
                Err(th)
            }
        }
    }

    pub fn substitute_no_sort_and_charge<A>(
        &self,
        term: &A,
        depth: u32,
        env: &Env<Par>,
    ) -> Result<A, InterpreterError>
    where
        Self: SubstituteTrait<A>,
        A: Clone + prost::Message,
    {
        // scala 'charge' function built in here
        match self.substitute_no_sort(term.clone(), depth, env) {
            Ok(subst_term) => {
                self.cost.charge(Cost::create_from_generic(
                    &subst_term.clone(),
                    "substitution".to_string(),
                ))?;
                Ok(subst_term)
            }
            Err(th) => {
                self.cost
                    .charge(Cost::create_from_generic(&term.clone(), "".to_string()))?;
                Err(th)
            }
        }
    }

    // pub here for testing purposes
    pub fn maybe_substitute_var(
        &self,
        term: Var,
        depth: u32,
        env: &Env<Par>,
    ) -> Result<Either<Var, Par>, InterpreterError> {
        if depth != 0 {
            Ok(Either::Left(term))
        } else {
            match term.clone() {
                normal_forms::Var::BoundVar(index) => match env.get(index) {
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
        term: normal_forms::Var,
        depth: u32,
        env: &Env<Par>,
    ) -> Result<Either<EVar, Par>, InterpreterError> {
        match self.maybe_substitute_var(term.into(), depth, env)? {
            Either::Left(v) => Ok(Either::Left(EVar { v: Some(v.into()) })),
            Either::Right(p) => Ok(Either::Right(p)),
        }
    }

    fn maybe_substitute_var_ref(
        &self,
        term: normal_forms::VarRef,
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

    fn sub_exp(
        &self,
        exprs: Vec<Expr>,
        depth: u32,
        env: &Env<Par>,
    ) -> Result<Par, InterpreterError> {
        exprs
            .into_iter()
            .try_fold(Par::default(), |par, expr| match &expr {
                &Expr::EVar(var) => match self.maybe_substitute_evar(var, depth, env)? {
                    Either::Left(_) => Ok(prepend_expr(par.into(), expr, depth)),
                    Either::Right(right_par) => {
                        Ok(concatenate_pars(right_par.into(), par.into()).into())
                    }
                },
                _ => self
                    .substitute_no_sort(expr.into(), depth, env)
                    .map(|e| prepend_expr(par, e, depth)),
            })
    }

    fn sub_conn(
        &self,
        conns: Vec<Connective>,
        depth: u32,
        env: &Env<Par>,
    ) -> Result<Par, InterpreterError> {
        conns
            .into_iter()
            .try_fold(Default::default(), |par, conn| match conn {
                Connective::VarRef(v) => {
                    match self.maybe_substitute_var_ref(v.clone(), depth, env) {
                        Either::Left(_) => Ok(prepend_connective(par, conn, depth)),
                        Either::Right(new_par) => {
                            Ok(concatenate_pars(new_par.into(), par.into()).into())
                        }
                    }
                }

                Connective::ConnAnd(ps) => {
                    let _ps: Vec<Par> = ps
                        .iter()
                        .map(|p| self.substitute_no_sort(p.clone(), depth, env))
                        .collect::<Result<Vec<Par>, InterpreterError>>()?;

                    Ok(prepend_connective(par, Connective::ConnAnd(ps), depth))
                }

                Connective::ConnOr(ps) => {
                    let _ps: Vec<Par> = ps
                        .iter()
                        .map(|p| self.substitute_no_sort(p.clone(), depth, env))
                        .collect::<Result<Vec<Par>, InterpreterError>>()?;

                    Ok(prepend_connective(
                        par,
                        Connective::ConnOr(ps.to_vec()),
                        depth,
                    ))
                }

                Connective::ConnNot(p) => self
                    .substitute_no_sort(p.clone(), depth, env)
                    .map(|p| prepend_connective(par, Connective::ConnNot(p), depth)),

                Connective::ConnBool(c) => {
                    Ok(prepend_connective(par, Connective::ConnBool(c), depth))
                }
                Connective::ConnInt(c) => {
                    Ok(prepend_connective(par, Connective::ConnInt(c), depth))
                }
                Connective::ConnString(c) => {
                    Ok(prepend_connective(par, Connective::ConnString(c), depth))
                }
                Connective::ConnUri(c) => {
                    Ok(prepend_connective(par, Connective::ConnUri(c), depth))
                }
                Connective::ConnByteArray(c) => {
                    Ok(prepend_connective(par, Connective::ConnByteArray(c), depth))
                }
            })
    }
}

impl SubstituteTrait<Par> for Substitute {
    fn substitute_no_sort(
        &self,
        term: Par,
        depth: u32,
        env: &Env<Par>,
    ) -> Result<Par, InterpreterError> {
        let exprs = self.sub_exp(term.exprs, depth, env)?;
        let connectives = self.sub_conn(term.connectives, depth, env)?;

        let sends = term
            .sends
            .into_iter()
            .map(|s| self.substitute_no_sort(s, depth, env))
            .collect::<Result<Vec<_>, InterpreterError>>()?;

        let bundles = term
            .bundles
            .iter()
            .map(|b| self.substitute_no_sort(b.clone(), depth, env))
            .collect::<Result<Vec<_>, InterpreterError>>()?;

        let receives = term
            .receives
            .iter()
            .map(|r| self.substitute_no_sort(r.clone(), depth, env))
            .collect::<Result<Vec<_>, InterpreterError>>()?;

        let news = term
            .news
            .iter()
            .map(|n| self.substitute_no_sort(n.clone(), depth, env))
            .collect::<Result<Vec<_>, InterpreterError>>()?;

        let matches = term
            .matches
            .iter()
            .map(|m| self.substitute_no_sort(m.clone(), depth, env))
            .collect::<Result<Vec<_>, InterpreterError>>()?;

        let right = Par {
            sends,
            receives,
            news,
            exprs: Vec::new(),
            matches,
            unforgeables: term.unforgeables,
            bundles,
            connectives: Vec::new(),
            locally_free: set_bits_until(term.locally_free.into(), env.shift).into(),
            connective_used: term.connective_used,
        };
        let right = concatenate_pars(connectives.into(), right.into());
        let result = concatenate_pars(exprs.into(), right);

        Ok(result.into())
    }

    fn substitute(&self, term: Par, depth: u32, env: &Env<Par>) -> Result<Par, InterpreterError> {
        let v = self.substitute_no_sort(term, depth, env)?;
        let v = ParSortMatcher::sort_match(&(v.into()));
        Ok(v.term.into())
    }
}

impl SubstituteTrait<normal_forms::Bundle> for Substitute {
    fn substitute(
        &self,
        term: normal_forms::Bundle,
        depth: u32,
        env: &Env<Par>,
    ) -> Result<normal_forms::Bundle, InterpreterError> {
        let term_body = term.body.clone();
        let sub_bundle = self.substitute(term_body, depth, env)?;
        let term_clone = term.clone();

        let result = match single_bundle(&sub_bundle.clone().into()) {
            None => {
                let mut term_mut = term_clone;
                term_mut.body = sub_bundle.into();
                term_mut.into()
            }
            Some(b) => BundleOps::merge(&term.into(), &b).into(),
        };

        Ok(result)
    }

    fn substitute_no_sort(
        &self,
        term: normal_forms::Bundle,
        depth: u32,
        env: &Env<Par>,
    ) -> Result<normal_forms::Bundle, InterpreterError> {
        let sub_bundle = self.substitute_no_sort(term.clone().body, depth, env)?;
        let term_clone = term.clone();

        let result = match single_bundle(&sub_bundle.clone().into()) {
            Some(b) => BundleOps::merge(&term.into(), &b).into(),
            None => {
                let mut term_mut = term_clone;
                term_mut.body = sub_bundle.into();
                term_mut.into()
            }
        };

        Ok(result)
    }
}

impl SubstituteTrait<normal_forms::Send> for Substitute {
    fn substitute_no_sort(
        &self,
        term: normal_forms::Send,
        depth: u32,
        env: &Env<Par>,
    ) -> Result<normal_forms::Send, InterpreterError> {
        let channels_sub = self
            .substitute_no_sort(term.clone().chan, depth, env)?
            .into();
        let locally_free = set_bits_until(term.locally_free.into(), env.shift);

        term.data
            .into_iter()
            .map(|p| self.substitute_no_sort(p.clone(), depth, env))
            .collect::<Result<Vec<_>, InterpreterError>>()
            .map(|data| normal_forms::Send {
                chan: channels_sub,
                data,
                persistent: term.persistent,
                locally_free: locally_free.into(),
                connective_used: term.connective_used,
            })
    }

    fn substitute(
        &self,
        term: normal_forms::Send,
        depth: u32,
        env: &Env<Par>,
    ) -> Result<normal_forms::Send, InterpreterError> {
        self.substitute_no_sort(term, depth, env)
            .map(|s| SendSortMatcher::sort_match(&(s.into())))
            .map(|st| st.term.into())
    }
}

impl SubstituteTrait<normal_forms::Receive> for Substitute {
    fn substitute_no_sort(
        &self,
        term: normal_forms::Receive,
        depth: u32,
        env: &Env<Par>,
    ) -> Result<normal_forms::Receive, InterpreterError> {
        let binds_sub = term
            .binds
            .into_iter()
            .map(
                |normal_forms::ReceiveBind {
                     patterns,
                     source,
                     remainder,
                     free_count,
                 }| {
                    let source = self.substitute_no_sort(source, depth, env)?;
                    let patterns = patterns
                        .into_iter()
                        .map(|p| self.substitute_no_sort(p, depth + 1, env))
                        .collect::<Result<Vec<Par>, InterpreterError>>()?;

                    Ok(normal_forms::ReceiveBind {
                        patterns,
                        source,
                        remainder: remainder.map(Into::into),
                        free_count: free_count as u32,
                    })
                },
            )
            .collect::<Result<Vec<normal_forms::ReceiveBind>, InterpreterError>>()?;

        let body_sub = self.substitute_no_sort(term.body, depth, &env.shift(term.bind_count))?;

        Ok(normal_forms::Receive {
            binds: binds_sub,
            body: body_sub,
            persistent: term.persistent,
            peek: term.peek,
            bind_count: term.bind_count,
            locally_free: set_bits_until(term.locally_free.into(), env.shift).into(),
            connective_used: term.connective_used,
        })
    }

    fn substitute(
        &self,
        term: normal_forms::Receive,
        depth: u32,
        env: &Env<Par>,
    ) -> Result<normal_forms::Receive, InterpreterError> {
        self.substitute_no_sort(term, depth, env)
            .map(|r| ReceiveSortMatcher::sort_match(&(r.into())))
            .map(|st| st.term.into())
    }
}

impl SubstituteTrait<normal_forms::New> for Substitute {
    fn substitute_no_sort(
        &self,
        term: normal_forms::New,
        depth: u32,
        env: &Env<Par>,
    ) -> Result<normal_forms::New, InterpreterError> {
        let env = &env.shift(term.bind_count);
        let par = self.substitute_no_sort(term.p, depth, env)?;
        let locally_free = set_bits_until(term.locally_free.into(), env.shift.into()).into();

        Ok(normal_forms::New {
            bind_count: term.bind_count,
            p: par,
            uris: term.uris,
            injections: term.injections,
            locally_free,
        })
    }

    fn substitute(
        &self,
        term: normal_forms::New,
        depth: u32,
        env: &Env<Par>,
    ) -> Result<normal_forms::New, InterpreterError> {
        let v = self.substitute_no_sort(term, depth, env)?;
        let v = NewSortMatcher::sort_match(&v.into());
        Ok(v.term.into())
    }
}

impl SubstituteTrait<normal_forms::Match> for Substitute {
    fn substitute_no_sort(
        &self,
        term: normal_forms::Match,
        depth: u32,
        env: &Env<Par>,
    ) -> Result<normal_forms::Match, InterpreterError> {
        let target_sub = self.substitute_no_sort(term.target, depth, env)?;

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
                        self.substitute_no_sort(source.clone(), depth, &env.shift(*free_count))?;

                    let sub_case = self.substitute_no_sort(pattern.clone(), depth + 1, env)?;

                    Ok(normal_forms::MatchCase {
                        pattern: sub_case,
                        source: par,
                        free_count: *free_count,
                    })
                },
            )
            .collect::<Result<Vec<normal_forms::MatchCase>, InterpreterError>>()?;

        let locally_free = set_bits_until(term.locally_free.into(), env.shift).into();

        Ok(normal_forms::Match {
            target: target_sub,
            cases,
            locally_free,
            ..term
        })
    }

    fn substitute(
        &self,
        term: normal_forms::Match,
        depth: u32,
        env: &Env<Par>,
    ) -> Result<normal_forms::Match, InterpreterError> {
        let value = self.substitute_no_sort(term, depth, env)?;
        let value = value.into();
        let value = MatchSortMatcher::sort_match(&value);
        Ok(value.term.into())
    }
}

impl SubstituteTrait<Expr> for Substitute {
    fn substitute(&self, term: Expr, depth: u32, env: &Env<Par>) -> Result<Expr, InterpreterError> {
        let result = match term {
            Expr::ENot(p) => Expr::ENot(self.substitute(p, depth, env)?),
            Expr::ENeg(p) => Expr::ENeg(self.substitute(p, depth, env)?),

            Expr::EMult(p1, p2) => {
                let p1 = self.substitute(p1, depth, env)?;
                let p2 = self.substitute(p2, depth, env)?;

                Expr::EMult(p1, p2)
            }

            Expr::EDiv(p1, p2) => {
                let p1 = self.substitute(p1, depth, env)?;
                let p2 = self.substitute(p2, depth, env)?;

                Expr::EDiv(p1, p2)
            }

            Expr::EMod(p1, p2) => {
                let p1 = self.substitute(p1, depth, env)?;
                let p2 = self.substitute(p2, depth, env)?;

                Expr::EMod(p1, p2)
            }

            Expr::EPercentPercent(p1, p2) => {
                let p1 = self.substitute(p1, depth, env)?;
                let p2 = self.substitute(p2, depth, env)?;

                Expr::EPercentPercent(p1, p2)
            }

            Expr::EPlus(p1, p2) => {
                let p1 = self.substitute(p1, depth, env)?;
                let p2 = self.substitute(p2, depth, env)?;

                Expr::EPlus(p1, p2)
            }

            Expr::EMinus(p1, p2) => {
                let p1 = self.substitute(p1, depth, env)?;
                let p2 = self.substitute(p2, depth, env)?;

                Expr::EPlus(p1, p2)
            }

            Expr::EPlusPlus(p1, p2) => {
                let p1 = self.substitute(p1, depth, env)?;
                let p2 = self.substitute(p2, depth, env)?;

                Expr::EPlusPlus(p1, p2)
            }

            Expr::EMinusMinus(p1, p2) => {
                let p1 = self.substitute(p1, depth, env)?;
                let p2 = self.substitute(p2, depth, env)?;

                Expr::EMinusMinus(p1, p2)
            }

            Expr::ELt(p1, p2) => {
                let p1 = self.substitute(p1, depth, env)?;
                let p2 = self.substitute(p2, depth, env)?;

                Expr::ELt(p1, p2)
            }

            Expr::ELte(p1, p2) => {
                let p1 = self.substitute(p1, depth, env)?;
                let p2 = self.substitute(p2, depth, env)?;

                Expr::ELte(p1, p2)
            }

            Expr::EGt(p1, p2) => {
                let p1 = self.substitute(p1, depth, env)?;
                let p2 = self.substitute(p2, depth, env)?;

                Expr::EGt(p1, p2)
            }

            Expr::EGte(p1, p2) => {
                let p1 = self.substitute(p1, depth, env)?;
                let p2 = self.substitute(p2, depth, env)?;

                Expr::EGte(p1, p2)
            }

            Expr::EEq(p1, p2) => {
                let p1 = self.substitute(p1, depth, env)?;
                let p2 = self.substitute(p2, depth, env)?;

                Expr::EEq(p1, p2)
            }

            Expr::ENeq(p1, p2) => {
                let p1 = self.substitute(p1, depth, env)?;
                let p2 = self.substitute(p2, depth, env)?;

                Expr::ENeq(p1, p2)
            }

            Expr::EAnd(p1, p2) => {
                let p1 = self.substitute(p1, depth, env)?;
                let p2 = self.substitute(p2, depth, env)?;

                Expr::EAnd(p1, p2)
            }

            Expr::EOr(p1, p2) => {
                let p1 = self.substitute(p1, depth, env)?;
                let p2 = self.substitute(p2, depth, env)?;

                Expr::EOr(p1, p2)
            }

            Expr::EMatches(normal_forms::EMatchesBody { target, pattern }) => {
                let target = self.substitute(target, depth, env)?;
                let pattern = self.substitute(pattern, depth, env)?;

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
                    .map(|p| self.substitute(p.clone(), depth, env))
                    .collect::<Result<Vec<Par>, _>>()?;

                let locally_free = set_bits_until(locally_free.into(), env.shift).into();

                Expr::EList(normal_forms::EListBody {
                    ps,
                    locally_free,
                    connective_used,
                    remainder,
                })
            }

            Expr::ETuple(normal_forms::ETupleBody {
                ps,
                locally_free,
                connective_used,
            }) => {
                let ps = ps
                    .iter()
                    .map(|p| self.substitute(p.clone(), depth, env))
                    .collect::<Result<Vec<Par>, _>>()?;

                let locally_free = set_bits_until(locally_free.into(), env.shift).into();

                Expr::ETuple(normal_forms::ETupleBody {
                    ps,
                    locally_free,
                    connective_used,
                })
            }

            Expr::ESet(eset) => {
                let par_set = ParSetTypeMapper::eset_to_par_set(eset.into());
                let _ps = par_set
                    .ps
                    .sorted_pars
                    .iter()
                    .map(|p| self.substitute(p.clone().into(), depth, env))
                    .collect::<Result<Vec<Par>, InterpreterError>>()?;

                let par_set_to_eset = ParSetTypeMapper::par_set_to_eset(ParSet {
                    ps: SortedParHashSet::create_from_vec(
                        _ps.into_iter().map(Into::into).collect(),
                    ),
                    connective_used: par_set.connective_used,
                    locally_free: set_bits_until(par_set.locally_free, env.shift),
                    remainder: par_set.remainder,
                });

                Expr::ESet(par_set_to_eset.into())
            }

            Expr::EMap(emap) => {
                let par_map = ParMapTypeMapper::emap_to_par_map(emap.into());
                let ps = par_map
                    .ps
                    .sorted_list
                    .into_iter()
                    .map(|(p1, p2)| {
                        let p1 = self.substitute(p1.into(), depth, env)?;
                        let p2 = self.substitute(p2.into(), depth, env)?;
                        Ok((p1, p2))
                    })
                    .collect::<Result<Vec<(Par, Par)>, InterpreterError>>()?;

                let emap_body = ParMapTypeMapper::par_map_to_emap(ParMap {
                    ps: SortedParMap::create_from_vec(
                        ps.into_iter()
                            .map(|(p1, p2)| (p1.into(), p2.into()))
                            .collect(),
                    ),
                    connective_used: par_map.connective_used,
                    locally_free: set_bits_until(par_map.locally_free, env.shift),
                    remainder: par_map.remainder,
                })
                .into();

                Expr::EMap(emap_body)
            }

            Expr::EMethod(normal_forms::EMethodBody {
                method_name,
                target,
                arguments,
                locally_free,
                connective_used,
            }) => {
                let sub_target = self.substitute(target, depth, env)?;
                let sub_arguments = arguments
                    .iter()
                    .map(|p| self.substitute(p.clone().into(), depth, env))
                    .collect::<Result<Vec<Par>, InterpreterError>>()?;

                Expr::EMethod(normal_forms::EMethodBody {
                    method_name,
                    target: sub_target,
                    arguments: sub_arguments,
                    locally_free: set_bits_until(locally_free.into(), env.shift).into(),
                    connective_used,
                })
            }

            _ => term,
        };

        Ok(result)
    }

    fn substitute_no_sort(
        &self,
        term: Expr,
        depth: u32,
        env: &Env<Par>,
    ) -> Result<Expr, InterpreterError> {
        let result = match term {
            Expr::ENot(p) => Expr::ENot(self.substitute_no_sort(p, depth, env)?),
            Expr::ENeg(p) => Expr::ENeg(self.substitute_no_sort(p, depth, env)?),

            Expr::EMult(p1, p2) => {
                let p1 = self.substitute_no_sort(p1, depth, env)?;
                let p2 = self.substitute_no_sort(p2, depth, env)?;

                Expr::EMult(p1, p2)
            }

            Expr::EDiv(p1, p2) => {
                let p1 = self.substitute_no_sort(p1, depth, env)?;
                let p2 = self.substitute_no_sort(p2, depth, env)?;

                Expr::EDiv(p1, p2)
            }

            Expr::EMod(p1, p2) => {
                let p1 = self.substitute_no_sort(p1, depth, env)?;
                let p2 = self.substitute_no_sort(p2, depth, env)?;

                Expr::EMod(p1, p2)
            }

            Expr::EPercentPercent(p1, p2) => {
                let p1 = self.substitute_no_sort(p1, depth, env)?;
                let p2 = self.substitute_no_sort(p2, depth, env)?;

                Expr::EPercentPercent(p1, p2)
            }

            Expr::EPlus(p1, p2) => {
                let p1 = self.substitute_no_sort(p1, depth, env)?;
                let p2 = self.substitute_no_sort(p2, depth, env)?;

                Expr::EPlus(p1, p2)
            }

            Expr::EMinus(p1, p2) => {
                let p1 = self.substitute_no_sort(p1, depth, env)?;
                let p2 = self.substitute_no_sort(p2, depth, env)?;

                Expr::EMinus(p1, p2)
            }

            Expr::EPlusPlus(p1, p2) => {
                let p1 = self.substitute_no_sort(p1, depth, env)?;
                let p2 = self.substitute_no_sort(p2, depth, env)?;

                Expr::EPlusPlus(p1, p2)
            }

            Expr::EMinusMinus(p1, p2) => {
                let p1 = self.substitute_no_sort(p1, depth, env)?;
                let p2 = self.substitute_no_sort(p2, depth, env)?;

                Expr::EMinusMinus(p1, p2)
            }

            Expr::ELt(p1, p2) => {
                let p1 = self.substitute_no_sort(p1, depth, env)?;
                let p2 = self.substitute_no_sort(p2, depth, env)?;

                Expr::ELt(p1, p2)
            }

            Expr::ELte(p1, p2) => {
                let p1 = self.substitute_no_sort(p1, depth, env)?;
                let p2 = self.substitute_no_sort(p2, depth, env)?;

                Expr::ELte(p1, p2)
            }

            Expr::EGt(p1, p2) => {
                let p1 = self.substitute_no_sort(p1, depth, env)?;
                let p2 = self.substitute_no_sort(p2, depth, env)?;

                Expr::EGt(p1, p2)
            }

            Expr::EGte(p1, p2) => {
                let p1 = self.substitute_no_sort(p1, depth, env)?;
                let p2 = self.substitute_no_sort(p2, depth, env)?;

                Expr::EGte(p1, p2)
            }

            Expr::EEq(p1, p2) => {
                let p1 = self.substitute_no_sort(p1, depth, env)?;
                let p2 = self.substitute_no_sort(p2, depth, env)?;

                Expr::EEq(p1, p2)
            }

            Expr::ENeq(p1, p2) => {
                let p1 = self.substitute_no_sort(p1, depth, env)?;
                let p2 = self.substitute_no_sort(p2, depth, env)?;

                Expr::ENeq(p1, p2)
            }

            Expr::EAnd(p1, p2) => {
                let p1 = self.substitute_no_sort(p1, depth, env)?;
                let p2 = self.substitute_no_sort(p2, depth, env)?;

                Expr::EAnd(p1, p2)
            }

            Expr::EOr(p1, p2) => {
                let p1 = self.substitute_no_sort(p1, depth, env)?;
                let p2 = self.substitute_no_sort(p2, depth, env)?;

                Expr::EOr(p1, p2)
            }

            Expr::EMatches(normal_forms::EMatchesBody { target, pattern }) => {
                let target = self.substitute_no_sort(target, depth, env)?;
                let pattern = self.substitute_no_sort(pattern, depth, env)?;

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
                    .map(|p| self.substitute_no_sort(p.clone().into(), depth, env))
                    .collect::<Result<Vec<_>, _>>()?;

                let locally_free = set_bits_until(locally_free.into(), env.shift);

                let locally_free =
                    MyBitVec::from_iter(locally_free.into_iter().map(|v| v as usize));

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
                    .collect::<Result<Vec<_>, _>>()?;

                let locally_free = set_bits_until(locally_free.into(), env.shift);

                let locally_free =
                    MyBitVec::from_iter(locally_free.into_iter().map(|v| v as usize));

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
                    .collect::<Result<Vec<_>, _>>()?;

                let iter = set_bits_until(eset.locally_free.into(), env.shift)
                    .into_iter()
                    .map(|v| v as usize)
                    .collect();

                let locally_free = MyBitVec::from_iter::<Vec<usize>>(iter);
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
                        let p1: Par = p1.into();
                        let p2: Par = p2.into();
                        let p1: models::rhoapi::Par =
                            self.substitute(p1, depth, env).map(Into::into)?;
                        let p2: models::rhoapi::Par =
                            self.substitute(p2, depth, env).map(Into::into)?;

                        Ok((p1, p2))
                    })
                    .collect::<Result<Vec<(models::rhoapi::Par, models::rhoapi::Par)>, InterpreterError>>()?;

                let ps = SortedParMap::create_from_vec(ps);

                let emap_body = ParMapTypeMapper::par_map_to_emap(ParMap {
                    ps,
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
                let target = self.substitute_no_sort(target, depth, env)?;
                let arguments = arguments
                    .into_iter()
                    .map(|p| self.substitute_no_sort(p, depth, env))
                    .collect::<Result<Vec<_>, _>>()?;

                let locally_free = set_bits_until(locally_free.into(), env.shift);

                Expr::EMethod(normal_forms::EMethodBody {
                    method_name,
                    target,
                    arguments,
                    locally_free: locally_free.into(),
                    connective_used,
                })
            }
            _ => term,
        };

        Ok(result)
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

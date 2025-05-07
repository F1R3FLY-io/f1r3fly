use crate::normal_forms::{self, EListBody, EMethodBody, ESetBody, ETupleBody, Expr, Par};
use crate::utils::{prepend_connective, prepend_expr};
use bitvec::vec::BitVec;
use models::rhoapi::connective::ConnectiveInstance;
use models::rhoapi::expr::ExprInstance::{self, EMatchesBody};
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
pub trait SubstituteTrait<A> {
    fn substitute(&self, term: A, depth: u32, env: &Env<Par>) -> A;

    fn substitute_no_sort(&self, term: A, depth: u32, env: &Env<Par>) -> A;
}

#[derive(Clone)]
pub struct Substitute {
    pub cost: CostManager,
}

impl Substitute {
    pub fn substitute_and_charge<A>(
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
        match self.substitute(term.clone(), depth, env) {
            Ok(subst_term) => {
                self.cost.charge(Cost::create_from_generic(
                    subst_term.clone(),
                    "substitution".to_string(),
                ))?;
                Ok(subst_term)
            }
            Err(th) => {
                self.cost
                    .charge(Cost::create_from_generic(term.clone(), "".to_string()))?;
                Err(th)
            }
        }
    }

    pub fn substitute_no_sort_and_charge<A>(&self, term: &A, depth: u32, env: &Env<Par>) -> A
    where
        Self: SubstituteTrait<A>,
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
        term: Var,
        depth: u32,
        env: &Env<Par>,
    ) -> Result<Either<Var, Par>, InterpreterError> {
        if depth != 0 {
            Ok(Either::Left(term))
        } else {
            let v = term
                .clone()
                .var_instance
                .ok_or(InterpreterError::UndefinedRequiredProtobufFieldError)?;
            match v {
                VarInstance::BoundVar(index) => match env.get(index as u32) {
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

impl SubstituteTrait<crate::normal_forms::Bundle> for Substitute {
    fn substitute(
        &self,
        term: crate::normal_forms::Bundle,
        depth: u32,
        env: &Env<Par>,
    ) -> Result<crate::normal_forms::Bundle, InterpreterError> {
        let sub_bundle = self.substitute(term.body.clone(), depth, env)?.into();
        let term_clone = term.clone();

        let result = match single_bundle(&sub_bundle) {
            None => {
                let mut term_mut = term_clone;
                term_mut.body = sub_bundle.into();
                term_mut
            }
            Some(b) => BundleOps::merge(&term.into(), &b).into(),
        };

        Ok(result)
    }

    fn substitute_no_sort(
        &self,
        term: crate::normal_forms::Bundle,
        depth: u32,
        env: &Env<Par>,
    ) -> crate::normal_forms::Bundle {
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
            .iter()
            .map(|s| self.substitute_no_sort(s.clone(), depth, env))
            .collect::<Vec<Send>, InterpreterError>();

        let bundles = term
            .bundles
            .iter()
            .map(|b| self.substitute_no_sort(b.clone(), depth, env))
            .collect::<Vec<Bundle>, InterpreterError>();

        let receives = term
            .receives
            .iter()
            .map(|r| self.substitute_no_sort(r.clone(), depth, env))
            .collect::<Vec<Receive>, InterpreterError>();

        let news = term
            .news
            .iter()
            .map(|n| self.substitute_no_sort(n.clone(), depth, env))
            .collect::<Vec<New>, InterpreterError>();

        let matches = term
            .matches
            .iter()
            .map(|m| self.substitute_no_sort(m.clone(), depth, env))
            .collect::<Vec<Match>, InterpreterError>();

        Ok(concatenate_pars(
            exprs,
            concatenate_pars(
                connectives,
                Par {
                    sends,
                    receives,
                    news,
                    exprs: Vec::new(),
                    matches,
                    unforgeables: term.unforgeables,
                    bundles,
                    connectives: Vec::new(),
                    locally_free: {
                        // println!("\nenv.shift in substitute_no_sort for par: {}", env.shift);
                        set_bits_until(term.locally_free, env.shift)
                    },
                    connective_used: term.connective_used,
                },
            ),
        ))
    }

    fn substitute(&self, term: Par, depth: u32, env: &Env<Par>) -> Result<Par, InterpreterError> {
        self.substitute_no_sort(term, depth, env)
            .map(|p| ParSortMatcher::sort_match(&p))
            .map(|st| st.term)
    }
}

impl SubstituteTrait<Send> for Substitute {
    fn substitute_no_sort(
        &self,
        term: Send,
        depth: u32,
        env: &Env<Par>,
    ) -> Result<Send, InterpreterError> {
        let channels_sub =
            self.substitute_no_sort(unwrap_option_safe(term.clone().chan)?, depth, env)?;

        let pars_sub = term
            .data
            .iter()
            .map(|p| self.substitute_no_sort(p.clone(), depth, env))
            .collect::<Result<Vec<Par>, InterpreterError>>()?;

        // println!("\nterm in substitute_no_sort for Send {:?}", term);

        Ok(Send {
            chan: Some(channels_sub),
            data: pars_sub,
            persistent: term.persistent,
            locally_free: set_bits_until(term.locally_free, env.shift),
            connective_used: term.connective_used,
        })
    }

    fn substitute(&self, term: Send, depth: i32, env: &Env<Par>) -> Result<Send, InterpreterError> {
        self.substitute_no_sort(term, depth, env)
            .map(|s| SendSortMatcher::sort_match(&s))
            .map(|st| st.term)
    }
}

impl SubstituteTrait<Receive> for Substitute {
    fn substitute_no_sort(
        &self,
        term: Receive,
        depth: u32,
        env: &Env<Par>,
    ) -> Result<Receive, InterpreterError> {
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

        Ok(Receive {
            binds: binds_sub,
            body: Some(body_sub),
            persistent: term.persistent,
            peek: term.peek,
            bind_count: term.bind_count,
            locally_free: set_bits_until(term.locally_free, env.shift),
            connective_used: term.connective_used,
        })
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

impl SubstituteTrait<New> for Substitute {
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

impl SubstituteTrait<Match> for Substitute {
    fn substitute_no_sort(
        &self,
        term: Match,
        depth: u32,
        env: &Env<Par>,
    ) -> Result<Match, InterpreterError> {
        let target_sub = self.substitute_no_sort(unwrap_option_safe(term.target)?, depth, env)?;

        let cases_sub = term
            .cases
            .iter()
            .map(
                |MatchCase {
                     pattern,
                     source,
                     free_count,
                 }| {
                    let par = self.substitute_no_sort(
                        unwrap_option_safe(source.clone())?,
                        depth,
                        &env.shift(*free_count),
                    )?;

                    let sub_case = self.substitute_no_sort(
                        unwrap_option_safe(pattern.clone())?,
                        depth + 1,
                        env,
                    )?;

                    Ok(MatchCase {
                        pattern: Some(sub_case),
                        source: Some(par),
                        free_count: *free_count,
                    })
                },
            )
            .collect::<Result<Vec<MatchCase>, InterpreterError>>()?;

        Ok(Match {
            target: Some(target_sub),
            cases: cases_sub,
            locally_free: set_bits_until(term.locally_free, env.shift),
            connective_used: term.connective_used,
        })
    }

    fn substitute(
        &self,
        term: Match,
        depth: u32,
        env: &Env<Par>,
    ) -> Result<Match, InterpreterError> {
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
        match unwrap_option_safe(term.expr_instance.clone())? {
            ExprInstance::ENotBody(ENot { p }) => self
                .substitute(unwrap_option_safe(p)?, depth, env)
                .map(|p| {
                    Ok(Expr {
                        expr_instance: Some(ExprInstance::ENotBody(ENot { p: Some(p) })),
                    })
                })?,

            ExprInstance::ENegBody(ENeg { p }) => self
                .substitute(unwrap_option_safe(p)?, depth, env)
                .map(|p| {
                    Ok(Expr {
                        expr_instance: Some(ExprInstance::ENegBody(ENeg { p: Some(p) })),
                    })
                })?,

            ExprInstance::EMultBody(EMult { p1, p2 }) => {
                let _p1 = self.substitute(unwrap_option_safe(p1)?, depth, env)?;
                let _p2 = self.substitute(unwrap_option_safe(p2)?, depth, env)?;

                Ok(Expr {
                    expr_instance: Some(ExprInstance::EMultBody(EMult {
                        p1: Some(_p1),
                        p2: Some(_p2),
                    })),
                })
            }

            ExprInstance::EDivBody(EDiv { p1, p2 }) => {
                let _p1 = self.substitute(unwrap_option_safe(p1)?, depth, env)?;
                let _p2 = self.substitute(unwrap_option_safe(p2)?, depth, env)?;

                Ok(Expr {
                    expr_instance: Some(ExprInstance::EDivBody(EDiv {
                        p1: Some(_p1),
                        p2: Some(_p2),
                    })),
                })
            }

            ExprInstance::EModBody(EMod { p1, p2 }) => {
                let _p1 = self.substitute(unwrap_option_safe(p1)?, depth, env)?;
                let _p2 = self.substitute(unwrap_option_safe(p2)?, depth, env)?;

                Ok(Expr {
                    expr_instance: Some(ExprInstance::EModBody(EMod {
                        p1: Some(_p1),
                        p2: Some(_p2),
                    })),
                })
            }

            ExprInstance::EPercentPercentBody(EPercentPercent { p1, p2 }) => {
                let _p1 = self.substitute(unwrap_option_safe(p1)?, depth, env)?;
                let _p2 = self.substitute(unwrap_option_safe(p2)?, depth, env)?;

                Ok(Expr {
                    expr_instance: Some(ExprInstance::EPercentPercentBody(EPercentPercent {
                        p1: Some(_p1),
                        p2: Some(_p2),
                    })),
                })
            }

            ExprInstance::EPlusBody(EPlus { p1, p2 }) => {
                let _p1 = self.substitute(unwrap_option_safe(p1)?, depth, env)?;
                let _p2 = self.substitute(unwrap_option_safe(p2)?, depth, env)?;

                Ok(Expr {
                    expr_instance: Some(ExprInstance::EPlusBody(EPlus {
                        p1: Some(_p1),
                        p2: Some(_p2),
                    })),
                })
            }

            ExprInstance::EMinusBody(EMinus { p1, p2 }) => {
                let _p1 = self.substitute(unwrap_option_safe(p1)?, depth, env)?;
                let _p2 = self.substitute(unwrap_option_safe(p2)?, depth, env)?;

                Ok(Expr {
                    expr_instance: Some(ExprInstance::EPlusBody(EPlus {
                        p1: Some(_p1),
                        p2: Some(_p2),
                    })),
                })
            }

            ExprInstance::EPlusPlusBody(EPlusPlus { p1, p2 }) => {
                let _p1 = self.substitute(unwrap_option_safe(p1)?, depth, env)?;
                let _p2 = self.substitute(unwrap_option_safe(p2)?, depth, env)?;

                Ok(Expr {
                    expr_instance: Some(ExprInstance::EPlusPlusBody(EPlusPlus {
                        p1: Some(_p1),
                        p2: Some(_p2),
                    })),
                })
            }

            ExprInstance::EMinusMinusBody(EMinusMinus { p1, p2 }) => {
                let _p1 = self.substitute(unwrap_option_safe(p1)?, depth, env)?;
                let _p2 = self.substitute(unwrap_option_safe(p2)?, depth, env)?;

                Ok(Expr {
                    expr_instance: Some(ExprInstance::EMinusMinusBody(EMinusMinus {
                        p1: Some(_p1),
                        p2: Some(_p2),
                    })),
                })
            }

            ExprInstance::ELtBody(ELt { p1, p2 }) => {
                let _p1 = self.substitute(unwrap_option_safe(p1)?, depth, env)?;
                let _p2 = self.substitute(unwrap_option_safe(p2)?, depth, env)?;

                Ok(Expr {
                    expr_instance: Some(ExprInstance::ELtBody(ELt {
                        p1: Some(_p1),
                        p2: Some(_p2),
                    })),
                })
            }

            ExprInstance::ELteBody(ELte { p1, p2 }) => {
                let _p1 = self.substitute(unwrap_option_safe(p1)?, depth, env)?;
                let _p2 = self.substitute(unwrap_option_safe(p2)?, depth, env)?;

                Ok(Expr {
                    expr_instance: Some(ExprInstance::ELteBody(ELte {
                        p1: Some(_p1),
                        p2: Some(_p2),
                    })),
                })
            }

            ExprInstance::EGtBody(EGt { p1, p2 }) => {
                let _p1 = self.substitute(unwrap_option_safe(p1)?, depth, env)?;
                let _p2 = self.substitute(unwrap_option_safe(p2)?, depth, env)?;

                Ok(Expr {
                    expr_instance: Some(ExprInstance::EGtBody(EGt {
                        p1: Some(_p1),
                        p2: Some(_p2),
                    })),
                })
            }

            ExprInstance::EGteBody(EGte { p1, p2 }) => {
                let _p1 = self.substitute(unwrap_option_safe(p1)?, depth, env)?;
                let _p2 = self.substitute(unwrap_option_safe(p2)?, depth, env)?;

                Ok(Expr {
                    expr_instance: Some(ExprInstance::EGteBody(EGte {
                        p1: Some(_p1),
                        p2: Some(_p2),
                    })),
                })
            }

            ExprInstance::EEqBody(EEq { p1, p2 }) => {
                let _p1 = self.substitute(unwrap_option_safe(p1)?, depth, env)?;
                let _p2 = self.substitute(unwrap_option_safe(p2)?, depth, env)?;

                Ok(Expr {
                    expr_instance: Some(ExprInstance::EEqBody(EEq {
                        p1: Some(_p1),
                        p2: Some(_p2),
                    })),
                })
            }

            ExprInstance::ENeqBody(ENeq { p1, p2 }) => {
                let _p1 = self.substitute(unwrap_option_safe(p1)?, depth, env)?;
                let _p2 = self.substitute(unwrap_option_safe(p2)?, depth, env)?;

                Ok(Expr {
                    expr_instance: Some(ExprInstance::ENeqBody(ENeq {
                        p1: Some(_p1),
                        p2: Some(_p2),
                    })),
                })
            }

            ExprInstance::EAndBody(EAnd { p1, p2 }) => {
                let _p1 = self.substitute(unwrap_option_safe(p1)?, depth, env)?;
                let _p2 = self.substitute(unwrap_option_safe(p2)?, depth, env)?;

                Ok(Expr {
                    expr_instance: Some(ExprInstance::EAndBody(EAnd {
                        p1: Some(_p1),
                        p2: Some(_p2),
                    })),
                })
            }

            ExprInstance::EOrBody(EOr { p1, p2 }) => {
                let _p1 = self.substitute(unwrap_option_safe(p1)?, depth, env)?;
                let _p2 = self.substitute(unwrap_option_safe(p2)?, depth, env)?;

                Ok(Expr {
                    expr_instance: Some(ExprInstance::EOrBody(EOr {
                        p1: Some(_p1),
                        p2: Some(_p2),
                    })),
                })
            }

            ExprInstance::EMatchesBody(EMatches { target, pattern }) => {
                let _target = self.substitute(unwrap_option_safe(target)?, depth, env)?;
                let _pattern = self.substitute(unwrap_option_safe(pattern)?, depth, env)?;

                Ok(Expr {
                    expr_instance: Some(ExprInstance::ELtBody(ELt {
                        p1: Some(_target),
                        p2: Some(_pattern),
                    })),
                })
            }

            ExprInstance::EListBody(EList {
                ps,
                locally_free,
                connective_used,
                remainder,
            }) => {
                let _ps = ps
                    .iter()
                    .map(|p| self.substitute(p.clone(), depth, env))
                    .collect::<Result<Vec<Par>, InterpreterError>>()?;

                let new_locally_free = set_bits_until(locally_free, env.shift);

                Ok(Expr {
                    expr_instance: Some(ExprInstance::EListBody(EList {
                        ps: _ps,
                        locally_free: new_locally_free,
                        connective_used,
                        remainder,
                    })),
                })
            }

            ExprInstance::ETupleBody(ETuple {
                ps,
                locally_free,
                connective_used,
            }) => {
                let _ps = ps
                    .iter()
                    .map(|p| self.substitute(p.clone(), depth, env))
                    .collect::<Result<Vec<Par>, InterpreterError>>()?;

                let new_locally_free = set_bits_until(locally_free, env.shift);

                Ok(Expr {
                    expr_instance: Some(ExprInstance::ETupleBody(ETuple {
                        ps: _ps,
                        locally_free: new_locally_free,
                        connective_used,
                    })),
                })
            }

            ExprInstance::ESetBody(eset) => {
                let par_set = ParSetTypeMapper::eset_to_par_set(eset);
                let _ps = par_set
                    .ps
                    .sorted_pars
                    .iter()
                    .map(|p| self.substitute(p.clone(), depth, env))
                    .collect::<Result<Vec<Par>, InterpreterError>>()?;

                Ok(Expr {
                    expr_instance: Some(ExprInstance::ESetBody(ParSetTypeMapper::par_set_to_eset(
                        ParSet {
                            ps: SortedParHashSet::create_from_vec(_ps),
                            connective_used: par_set.connective_used,
                            locally_free: set_bits_until(par_set.locally_free, env.shift),
                            remainder: par_set.remainder,
                        },
                    ))),
                })
            }

            ExprInstance::EMapBody(emap) => {
                let par_map = ParMapTypeMapper::emap_to_par_map(emap);
                let _ps = par_map
                    .ps
                    .sorted_list
                    .iter()
                    .map(|p| {
                        let p1 = self.substitute(p.0.clone(), depth, env)?;
                        let p2 = self.substitute(p.1.clone(), depth, env)?;
                        Ok((p1, p2))
                    })
                    .collect::<Result<Vec<(Par, Par)>, InterpreterError>>()?;

                Ok(Expr {
                    expr_instance: Some(ExprInstance::EMapBody(ParMapTypeMapper::par_map_to_emap(
                        ParMap {
                            ps: SortedParMap::create_from_vec(_ps),
                            connective_used: par_map.connective_used,
                            locally_free: set_bits_until(par_map.locally_free, env.shift),
                            remainder: par_map.remainder,
                        },
                    ))),
                })
            }

            ExprInstance::EMethodBody(EMethod {
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

                Ok(Expr {
                    expr_instance: Some(ExprInstance::EMethodBody(EMethod {
                        method_name,
                        target: Some(sub_target),
                        arguments: sub_arguments,
                        locally_free: set_bits_until(locally_free, env.shift),
                        connective_used,
                    })),
                })
            }

            _ => Ok(term),
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

            Expr::EList(EListBody {
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

                Expr::EList(EListBody {
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
                )
                .into_iter()
                .map(|v| v as usize);

                Expr::EMethod(EMethodBody {
                    method_name,
                    target,
                    arguments,
                    locally_free: BitVec::from_iter(locally_free),
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

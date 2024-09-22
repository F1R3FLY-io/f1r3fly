use expr::ExprInstance;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::collections::HashSet;

use crate::rhoapi::*;
use crate::rust::utils::connective::ConnectiveInstance::*;
use crate::rust::utils::expr::ExprInstance::EVarBody;
use crate::rust::utils::expr::ExprInstance::*;
use crate::rust::utils::var::VarInstance::{BoundVar, FreeVar, Wildcard};
use crate::rust::utils::var::WildcardMsg;

use super::par_map::ParMap;
use super::par_map_type_mapper::ParMapTypeMapper;
use super::par_set::ParSet;
use super::par_set_type_mapper::ParSetTypeMapper;
use super::rholang::implicits::vector_par;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct OptionResult<A, K> {
    pub continuation: K,
    pub data: A,
}

// Adding helper functions 'with_*' to protobuf message 'Par'
impl Par {
    pub fn with_sends(&self, new_sends: Vec<Send>) -> Par {
        Par {
            sends: new_sends,
            ..self.clone()
        }
    }

    pub fn with_receives(&self, new_receives: Vec<Receive>) -> Par {
        Par {
            receives: new_receives,
            ..self.clone()
        }
    }

    pub fn with_news(&self, new_news: Vec<New>) -> Par {
        Par {
            news: new_news,
            ..self.clone()
        }
    }

    pub fn with_exprs(&self, new_exprs: Vec<Expr>) -> Par {
        Par {
            exprs: new_exprs,
            ..self.clone()
        }
    }

    pub fn with_matches(&self, new_matches: Vec<Match>) -> Par {
        Par {
            matches: new_matches,
            ..self.clone()
        }
    }

    pub fn with_bundles(&self, new_bundles: Vec<Bundle>) -> Par {
        Par {
            bundles: new_bundles,
            ..self.clone()
        }
    }

    pub fn with_unforgeables(&self, new_unforgeables: Vec<GUnforgeable>) -> Par {
        Par {
            unforgeables: new_unforgeables,
            ..self.clone()
        }
    }

    pub fn with_connectives(&self, new_connectives: Vec<Connective>) -> Par {
        Par {
            connectives: new_connectives,
            ..self.clone()
        }
    }

    // See models/src/main/scala/coop/rchain/models/rholang/implicits.scala - prepend
    pub fn prepend_send(&mut self, s: Send) -> Par {
        let mut new_sends = vec![s.clone()];
        new_sends.append(&mut self.sends);

        Par {
            sends: new_sends,
            locally_free: union(self.locally_free.clone(), s.locally_free),
            connective_used: self.connective_used || s.connective_used,
            ..self.clone()
        }
    }

    pub fn is_empty(&self) -> bool {
        self.sends.is_empty()
            && self.receives.is_empty()
            && self.news.is_empty()
            && self.matches.is_empty()
            && self.bundles.is_empty()
            && self.exprs.is_empty()
    }
}

// See rholang/src/main/scala/coop/rchain/rholang/interpreter/matcher/package.scala - FreeMap
pub type FreeMap = BTreeMap<i32, Par>;
pub fn new_free_map() -> FreeMap {
    BTreeMap::new()
}

// See rholang/src/main/scala/coop/rchain/rholang/interpreter/matcher/package.scala - runFirst
// STUBBED OUT
pub fn run_first<A>() -> Option<(FreeMap, A)> {
    None
}

// See rholang/src/main/scala/coop/rchain/rholang/interpreter/matcher/package.scala - attemptOpt
// NOT FULLY IMPLEMENTED
pub fn attempt_opt(operation: Option<()>) -> Option<()> {
    operation.map(|_| ())
}

// See rholang/src/main/scala/coop/rchain/rholang/interpreter/storage/package.scala - toSeq
pub fn to_vec(fm: FreeMap, max: i32) -> Vec<Par> {
    (0..max)
        .map(|i| match fm.get(&i) {
            Some(par) => par.clone(),
            None => Par::default(),
        })
        .collect()
}

/*
    'This function takes two Vec<u8> as input, converts them to HashSet<u8>, performs the union operation, and then converts
  the result back to Vec<u8>.' - GPT-4
*/
pub fn union<A: Eq + std::hash::Hash + Clone>(vec1: Vec<A>, vec2: Vec<A>) -> Vec<A> {
    let set1: HashSet<_> = vec1.into_iter().collect();
    let set2: HashSet<_> = vec2.into_iter().collect();
    set1.union(&set2).cloned().collect()
}

// See rholang/src/main/scala/coop/rchain/rholang/interpreter/matcher/ParSpatialMatcherUtils.scala - noFrees[Par]
pub fn no_frees(par: Par) -> Par {
    par.with_exprs(no_frees_exprs(par.exprs.clone()))
}

// See rholang/src/main/scala/coop/rchain/rholang/interpreter/matcher/ParSpatialMatcherUtils.scala - noFrees[Seq[Expr]]
pub fn no_frees_exprs(exprs: Vec<Expr>) -> Vec<Expr> {
    exprs
        .iter()
        .filter(|expr| match expr.expr_instance.clone() {
            Some(EVarBody(EVar { v })) => match v.unwrap().var_instance {
                Some(FreeVar(_)) => false,
                Some(Wildcard(_)) => false,
                _ => true,
            },

            _ => true,
        })
        .cloned()
        .collect()
}

// See shared/src/main/scala/coop/rchain/catscontrib/Alternative_.scala - guard
pub fn guard(condition: bool) -> Option<()> {
    if condition {
        Some(())
    } else {
        None
    }
}

// Helper functions
pub fn new_conn_and_body_par(
    _ps: Vec<Par>,
    _locally_free_par: Vec<u8>,
    _connective_used_par: bool,
) -> Par {
    vector_par(_locally_free_par, _connective_used_par).with_connectives(vec![Connective {
        connective_instance: Some(ConnAndBody(ConnectiveBody { ps: _ps })),
    }])
}

pub fn new_conn_or_body_par(
    _ps: Vec<Par>,
    _locally_free_par: Vec<u8>,
    _connective_used_par: bool,
) -> Par {
    vector_par(_locally_free_par, _connective_used_par).with_connectives(vec![Connective {
        connective_instance: Some(ConnOrBody(ConnectiveBody { ps: _ps })),
    }])
}

pub fn new_conn_not_body_par(
    _body: Par,
    _locally_free_par: Vec<u8>,
    _connective_used_par: bool,
) -> Par {
    vector_par(_locally_free_par, _connective_used_par).with_connectives(vec![Connective {
        connective_instance: Some(ConnNotBody(_body)),
    }])
}

pub fn new_send(
    _chan: Par,
    _data: Vec<Par>,
    _persistent: bool,
    _locally_free: Vec<u8>,
    _connective_used: bool,
) -> Send {
    Send {
        chan: Some(_chan),
        data: _data,
        persistent: _persistent,
        locally_free: _locally_free,
        connective_used: _connective_used,
    }
}

pub fn new_send_par(
    _chan: Par,
    _data: Vec<Par>,
    _persistent: bool,
    _locally_free: Vec<u8>,
    _connective_used: bool,
    _locally_free_par: Vec<u8>,
    _connective_used_par: bool,
) -> Par {
    vector_par(_locally_free_par, _connective_used_par).with_sends(vec![Send {
        chan: Some(_chan),
        data: _data,
        persistent: _persistent,
        locally_free: _locally_free,
        connective_used: _connective_used,
    }])
}

pub fn new_match_par(
    _target: Par,
    _cases: Vec<MatchCase>,
    _locally_free: Vec<u8>,
    _connective_used: bool,
    _locally_free_par: Vec<u8>,
    _connective_used_par: bool,
) -> Par {
    vector_par(_locally_free_par, _connective_used_par).with_matches(vec![Match {
        target: Some(_target),
        cases: _cases,
        locally_free: _locally_free,
        connective_used: _connective_used,
    }])
}

pub fn new_receive_par(
    _binds: Vec<ReceiveBind>,
    _body: Par,
    _persistent: bool,
    _peek: bool,
    _bind_count: i32,
    _locally_free: Vec<u8>,
    _connective_used: bool,
    _locally_free_par: Vec<u8>,
    _connective_used_par: bool,
) -> Par {
    vector_par(_locally_free_par, _connective_used_par).with_receives(vec![Receive {
        binds: _binds,
        body: Some(_body),
        persistent: _persistent,
        peek: _peek,
        bind_count: _bind_count,
        locally_free: _locally_free,
        connective_used: _connective_used,
    }])
}

pub fn new_new_par(
    _bind_count: i32,
    _p: Par,
    _uri: Vec<String>,
    _injections: BTreeMap<String, Par>,
    _locally_free: Vec<u8>,
    _locally_free_par: Vec<u8>,
    _connective_used_par: bool,
) -> Par {
    vector_par(_locally_free_par, _connective_used_par).with_news(vec![New {
        bind_count: _bind_count,
        p: Some(_p),
        uri: _uri,
        injections: _injections,
        locally_free: _locally_free,
    }])
}

pub fn new_eset_par(
    _ps: Vec<Par>,
    _locally_free: Vec<u8>,
    _connective_used: bool,
    _remainder: Option<Var>,
    _locally_free_par: Vec<u8>,
    _connective_used_par: bool,
) -> Par {
    vector_par(_locally_free_par, _connective_used_par).with_exprs(vec![new_eset_expr(
        _ps,
        _locally_free,
        _connective_used,
        _remainder,
    )])
}

pub fn new_eset_expr(
    _ps: Vec<Par>,
    _locally_free: Vec<u8>,
    _connective_used: bool,
    _remainder: Option<Var>,
) -> Expr {
    Expr {
        expr_instance: Some(ESetBody(ParSetTypeMapper::par_set_to_eset(ParSet::new(
            _ps,
            _connective_used,
            _locally_free,
            _remainder,
        )))),
    }
}

pub fn new_emap_par(
    _kvs: Vec<KeyValuePair>,
    _locally_free: Vec<u8>,
    _connective_used: bool,
    _remainder: Option<Var>,
    _locally_free_par: Vec<u8>,
    _connective_used_par: bool,
) -> Par {
    vector_par(_locally_free_par, _connective_used_par).with_exprs(vec![new_emap_expr(
        _kvs,
        _locally_free,
        _connective_used,
        _remainder,
    )])
}

pub fn new_emap_expr(
    _kvs: Vec<KeyValuePair>,
    _locally_free: Vec<u8>,
    _connective_used: bool,
    _remainder: Option<Var>,
) -> Expr {
    Expr {
        expr_instance: Some(EMapBody(ParMapTypeMapper::par_map_to_emap(ParMap::new(
            _kvs.into_iter()
                .filter_map(|kv| {
                    if let (Some(key), Some(value)) = (kv.key, kv.value) {
                        Some((key, value))
                    } else {
                        None
                    }
                })
                .collect(),
            _connective_used,
            _locally_free,
            _remainder,
        )))),
    }
}

pub fn new_key_value_pair(_key: Par, _value: Par) -> KeyValuePair {
    KeyValuePair {
        key: Some(_key),
        value: Some(_value),
    }
}

pub fn new_gint_par(value: i64, _locally_free_par: Vec<u8>, _connective_used_par: bool) -> Par {
    vector_par(_locally_free_par, _connective_used_par).with_exprs(vec![new_gint_expr(value)])
}

pub fn new_gint_expr(value: i64) -> Expr {
    Expr {
        expr_instance: Some(GInt(value)),
    }
}

pub fn new_gbool_par(value: bool, _locally_free_par: Vec<u8>, _connective_used_par: bool) -> Par {
    vector_par(_locally_free_par, _connective_used_par).with_exprs(vec![new_gbool_expr(value)])
}

pub fn new_gbool_expr(value: bool) -> Expr {
    Expr {
        expr_instance: Some(GBool(value)),
    }
}

pub fn new_gstring_par(
    value: String,
    _locally_free_par: Vec<u8>,
    _connective_used_par: bool,
) -> Par {
    vector_par(_locally_free_par, _connective_used_par).with_exprs(vec![new_gstring_expr(value)])
}

pub fn new_gstring_expr(value: String) -> Expr {
    Expr {
        expr_instance: Some(GString(value)),
    }
}

pub fn new_wildcard_par(_locally_free_par: Vec<u8>, _connective_used_par: bool) -> Par {
    vector_par(_locally_free_par, _connective_used_par).with_exprs(vec![Expr {
        expr_instance: Some(EVarBody(EVar {
            v: Some(Var {
                var_instance: Some(Wildcard(WildcardMsg {})),
            }),
        })),
    }])
}

pub fn new_wildcard_expr() -> Expr {
    Expr {
        expr_instance: Some(EVarBody(EVar {
            v: Some(Var {
                var_instance: Some(Wildcard(WildcardMsg {})),
            }),
        })),
    }
}

pub fn new_wildcard_var() -> Var {
    Var {
        var_instance: Some(Wildcard(WildcardMsg {})),
    }
}

pub fn new_boundvar_par(value: i32, _locally_free_par: Vec<u8>, _connective_used_par: bool) -> Par {
    vector_par(_locally_free_par, _connective_used_par).with_exprs(vec![new_boundvar_expr(value)])
}

pub fn new_boundvar_expr(value: i32) -> Expr {
    Expr {
        expr_instance: Some(EVarBody(EVar {
            v: Some(Var {
                var_instance: Some(BoundVar(value)),
            }),
        })),
    }
}

// "connective_used" is always "true" on "freevar"
pub fn new_freevar_par(value: i32, _locally_free_par: Vec<u8>) -> Par {
    vector_par(_locally_free_par, true).with_exprs(vec![new_freevar_expr(value)])
}

pub fn new_freevar_expr(value: i32) -> Expr {
    Expr {
        expr_instance: Some(EVarBody(EVar {
            v: Some(Var {
                var_instance: Some(FreeVar(value)),
            }),
        })),
    }
}

pub fn new_freevar_var(value: i32) -> Var {
    Var {
        var_instance: Some(FreeVar(value)),
    }
}

pub fn new_elist_par(
    _ps: Vec<Par>,
    _locally_free: Vec<u8>,
    _connective_used_elist: bool,
    _remainder: Option<Var>,
    _locally_free_par: Vec<u8>,
    _connective_used_par: bool,
) -> Par {
    vector_par(_locally_free_par, _connective_used_par).with_exprs(vec![new_elist_expr(
        _ps,
        _locally_free,
        _connective_used_elist,
        _remainder,
    )])
}

pub fn new_elist_expr(
    _ps: Vec<Par>,
    _locally_free: Vec<u8>,
    _connective_used: bool,
    _remainder: Option<Var>,
) -> Expr {
    Expr {
        expr_instance: Some(EListBody(EList {
            ps: _ps,
            locally_free: _locally_free,
            connective_used: _connective_used,
            remainder: _remainder,
        })),
    }
}

pub fn new_etuple_par(_ps: Vec<Par>) -> Par {
    vector_par(Vec::new(), false).with_exprs(vec![new_etuple_expr(_ps, Vec::new(), false)])
}

pub fn new_etuple_expr(_ps: Vec<Par>, _locally_free: Vec<u8>, _connective_used: bool) -> Expr {
    Expr {
        expr_instance: Some(ETupleBody(ETuple {
            ps: _ps,
            locally_free: _locally_free,
            connective_used: _connective_used,
        })),
    }
}

pub fn new_eplus_par(
    lhs_value: i64,
    rhs_value: i64,
    locally_free_par: Vec<u8>,
    connective_used_par: bool,
) -> Par {
    Par::default().with_exprs(vec![Expr {
        expr_instance: Some(ExprInstance::EPlusBody(EPlus {
            p1: Some(new_gint_par(
                lhs_value,
                locally_free_par.clone(),
                connective_used_par,
            )),
            p2: Some(new_gint_par(
                rhs_value,
                locally_free_par,
                connective_used_par,
            )),
        })),
    }])
}

pub fn new_bundle_par(body: Par, write_flag: bool, read_flag: bool) -> Par {
    Par::default().with_bundles(vec![Bundle {
        body: Some(body),
        write_flag,
        read_flag,
    }])
}

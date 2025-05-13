use std::{cmp::Ordering, collections::BTreeMap, ops::Deref, ops::DerefMut};

use bitvec::{order::Lsb0, slice::BitSlice, vec::BitVec};
use itertools::Itertools;
use models::{
    rhoapi::{
        EMinusMinus, EMod, EPercentPercent, EPlusPlus, KeyValuePair,
        expr::ExprInstance::*,
        var::{VarInstance, WildcardMsg},
    },
    rust::par_set::ParSet,
};
use prost::Message;

use crate::sort_matcher::Sorted;

use super::{sort_matcher::Sortable, sorter::*};

pub const GUNFORGEABLE_SIZE: usize = 32;

/// A parallel composition of Rholang terms.
#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct Par<const N: usize = GUNFORGEABLE_SIZE> {
    pub sends: Vec<Send>,
    pub receives: Vec<Receive>,
    pub news: Vec<New>,
    pub exprs: Vec<Expr>,
    pub matches: Vec<Match>,
    pub unforgeables: Vec<GUnforgeable<N>>,
    pub bundles: Vec<Bundle>,
    pub connectives: Vec<Connective>,
    pub locally_free: MyBitVec,
    pub connective_used: bool,
}

impl<const N: usize> Default for Par<N> {
    fn default() -> Self {
        Self {
            sends: Vec::new(),
            receives: Vec::new(),
            news: Vec::new(),
            exprs: Vec::new(),
            matches: Vec::new(),
            unforgeables: Vec::new(),
            bundles: Vec::new(),
            connectives: Vec::new(),
            locally_free: BitVec::EMPTY.into(),
            connective_used: false,
        }
    }
}

impl<const N: usize> From<Par<N>> for models::rhoapi::Par {
    fn from(value: Par<N>) -> Self {
        Self {
            sends: value.sends.into_iter().map(Into::into).collect(),
            receives: value.receives.into_iter().map(Into::into).collect(),
            news: value.news.into_iter().map(Into::into).collect(),
            exprs: value.exprs.into_iter().map(|v| v.into()).collect(),
            matches: value
                .matches
                .into_iter()
                .map(|v| models::rhoapi::Match {
                    target: Some(v.target.into()),
                    cases: v.cases.into_iter().map(Into::into).collect(),
                    locally_free: v.locally_free.into(),
                    connective_used: v.connective_used,
                })
                .collect(),
            unforgeables: value.unforgeables.into_iter().map(|v| v.into()).collect(),
            bundles: value.bundles.into_iter().map(Into::into).collect(),
            connectives: value.connectives.into_iter().map(Into::into).collect(),
            locally_free: value.locally_free.into(),
            connective_used: value.connective_used,
        }
    }
}

impl<const N: usize> Par<N> {
    pub fn concat_with(&mut self, other: Par<N>) {
        if self.is_nil() {
            // worth it?
            self.clone_from(&other);
            return;
        }
        self.sends.extend(other.sends);
        self.receives.extend(other.receives);
        self.news.extend(other.news);
        self.exprs.extend(other.exprs);
        self.matches.extend(other.matches);
        //This one uses a different form to take advantage of the fact that Unforgeable's are copyable
        self.unforgeables.extend(&other.unforgeables);
        self.bundles.extend(other.bundles);
        union_inplace(&mut self.locally_free, &other.locally_free);
        self.connective_used = self.connective_used || other.connective_used;
    }

    pub fn is_nil(&self) -> bool {
        !self.connective_used
            && self.bundles.is_empty()
            && self.connectives.is_empty()
            && self.exprs.is_empty()
            && self.locally_free.is_empty()
            && self.matches.is_empty()
            && self.news.is_empty()
            && self.receives.is_empty()
            && self.sends.is_empty()
            && self.unforgeables.is_empty()
    }

    pub fn single_bundle(&self) -> Option<&Bundle> {
        if self.is_single_bundle() {
            self.bundles.first()
        } else {
            None
        }
    }
    pub fn single_bundle_mut(&mut self) -> Option<&mut Bundle> {
        if self.is_single_bundle() {
            self.bundles.first_mut()
        } else {
            None
        }
    }
    pub fn cast_to_bundle(mut self) -> Bundle {
        self.bundles.swap_remove(0)
    }
    fn is_single_bundle(&self) -> bool {
        self.sends.is_empty()
            && self.receives.is_empty()
            && self.news.is_empty()
            && self.exprs.is_empty()
            && self.matches.is_empty()
            && self.unforgeables.is_empty()
            && self.connectives.is_empty()
            && self.bundles.len() == 1
    }

    pub fn single_connective(&self) -> Option<&Connective> {
        if self.is_single_connective() {
            self.connectives.first()
        } else {
            None
        }
    }
    pub fn single_connective_mut(&mut self) -> Option<&mut Connective> {
        if self.is_single_connective() {
            self.connectives.first_mut()
        } else {
            None
        }
    }
    pub fn cast_to_connective(mut self) -> Connective {
        self.connectives.swap_remove(0)
    }
    fn is_single_connective(&self) -> bool {
        self.sends.is_empty()
            && self.receives.is_empty()
            && self.news.is_empty()
            && self.exprs.is_empty()
            && self.matches.is_empty()
            && self.bundles.is_empty()
            && self.connectives.len() == 1
    }

    pub fn push_bundle(&mut self, b: Bundle) {
        union_inplace(&mut self.locally_free, &b.body.locally_free);
        self.bundles.push(b);
    }

    pub fn push_expr(&mut self, e: Expr, depth: u32) {
        union_inplace(&mut self.locally_free, &e.locally_free(depth));
        self.connective_used = self.connective_used || e.connective_used();
        self.exprs.push(e);
    }

    pub fn push_match(&mut self, m: Match) {
        union_inplace(&mut self.locally_free, &m.locally_free);
        self.connective_used = self.connective_used || m.connective_used;
        self.matches.push(m);
    }

    pub fn push_connective(&mut self, c: Connective, depth: u32) {
        self.locally_free = c.locally_free(depth); // is it ok?
        self.connective_used = self.connective_used || c.connective_used();
        self.connectives.push(c);
    }

    pub fn push_receive(&mut self, r: Receive) {
        union_inplace(&mut self.locally_free, &r.locally_free);
        self.connective_used = self.connective_used || r.connective_used;
        self.receives.push(r);
    }

    pub fn push_send(&mut self, s: Send) {
        union_inplace(&mut self.locally_free, &s.locally_free[..]);
        self.connective_used = self.connective_used || s.connective_used;
        self.sends.push(s);
    }

    pub fn push_new(&mut self, new: New) {
        union_inplace(&mut self.locally_free, &new.locally_free);
        self.connective_used = self.connective_used || new.p.connective_used;
        self.news.push(new);
    }

    pub fn gtrue() -> Par<N> {
        Par::<N> {
            exprs: vec![Expr::GTRUE],
            ..Default::default()
        }
    }

    pub fn gfalse() -> Par<N> {
        Par::<N> {
            exprs: vec![Expr::GFALSE],
            ..Default::default()
        }
    }

    pub fn gint(value: i64) -> Par<N> {
        Par::<N> {
            exprs: vec![Expr::GInt(value)],
            ..Default::default()
        }
    }

    pub fn gstr(value: String) -> Par<N> {
        Par::<N> {
            exprs: vec![Expr::GString(value)],
            ..Default::default()
        }
    }

    pub fn wild() -> Par<N> {
        Par::<N> {
            exprs: vec![Expr::WILDCARD],
            connective_used: true,
            ..Default::default()
        }
    }

    pub fn bound_var(idx: u32) -> Par<N> {
        Par::<N> {
            exprs: vec![Expr::new_bound_var(idx)],
            locally_free: single_bit(idx as usize),
            ..Default::default()
        }
    }

    pub fn free_var(idx: u32) -> Par<N> {
        Par::<N> {
            exprs: vec![Expr::new_free_var(idx)],
            connective_used: true,
            ..Default::default()
        }
    }

    pub fn free_vars(n: u32) -> Vec<Par<N>> {
        (0..n).map(Par::free_var).collect()
    }

    pub fn bound_vars(n: u32) -> Vec<Par<N>> {
        (0..n).rev().map(Par::bound_var).collect_vec()
    }

    pub fn elist(body: EListBody) -> Par<N> {
        let connective_used = body.connective_used;
        let locally_free = body.locally_free.clone();
        Par::<N> {
            exprs: vec![Expr::EList(body)],
            locally_free,
            connective_used,
            ..Default::default()
        }
    }
}

impl<const N: usize> From<models::rhoapi::Par> for Par<N> {
    fn from(par: models::rhoapi::Par) -> Self {
        Par {
            sends: par.sends.into_iter().map(Into::into).collect(),
            receives: par.receives.into_iter().map(Into::into).collect(),
            news: par.news.into_iter().map(Into::into).collect(),
            exprs: par.exprs.into_iter().map(Into::into).collect(),
            matches: par.matches.into_iter().map(Into::into).collect(),
            unforgeables: par.unforgeables.into_iter().map(Into::into).collect(),
            bundles: par.bundles.into_iter().map(Into::into).collect(),
            connectives: par.connectives.into_iter().map(Into::into).collect(),
            locally_free: MyBitVec::<usize, _>::from_iter(
                par.locally_free.into_iter().map(|v| v as usize),
            ),
            connective_used: par.connective_used.into(),
        }
    }
}

impl Sortable for Par {
    type Sorter<'a> = ParSorter<'a>;

    fn sorter(&mut self) -> Self::Sorter<'_> {
        ParSorter::new(self)
    }
}

/// *
/// Either rholang code or code built in to the interpreter.

pub enum TaggedContinuation {
    ParBody(ParWithRandom),
    ScalaBodyRef(i64),
}

/// *
/// Rholang code along with the state of a split random number
/// generator for generating new unforgeable names.

pub struct ParWithRandom {
    pub body: Par,
    pub random_state: Vec<u8>,
}

/// *
/// Cost of the performed operations.

pub struct PCost(u64);

pub struct ListParWithRandom {
    pub pars: Vec<Par>,
    pub random_state: Vec<u8>,
}

/// While we use vars in both positions, when producing the normalized
/// representation we need a discipline to track whether a var is a name or a
/// process.
/// These are DeBruijn levels
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub enum Var {
    BoundVar(u32),
    FreeVar(u32),
    Wildcard,
}

impl From<models::rhoapi::Var> for Var {
    fn from(value: models::rhoapi::Var) -> Self {
        match value.var_instance.unwrap() {
            models::rhoapi::var::VarInstance::BoundVar(v) => Var::BoundVar(v as u32),
            models::rhoapi::var::VarInstance::FreeVar(v) => Var::FreeVar(v as u32),
            models::rhoapi::var::VarInstance::Wildcard(_) => Var::Wildcard,
        }
    }
}

impl From<Var> for models::rhoapi::Var {
    fn from(value: Var) -> Self {
        let var_instance = match value {
            Var::BoundVar(v) => VarInstance::BoundVar(v as i32),
            Var::FreeVar(v) => VarInstance::FreeVar(v as i32),
            Var::Wildcard => VarInstance::Wildcard(WildcardMsg {}),
        };
        Self {
            var_instance: Some(var_instance),
        }
    }
}

impl Var {
    pub fn connective_used(&self) -> bool {
        match self {
            Var::BoundVar(_) => false,
            _ => true,
        }
    }

    pub fn locally_free(&self, depth: u32) -> MyBitVec {
        match self {
            Var::BoundVar(index) if depth == 0 => single_bit(*index as usize),
            _ => BitVec::EMPTY.into(),
        }
    }
}

/// *
/// Nothing can be received from a (quoted) bundle with `readFlag = false`.
/// Likewise nothing can be sent to a (quoted) bundle with `writeFlag = false`.
///
/// If both flags are set to false, bundle allows only for equivalance check.
#[derive(Debug, PartialEq, Eq, Clone, Default, Hash)]
pub struct Bundle {
    pub body: Par,
    /// flag indicating whether bundle is writeable
    pub write_flag: bool,
    /// flag indicating whether bundle is readable
    pub read_flag: bool,
}

impl From<models::rhoapi::Bundle> for Bundle {
    fn from(bundle: models::rhoapi::Bundle) -> Self {
        Bundle {
            body: bundle.body.map(Into::into).unwrap(),
            write_flag: bundle.write_flag,
            read_flag: bundle.read_flag,
        }
    }
}

impl From<Bundle> for models::rhoapi::Bundle {
    fn from(bundle: Bundle) -> Self {
        models::rhoapi::Bundle {
            body: Some(bundle.body.into()),
            write_flag: bundle.write_flag,
            read_flag: bundle.read_flag,
        }
    }
}

impl Sortable for Bundle {
    type Sorter<'a> = BundleSorter<'a>;

    fn sorter(&mut self) -> Self::Sorter<'_> {
        BundleSorter::new(self)
    }
}

// Helper enum. This is 'GeneratedMessage' in Scala
#[derive(Clone, Debug)]
pub enum GeneratedMessage {
    Send(Send),
    Receive(Receive),
    New(New),
    Match(Match),
    Bundle(Bundle),
    Expr(Expr),
}

/// *
/// A send is written `chan!(data)` or `chan!!(data)` for a persistent send.
///
/// Upon send, all free variables in data are substituted with their values.
#[derive(Debug, PartialEq, Eq, Clone, Default, Hash)]
pub struct Send {
    pub chan: Par,
    pub data: Vec<Par>,
    pub persistent: bool,
    pub locally_free: MyBitVec,
    pub connective_used: bool,
}

impl From<Send> for models::rhoapi::Send {
    fn from(send: Send) -> Self {
        models::rhoapi::Send {
            chan: Some(send.chan.into()),
            data: send.data.into_iter().map(Into::into).collect(),
            persistent: send.persistent,
            locally_free: send.locally_free.into(),
            connective_used: send.connective_used.into(),
        }
    }
}

impl From<models::rhoapi::Send> for Send {
    fn from(send: models::rhoapi::Send) -> Self {
        Send {
            chan: send.chan.map(Into::into).unwrap(),
            data: send.data.into_iter().map(Into::into).collect(),
            persistent: send.persistent,
            locally_free: MyBitVec::<_, Lsb0>::from_iter(
                send.locally_free.into_iter().map(|v| v as usize),
            ),
            connective_used: send.connective_used.into(),
        }
    }
}

impl Sortable for Send {
    type Sorter<'a> = SendSorter<'a>;

    fn sorter(&mut self) -> Self::Sorter<'_> {
        SendSorter::new(self)
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct ReceiveBind {
    pub patterns: Vec<Par>,
    pub source: Par,
    pub remainder: Option<Var>,
    pub free_count: u32,
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct BindPattern {
    pub patterns: Vec<Par>,
    pub remainder: Option<Var>,
    pub free_count: usize,
}

/// *
/// A receive is written `for(binds) { body }`
/// i.e. `for(patterns <- source) { body }`
/// or for a persistent recieve: `for(patterns <= source) { body }`.
///
/// It's an error for free Variable to occur more than once in a pattern.
#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct Receive {
    pub binds: Vec<ReceiveBind>,
    pub body: Par,
    pub persistent: bool,
    pub peek: bool,
    pub bind_count: u32,
    pub locally_free: MyBitVec,
    pub connective_used: bool,
}

impl From<Receive> for models::rhoapi::Receive {
    fn from(value: Receive) -> Self {
        Self {
            binds: value.binds.into_iter().map(Into::into).collect(),
            body: Some(value.body.into()),
            persistent: value.persistent,
            peek: value.peek,
            bind_count: value.bind_count as i32,
            locally_free: value.locally_free.into(),
            connective_used: value.connective_used,
        }
    }
}

impl From<models::rhoapi::ReceiveBind> for ReceiveBind {
    fn from(value: models::rhoapi::ReceiveBind) -> Self {
        Self {
            patterns: value.patterns.into_iter().map(Into::into).collect(),
            remainder: value.remainder.map(Into::into),
            free_count: value.free_count as u32,
            source: value.source.map(Into::into).unwrap(),
        }
    }
}

impl From<ReceiveBind> for models::rhoapi::ReceiveBind {
    fn from(value: ReceiveBind) -> Self {
        Self {
            patterns: value.patterns.into_iter().map(Into::into).collect(),
            source: Some(value.source.into()),
            remainder: value.remainder.map(Into::into),
            free_count: value.free_count as i32,
        }
    }
}

impl From<models::rhoapi::Receive> for Receive {
    fn from(receive: models::rhoapi::Receive) -> Self {
        Self {
            binds: receive.binds.into_iter().map(Into::into).collect(),
            body: receive.body.map(Into::into).unwrap(),
            persistent: receive.persistent,
            peek: receive.peek,
            bind_count: receive.bind_count as u32,
            locally_free: MyBitVec::<usize, _>::from_vec(
                receive
                    .locally_free
                    .into_iter()
                    .map(|v| v as usize)
                    .collect(),
            ),
            connective_used: receive.connective_used,
        }
    }
}

impl Sortable for Receive {
    type Sorter<'a> = ReceiveSorter<'a>;

    fn sorter(&mut self) -> Self::Sorter<'_> {
        ReceiveSorter::new(self)
    }
}

/// Number of variables bound in the new statement.
/// For normalized form, p should not contain solely another new.
/// Also for normalized form, the first use should be level+0, next use level+1
/// up to level+count for the last used variable.
#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct New {
    /// Includes any uris listed below. This makes it easier to substitute or walk
    /// a term.
    pub bind_count: u32,
    pub p: Par,
    /// For normalization, uri-referenced variables come at the end, and in
    /// lexicographical order.
    pub uris: Vec<String>,
    pub locally_free: MyBitVec,
    pub injections: BTreeMap<String, Par>,
}

impl From<New> for models::rhoapi::New {
    fn from(value: New) -> Self {
        Self {
            bind_count: value.bind_count as i32,
            p: Some(value.p.into()),
            uri: value.uris,
            locally_free: value.locally_free.into(),
            injections: value
                .injections
                .into_iter()
                .map(|(k, v)| (k, v.into()))
                .collect(),
        }
    }
}

impl From<models::rhoapi::New> for New {
    fn from(new: models::rhoapi::New) -> Self {
        Self {
            bind_count: new.bind_count as u32,
            p: new.p.map(Into::into).unwrap(),
            uris: new.uri,
            locally_free: MyBitVec::<usize, _>::from_vec(
                new.locally_free.into_iter().map(|v| v as usize).collect(),
            ),
            injections: new
                .injections
                .into_iter()
                .map(|(key, value)| (key, value.into()))
                .collect(),
        }
    }
}

impl Sortable for New {
    type Sorter<'a> = NewSorter<'a>;

    fn sorter(&mut self) -> Self::Sorter<'_> {
        NewSorter::new(self)
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct MatchCase {
    pub pattern: Par,
    pub source: Par,
    pub free_count: u32,
}

impl From<MatchCase> for models::rhoapi::MatchCase {
    fn from(value: MatchCase) -> Self {
        models::rhoapi::MatchCase {
            pattern: Some(value.pattern.into()),
            source: Some(value.source.into()),
            free_count: value.free_count as i32,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct Match {
    pub target: Par,
    pub cases: Vec<MatchCase>,
    pub locally_free: MyBitVec,
    pub connective_used: bool,
}

impl From<Match> for models::rhoapi::Match {
    fn from(value: Match) -> Self {
        Self {
            target: Some(value.target.into()),
            cases: value.cases.into_iter().map(Into::into).collect(),
            locally_free: value.locally_free.into(),
            connective_used: value.connective_used,
        }
    }
}

#[derive(Debug, Eq, PartialEq, Clone, Hash, Default)]
pub(crate) struct MyBitVec<T = usize, O = Lsb0>(BitVec<T, O>)
where
    T: bitvec::store::BitStore,
    O: bitvec::order::BitOrder;
impl MyBitVec {
    pub(crate) fn from_vec(locally_free: Vec<usize>) -> MyBitVec {
        Self(BitVec::from_vec(locally_free))
    }
}

impl From<BitVec> for MyBitVec {
    fn from(value: BitVec) -> Self {
        Self(value)
    }
}

impl Deref for MyBitVec {
    type Target = BitVec;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for MyBitVec {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<MyBitVec> for Vec<u8> {
    fn from(value: MyBitVec) -> Self {
        value.iter().map(|v| *v as u8).collect()
    }
}

impl FromIterator<usize> for MyBitVec {
    fn from_iter<T: IntoIterator<Item = usize>>(iter: T) -> Self {
        iter.into_iter().collect()
    }
}

impl From<Vec<u8>> for MyBitVec {
    fn from(value: Vec<u8>) -> Self {
        value.into_iter().map(|v| v as usize).collect()
    }
}

impl<T, O> AsRef<BitSlice<T, O>> for MyBitVec<T, O>
where
    T: bitvec::store::BitStore,
    O: bitvec::order::BitOrder,
{
    fn as_ref(&self) -> &BitSlice<T, O> {
        self.0.as_bitslice()
    }
}

impl From<models::rhoapi::Match> for Match {
    fn from(value: models::rhoapi::Match) -> Self {
        Match {
            target: value.target.map(Into::into).unwrap(),
            cases: value
                .cases
                .into_iter()
                .map(|case| MatchCase {
                    pattern: case.pattern.map(Into::into).unwrap(),
                    source: case.source.map(Into::into).unwrap(),
                    free_count: case.free_count as u32,
                })
                .collect(),
            locally_free: value.locally_free.into(),
            connective_used: value.connective_used,
        }
    }
}

impl Sortable for Match {
    type Sorter<'a> = MatchSorter<'a>;

    fn sorter(&mut self) -> Self::Sorter<'_> {
        MatchSorter::new(self)
    }
}

impl From<models::rhoapi::Expr> for Expr {
    fn from(value: models::rhoapi::Expr) -> Self {
        match value.expr_instance.unwrap() {
            models::rhoapi::expr::ExprInstance::GBool(v) => Expr::GBool(v),
            models::rhoapi::expr::ExprInstance::GInt(v) => Expr::GInt(v),
            models::rhoapi::expr::ExprInstance::GString(v) => Expr::GString(v),
            models::rhoapi::expr::ExprInstance::GUri(v) => Expr::GUri(v),
            models::rhoapi::expr::ExprInstance::GByteArray(items) => Expr::GByteArray(items),
            models::rhoapi::expr::ExprInstance::ENotBody(enot) => {
                Expr::ENot(enot.p.map(Into::into).unwrap())
            }
            models::rhoapi::expr::ExprInstance::ENegBody(eneg) => {
                Expr::ENeg(eneg.p.map(Into::into).unwrap())
            }
            models::rhoapi::expr::ExprInstance::EMultBody(models::rhoapi::EMult { p1, p2 }) => {
                Expr::EMult(p1.map(Into::into).unwrap(), p2.map(Into::into).unwrap())
            }
            models::rhoapi::expr::ExprInstance::EDivBody(models::rhoapi::EDiv { p1, p2 }) => {
                Expr::EDiv(p1.map(Into::into).unwrap(), p2.map(Into::into).unwrap())
            }
            models::rhoapi::expr::ExprInstance::EPlusBody(models::rhoapi::EPlus { p1, p2 }) => {
                Expr::EPlus(p1.map(Into::into).unwrap(), p2.map(Into::into).unwrap())
            }
            models::rhoapi::expr::ExprInstance::EMinusBody(models::rhoapi::EMinus { p1, p2 }) => {
                Expr::EMinus(p1.map(Into::into).unwrap(), p2.map(Into::into).unwrap())
            }
            models::rhoapi::expr::ExprInstance::ELtBody(models::rhoapi::ELt { p1, p2 }) => {
                Expr::ELt(p1.map(Into::into).unwrap(), p2.map(Into::into).unwrap())
            }
            models::rhoapi::expr::ExprInstance::ELteBody(models::rhoapi::ELte { p1, p2 }) => {
                Expr::ELte(p1.map(Into::into).unwrap(), p2.map(Into::into).unwrap())
            }
            models::rhoapi::expr::ExprInstance::EGtBody(models::rhoapi::EGt { p1, p2 }) => {
                Expr::EGt(p1.map(Into::into).unwrap(), p2.map(Into::into).unwrap())
            }
            models::rhoapi::expr::ExprInstance::EGteBody(models::rhoapi::EGte { p1, p2 }) => {
                Expr::EGte(p1.map(Into::into).unwrap(), p2.map(Into::into).unwrap())
            }
            models::rhoapi::expr::ExprInstance::EEqBody(models::rhoapi::EEq { p1, p2 }) => {
                Expr::EEq(p1.map(Into::into).unwrap(), p2.map(Into::into).unwrap())
            }
            models::rhoapi::expr::ExprInstance::ENeqBody(models::rhoapi::ENeq { p1, p2 }) => {
                Expr::ENeq(p1.map(Into::into).unwrap(), p2.map(Into::into).unwrap())
            }
            models::rhoapi::expr::ExprInstance::EAndBody(models::rhoapi::EAnd { p1, p2 }) => {
                Expr::EAnd(p1.map(Into::into).unwrap(), p2.map(Into::into).unwrap())
            }
            models::rhoapi::expr::ExprInstance::EOrBody(models::rhoapi::EOr { p1, p2 }) => {
                Expr::EOr(p1.map(Into::into).unwrap(), p2.map(Into::into).unwrap())
            }
            models::rhoapi::expr::ExprInstance::EVarBody(evar) => {
                Expr::EVar(evar.v.map(Into::into).unwrap())
            }
            models::rhoapi::expr::ExprInstance::EListBody(elist) => Expr::EList(elist.into()),
            models::rhoapi::expr::ExprInstance::ETupleBody(etuple) => Expr::ETuple(etuple.into()),
            models::rhoapi::expr::ExprInstance::ESetBody(eset) => Expr::ESet(eset.into()),
            models::rhoapi::expr::ExprInstance::EMapBody(emap) => Expr::EMap(emap.into()),
            models::rhoapi::expr::ExprInstance::EMethodBody(emethod) => {
                Expr::EMethod(emethod.into())
            }
            models::rhoapi::expr::ExprInstance::EMatchesBody(ematches) => {
                Expr::EMatches(ematches.into())
            }
            models::rhoapi::expr::ExprInstance::EPercentPercentBody(EPercentPercent { p1, p2 }) => {
                Expr::EPercentPercent(p1.map(Into::into).unwrap(), p2.map(Into::into).unwrap())
            }
            models::rhoapi::expr::ExprInstance::EPlusPlusBody(EPlusPlus { p1, p2 }) => {
                Expr::EPlusPlus(p1.map(Into::into).unwrap(), p2.map(Into::into).unwrap())
            }
            models::rhoapi::expr::ExprInstance::EMinusMinusBody(EMinusMinus { p1, p2 }) => {
                Expr::EMinusMinus(p1.map(Into::into).unwrap(), p2.map(Into::into).unwrap())
            }
            models::rhoapi::expr::ExprInstance::EModBody(EMod { p1, p2 }) => {
                Expr::EMod(p1.map(Into::into).unwrap(), p2.map(Into::into).unwrap())
            }
        }
    }
}

/// Any process may be an operand to an expression.
/// Only processes equivalent to a ground process of compatible type will reduce.
#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub enum Expr {
    GBool(bool),
    GInt(i64),
    GString(String),
    GUri(String),
    GByteArray(Vec<u8>),
    ENot(Par),
    ENeg(Par),
    EMult(Par, Par),
    EDiv(Par, Par),
    EPlus(Par, Par),
    EMinus(Par, Par),
    ELt(Par, Par),
    ELte(Par, Par),
    EGt(Par, Par),
    EGte(Par, Par),
    EEq(Par, Par),
    ENeq(Par, Par),
    EAnd(Par, Par),
    EOr(Par, Par),
    /// A variable used as a var should be bound in a process context, not a name
    /// context. For example:
    /// `for (@x <- c1; @y <- c2) { z!(x + y) }` is fine, but
    /// `for (x <- c1; y <- c2) { z!(x + y) }` should raise an error.
    EVar(Var),
    EList(EListBody),
    ETuple(ETupleBody),
    ESet(ESetBody),
    EMap(EMapBody),
    EMethod(EMethodBody),
    EMatches(EMatchesBody),
    /// string interpolation
    EPercentPercent(Par, Par),
    /// concatenation
    EPlusPlus(Par, Par),
    /// set difference
    EMinusMinus(Par, Par),
    EMod(Par, Par),
}

impl From<Expr> for models::rhoapi::Expr {
    fn from(value: Expr) -> Self {
        let instance = match value {
            Expr::GBool(v) => GBool(v),
            Expr::GInt(v) => GInt(v),
            Expr::GString(v) => GString(v),
            Expr::GUri(v) => GUri(v),
            Expr::GByteArray(items) => GByteArray(items),
            Expr::ENot(par) => ENotBody(models::rhoapi::ENot {
                p: Some(par.into()),
            }),
            Expr::ENeg(par) => ENegBody(models::rhoapi::ENeg {
                p: Some(par.into()),
            }),
            Expr::EMult(p1, p2) => EMultBody(models::rhoapi::EMult {
                p1: Some(p1.into()),
                p2: Some(p2.into()),
            }),
            Expr::EDiv(p1, p2) => EDivBody(models::rhoapi::EDiv {
                p1: Some(p1.into()),
                p2: Some(p2.into()),
            }),
            Expr::EPlus(p1, p2) => EPlusBody(models::rhoapi::EPlus {
                p1: Some(p1.into()),
                p2: Some(p2.into()),
            }),
            Expr::EMinus(p1, p2) => EMinusBody(models::rhoapi::EMinus {
                p1: Some(p1.into()),
                p2: Some(p2.into()),
            }),
            Expr::ELt(p1, p2) => ELtBody(models::rhoapi::ELt {
                p1: Some(p1.into()),
                p2: Some(p2.into()),
            }),
            Expr::ELte(p1, p2) => ELteBody(models::rhoapi::ELte {
                p1: Some(p1.into()),
                p2: Some(p2.into()),
            }),
            Expr::EGt(p1, p2) => EGtBody(models::rhoapi::EGt {
                p1: Some(p1.into()),
                p2: Some(p2.into()),
            }),
            Expr::EGte(p1, p2) => EGteBody(models::rhoapi::EGte {
                p1: Some(p1.into()),
                p2: Some(p2.into()),
            }),
            Expr::EEq(p1, p2) => EEqBody(models::rhoapi::EEq {
                p1: Some(p1.into()),
                p2: Some(p2.into()),
            }),
            Expr::ENeq(p1, p2) => ENeqBody(models::rhoapi::ENeq {
                p1: Some(p1.into()),
                p2: Some(p2.into()),
            }),
            Expr::EAnd(p1, p2) => EAndBody(models::rhoapi::EAnd {
                p1: Some(p1.into()),
                p2: Some(p2.into()),
            }),
            Expr::EOr(p1, p2) => EOrBody(models::rhoapi::EOr {
                p1: Some(p1.into()),
                p2: Some(p2.into()),
            }),
            Expr::EVar(var) => EVarBody(models::rhoapi::EVar {
                v: Some(match var {
                    Var::BoundVar(idx) => models::rhoapi::Var {
                        var_instance: Some(models::rhoapi::var::VarInstance::BoundVar(idx as i32)),
                    },
                    Var::FreeVar(idx) => models::rhoapi::Var {
                        var_instance: Some(models::rhoapi::var::VarInstance::FreeVar(idx as i32)),
                    },
                    Var::Wildcard => models::rhoapi::Var {
                        var_instance: Some(models::rhoapi::var::VarInstance::Wildcard(
                            WildcardMsg {},
                        )),
                    },
                }),
            }),
            Expr::EList(elist_body) => EListBody(models::rhoapi::EList {
                ps: elist_body.ps.into_iter().map(Into::into).collect(),
                locally_free: elist_body.locally_free.into(),
                connective_used: elist_body.connective_used,
                remainder: elist_body.remainder.map(|var| match var {
                    Var::BoundVar(idx) => models::rhoapi::Var {
                        var_instance: Some(models::rhoapi::var::VarInstance::BoundVar(idx as i32)),
                    },
                    Var::FreeVar(idx) => models::rhoapi::Var {
                        var_instance: Some(models::rhoapi::var::VarInstance::FreeVar(idx as i32)),
                    },
                    Var::Wildcard => models::rhoapi::Var {
                        var_instance: Some(models::rhoapi::var::VarInstance::Wildcard(
                            WildcardMsg {},
                        )),
                    },
                }),
            }),
            Expr::ETuple(etuple_body) => ETupleBody(models::rhoapi::ETuple {
                ps: etuple_body.ps.into_iter().map(Into::into).collect(),
                locally_free: etuple_body.locally_free.into(),
                connective_used: etuple_body.connective_used,
            }),
            Expr::ESet(eset_body) => ESetBody(models::rhoapi::ESet {
                ps: eset_body.ps.into_iter().map(Into::into).collect(),
                locally_free: eset_body.locally_free.into(),
                connective_used: eset_body.connective_used,
                remainder: eset_body.remainder.map(|var| match var {
                    Var::BoundVar(idx) => models::rhoapi::Var {
                        var_instance: Some(models::rhoapi::var::VarInstance::BoundVar(idx as i32)),
                    },
                    Var::FreeVar(idx) => models::rhoapi::Var {
                        var_instance: Some(models::rhoapi::var::VarInstance::FreeVar(idx as i32)),
                    },
                    Var::Wildcard => models::rhoapi::Var {
                        var_instance: Some(models::rhoapi::var::VarInstance::Wildcard(
                            WildcardMsg {},
                        )),
                    },
                }),
            }),
            Expr::EMap(emap_body) => EMapBody(models::rhoapi::EMap {
                kvs: emap_body
                    .ps
                    .into_iter()
                    .map(|(k, v)| models::rhoapi::KeyValuePair {
                        key: Some(k.into()),
                        value: Some(v.into()),
                    })
                    .collect(),
                locally_free: emap_body.locally_free.into(),
                connective_used: emap_body.connective_used,
                remainder: emap_body.remainder.map(|var| match var {
                    Var::BoundVar(idx) => models::rhoapi::Var {
                        var_instance: Some(models::rhoapi::var::VarInstance::BoundVar(idx as i32)),
                    },
                    Var::FreeVar(idx) => models::rhoapi::Var {
                        var_instance: Some(models::rhoapi::var::VarInstance::FreeVar(idx as i32)),
                    },
                    Var::Wildcard => models::rhoapi::Var {
                        var_instance: Some(models::rhoapi::var::VarInstance::Wildcard(
                            WildcardMsg {},
                        )),
                    },
                }),
            }),
            Expr::EMethod(emethod_body) => EMethodBody(models::rhoapi::EMethod {
                method_name: emethod_body.method_name,
                target: Some(emethod_body.target.into()),
                arguments: emethod_body.arguments.into_iter().map(Into::into).collect(),
                locally_free: emethod_body.locally_free.into(),
                connective_used: emethod_body.connective_used,
            }),
            Expr::EMatches(ematches_body) => EMatchesBody(models::rhoapi::EMatches {
                target: Some(ematches_body.target.into()),
                pattern: Some(ematches_body.pattern.into()),
            }),
            Expr::EPercentPercent(p1, p2) => EPercentPercentBody(models::rhoapi::EPercentPercent {
                p1: Some(p1.into()),
                p2: Some(p2.into()),
            }),
            Expr::EPlusPlus(p1, p2) => EPlusPlusBody(models::rhoapi::EPlusPlus {
                p1: Some(p1.into()),
                p2: Some(p2.into()),
            }),
            Expr::EMinusMinus(p1, p2) => EMinusMinusBody(models::rhoapi::EMinusMinus {
                p1: Some(p1.into()),
                p2: Some(p2.into()),
            }),
            Expr::EMod(p1, p2) => EModBody(models::rhoapi::EMod {
                p1: Some(p1.into()),
                p2: Some(p2.into()),
            }),
        };

        Self {
            expr_instance: Some(instance),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub(crate) struct ElistBody {
    pub ps: Vec<Par>,
    pub locally_free: MyBitVec,
    pub connective_used: bool,
    pub remainder: Option<Var>,
}

impl From<models::rhoapi::EList> for EListBody {
    fn from(body: models::rhoapi::EList) -> Self {
        Self {
            ps: body.ps.into_iter().map(Into::into).collect(),
            locally_free: MyBitVec::<usize, _>::from_vec(
                body.locally_free.into_iter().map(|v| v as usize).collect(),
            ),
            connective_used: body.connective_used,
            remainder: body.remainder.map(Into::into),
        }
    }
}

impl Sortable for Expr {
    type Sorter<'a> = ExprSorter<'a>;

    fn sorter(&mut self) -> Self::Sorter<'_> {
        ExprSorter::new(self)
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Default, Hash)]
pub struct EListBody {
    pub ps: Vec<Par>,
    pub locally_free: MyBitVec,
    pub connective_used: bool,
    pub remainder: Option<Var>,
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct ETupleBody {
    pub ps: Vec<Par>,
    pub locally_free: MyBitVec,
    pub connective_used: bool,
}

impl From<models::rhoapi::ETuple> for ETupleBody {
    fn from(tuple: models::rhoapi::ETuple) -> Self {
        ETupleBody {
            ps: tuple.ps.into_iter().map(Into::into).collect(),
            locally_free: MyBitVec::<usize, _>::from_vec(
                tuple.locally_free.into_iter().map(|v| v as usize).collect(),
            ),
            connective_used: tuple.connective_used,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Default, Hash)]
pub struct ESetBody {
    pub ps: Vec<Par>,
    pub locally_free: MyBitVec,
    pub connective_used: bool,
    pub remainder: Option<Var>,
}

impl From<models::rhoapi::ESet> for ESetBody {
    fn from(set: models::rhoapi::ESet) -> Self {
        ESetBody {
            ps: set.ps.into_iter().map(Into::into).collect::<Vec<Par>>(),
            locally_free: MyBitVec::<usize, _>::from_vec(
                set.locally_free.into_iter().map(|v| v as usize).collect(),
            ),
            connective_used: set.connective_used,
            remainder: set.remainder.map(Into::into),
        }
    }
}

impl From<ESetBody> for models::rhoapi::ESet {
    fn from(set: ESetBody) -> Self {
        models::rhoapi::ESet {
            ps: set
                .ps
                .into_iter()
                .map(Into::into)
                .collect::<Vec<models::rhoapi::Par>>(),
            locally_free: set.locally_free.into(),
            connective_used: set.connective_used,
            remainder: set.remainder.map(Into::into),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Default, Hash)]
pub struct EMapBody {
    pub ps: Vec<(Par, Par)>,
    pub locally_free: MyBitVec,
    pub connective_used: bool,
    pub remainder: Option<Var>,
}

impl From<models::rhoapi::EMap> for EMapBody {
    fn from(map: models::rhoapi::EMap) -> Self {
        EMapBody {
            ps: map
                .kvs
                .into_iter()
                .map(|kvp| {
                    (
                        kvp.key.map(Into::into).unwrap(),
                        kvp.value.map(Into::into).unwrap(),
                    )
                })
                .collect::<Vec<(Par, Par)>>(),
            locally_free: MyBitVec::<usize, _>::from_vec(
                map.locally_free.into_iter().map(|v| v as usize).collect(),
            ),
            connective_used: map.connective_used,
            remainder: map.remainder.map(Into::into),
        }
    }
}

impl From<EMapBody> for models::rhoapi::EMap {
    fn from(value: EMapBody) -> Self {
        Self {
            kvs: value
                .ps
                .into_iter()
                .map(|(k, v)| KeyValuePair {
                    key: Some(k.into()),
                    value: Some(v.into()),
                })
                .collect(),
            locally_free: value.locally_free.into(),
            connective_used: value.connective_used,
            remainder: value.remainder.map(Into::into),
        }
    }
}

// #[derive(Debug, PartialEq, Eq, Clone, Copy)]
// pub struct KeyValuePair {
//     pub key: Par,
//     pub value: Par,
// }

/// *
/// `target.method(arguments)`
#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct EMethodBody {
    pub method_name: String,
    pub target: Par,
    pub arguments: Vec<Par>,
    pub locally_free: MyBitVec,
    pub connective_used: bool,
}

impl From<models::rhoapi::EMethod> for EMethodBody {
    fn from(method: models::rhoapi::EMethod) -> Self {
        EMethodBody {
            method_name: method.method_name,
            target: method.target.map(Into::into).unwrap(),
            arguments: method.arguments.into_iter().map(Into::into).collect(),
            locally_free: MyBitVec::from_vec(
                method
                    .locally_free
                    .into_iter()
                    .map(|v| v as usize)
                    .collect(),
            ),
            connective_used: method.connective_used,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct EMatchesBody {
    pub pattern: Par,
    pub target: Par,
}

impl From<models::rhoapi::EMatches> for EMatchesBody {
    fn from(matches: models::rhoapi::EMatches) -> Self {
        EMatchesBody {
            pattern: matches.pattern.map(Into::into).unwrap(),
            target: matches.target.map(Into::into).unwrap(),
        }
    }
}

impl Expr {
    pub fn new_bound_var(idx: u32) -> Expr {
        Expr::EVar(Var::BoundVar(idx))
    }

    pub fn new_free_var(idx: u32) -> Expr {
        Expr::EVar(Var::FreeVar(idx))
    }

    pub const WILDCARD: Expr = Expr::EVar(Var::Wildcard);
    pub const GTRUE: Expr = Expr::GBool(true);
    pub const GFALSE: Expr = Expr::GBool(false);

    pub fn connective_used(&self) -> bool {
        match self {
            Expr::GBool(_)
            | Expr::GInt(_)
            | Expr::GString(_)
            | Expr::GUri(_)
            | Expr::GByteArray(_) => false,

            Expr::EList(e) => e.connective_used,
            Expr::ETuple(e) => e.connective_used,
            Expr::ESet(e) => e.connective_used,
            Expr::EMap(e) => e.connective_used,

            Expr::EVar(v) => v.connective_used(),

            Expr::ENot(p) | Expr::ENeg(p) => p.connective_used,

            Expr::EMult(p1, p2)
            | Expr::EDiv(p1, p2)
            | Expr::EMod(p1, p2)
            | Expr::EPlus(p1, p2)
            | Expr::EMinus(p1, p2)
            | Expr::ELt(p1, p2)
            | Expr::ELte(p1, p2)
            | Expr::EGt(p1, p2)
            | Expr::EGte(p1, p2)
            | Expr::EEq(p1, p2)
            | Expr::ENeq(p1, p2)
            | Expr::EAnd(p1, p2)
            | Expr::EOr(p1, p2)
            | Expr::EPercentPercent(p1, p2)
            | Expr::EPlusPlus(p1, p2)
            | Expr::EMinusMinus(p1, p2) => p1.connective_used || p2.connective_used,

            Expr::EMethod(e) => e.connective_used,
            Expr::EMatches(EMatchesBody { target, .. }) => target.connective_used,
        }
    }

    pub fn locally_free(&self, depth: u32) -> MyBitVec {
        match self {
            Expr::GBool(_)
            | Expr::GInt(_)
            | Expr::GString(_)
            | Expr::GUri(_)
            | Expr::GByteArray(_) => BitVec::EMPTY.into(),

            Expr::EList(e) => e.locally_free.clone(),
            Expr::ETuple(e) => e.locally_free.clone(),
            Expr::ESet(e) => e.locally_free.clone(),
            Expr::EMap(e) => e.locally_free.clone(),

            Expr::EVar(v) => v.locally_free(depth),

            Expr::ENot(p) | Expr::ENeg(p) => p.locally_free.clone(),

            Expr::EMult(p1, p2)
            | Expr::EDiv(p1, p2)
            | Expr::EMod(p1, p2)
            | Expr::EPlus(p1, p2)
            | Expr::EMinus(p1, p2)
            | Expr::ELt(p1, p2)
            | Expr::ELte(p1, p2)
            | Expr::EGt(p1, p2)
            | Expr::EGte(p1, p2)
            | Expr::EEq(p1, p2)
            | Expr::ENeq(p1, p2)
            | Expr::EAnd(p1, p2)
            | Expr::EOr(p1, p2)
            | Expr::EPercentPercent(p1, p2)
            | Expr::EPlusPlus(p1, p2)
            | Expr::EMinusMinus(p1, p2) => union(&p1.locally_free, &p2.locally_free),

            Expr::EMethod(e) => e.locally_free.clone(),
            Expr::EMatches(EMatchesBody { target, .. }) => target.locally_free.clone(),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub enum Connective {
    ConnAnd(Vec<Par>),
    ConnOr(Vec<Par>),
    ConnNot(Par),
    VarRef(VarRef),
    ConnBool(bool),
    ConnInt(bool),
    ConnString(bool),
    ConnUri(bool),
    ConnByteArray(bool),
}

impl From<models::rhoapi::Connective> for Connective {
    fn from(connective: models::rhoapi::Connective) -> Self {
        match connective.connective_instance.unwrap() {
            models::rhoapi::connective::ConnectiveInstance::ConnAndBody(connective_body) => {
                Connective::ConnAnd(connective_body.ps.into_iter().map(Into::into).collect())
            }
            models::rhoapi::connective::ConnectiveInstance::ConnOrBody(connective_body) => {
                Connective::ConnOr(connective_body.ps.into_iter().map(Into::into).collect())
            }
            models::rhoapi::connective::ConnectiveInstance::ConnNotBody(par) => {
                Connective::ConnNot(par.into())
            }
            models::rhoapi::connective::ConnectiveInstance::VarRefBody(var_ref) => {
                Connective::VarRef(var_ref.into())
            }
            models::rhoapi::connective::ConnectiveInstance::ConnBool(v) => Connective::ConnBool(v),
            models::rhoapi::connective::ConnectiveInstance::ConnInt(v) => Connective::ConnInt(v),
            models::rhoapi::connective::ConnectiveInstance::ConnString(v) => {
                Connective::ConnString(v)
            }
            models::rhoapi::connective::ConnectiveInstance::ConnUri(v) => Connective::ConnUri(v),
            models::rhoapi::connective::ConnectiveInstance::ConnByteArray(v) => {
                Connective::ConnByteArray(v)
            }
        }
    }
}

impl From<Connective> for models::rhoapi::Connective {
    fn from(value: Connective) -> Self {
        Self {
            connective_instance: match value {
                Connective::ConnBool(v) => {
                    models::rhoapi::connective::ConnectiveInstance::ConnBool(v).into()
                }
                Connective::ConnInt(v) => {
                    models::rhoapi::connective::ConnectiveInstance::ConnInt(v).into()
                }
                Connective::ConnString(v) => {
                    models::rhoapi::connective::ConnectiveInstance::ConnString(v).into()
                }
                Connective::ConnUri(v) => {
                    models::rhoapi::connective::ConnectiveInstance::ConnUri(v).into()
                }
                Connective::ConnByteArray(v) => {
                    models::rhoapi::connective::ConnectiveInstance::ConnByteArray(v).into()
                }
                Connective::ConnAnd(pars) => {
                    models::rhoapi::connective::ConnectiveInstance::ConnAndBody(
                        models::rhoapi::ConnectiveBody {
                            ps: pars.into_iter().map(Into::into).collect(),
                        },
                    )
                    .into()
                }
                Connective::ConnOr(pars) => {
                    models::rhoapi::connective::ConnectiveInstance::ConnOrBody(
                        models::rhoapi::ConnectiveBody {
                            ps: pars.into_iter().map(Into::into).collect(),
                        },
                    )
                    .into()
                }
                Connective::ConnNot(par) => {
                    models::rhoapi::connective::ConnectiveInstance::ConnNotBody(par.into()).into()
                }
                Connective::VarRef(var_ref) => {
                    models::rhoapi::connective::ConnectiveInstance::VarRefBody(
                        models::rhoapi::VarRef {
                            index: var_ref.index as i32,
                            depth: var_ref.depth as i32,
                        },
                    )
                    .into()
                }
            },
        }
    }
}

impl Connective {
    pub fn connective_used(&self) -> bool {
        match self {
            Connective::VarRef(_) => false,
            _ => true,
        }
    }
    pub fn locally_free(&self, depth: u32) -> MyBitVec {
        match self {
            Connective::VarRef(VarRef {
                index,
                depth: var_depth,
            }) if depth == *var_depth => single_bit(*index as usize),
            _ => BitVec::EMPTY.into(),
        }
    }
}

impl Sortable for Connective {
    type Sorter<'a> = ConnectiveSorter<'a>;

    fn sorter(&mut self) -> Self::Sorter<'_> {
        ConnectiveSorter::new(self)
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct VarRef {
    pub index: u32,
    pub depth: u32,
}

impl From<models::rhoapi::VarRef> for VarRef {
    fn from(var_ref: models::rhoapi::VarRef) -> Self {
        VarRef {
            index: var_ref.index as u32,
            depth: var_ref.depth as u32,
        }
    }
}

impl From<VarRef> for models::rhoapi::VarRef {
    fn from(var_ref: VarRef) -> Self {
        models::rhoapi::VarRef {
            index: var_ref.index as i32,
            depth: var_ref.depth as i32,
        }
    }
}

// pub struct DeployId {
//     pub sig: Vec<u8>,
// }

// pub struct DeployerId {
//     pub public_key: Vec<u8>,
// }

/// Unforgeable names resulting from `new x { ... }`
/// These should only occur as the program is being evaluated. There is no way in
/// the grammar to construct them.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct GUnforgeable<const N: usize>(pub [u8; N]);

impl<const N: usize> From<models::rhoapi::GUnforgeable> for GUnforgeable<N> {
    fn from(unf: models::rhoapi::GUnforgeable) -> Self {
        let mut buf = [0u8; N];

        unf.unf_instance
            .expect("unf_instance can't be None")
            .encode(&mut buf.as_mut_slice());

        GUnforgeable(buf)
    }
}

impl<const N: usize> From<GUnforgeable<N>> for models::rhoapi::GUnforgeable {
    fn from(unf: GUnforgeable<N>) -> Self {
        models::rhoapi::GUnforgeable::decode(&unf.0[..]).unwrap()
    }
}

impl<const N: usize> GUnforgeable<N> {
    pub fn to_bytes_vec(&self) -> Vec<u8> {
        let mut unf_private_data = std::mem::ManuallyDrop::new(self.0.to_vec());
        unsafe {
            Vec::from_raw_parts(
                unf_private_data.as_mut_ptr() as *mut u8,
                32,
                unf_private_data.capacity(),
            )
        }
    }
}

impl<const N: usize> PartialOrd for GUnforgeable<N> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<const N: usize> Ord for GUnforgeable<N> {
    fn cmp(&self, other: &Self) -> Ordering {
        let lhs = &self.0;
        let rhs = &other.0;
        for i in 0..32 {
            match (lhs[i] as u8).cmp(&(rhs[i] as u8)) {
                Ordering::Equal => (),
                non_eq => return non_eq,
            }
        }
        return Ordering::Equal;
    }
}

// #[derive(Debug, PartialEq, Eq)]
// pub struct GPrivate {
//     pub id: Vec<u8>,
// }

// #[derive(Debug, PartialEq, Eq)]
// pub struct GDeployId {
//     pub sig: Vec<u8>,
// }

// #[derive(Debug, PartialEq, Eq)]
// pub struct GDeployerId {
//     pub public_key: Vec<u8>,
// }

// #[derive(Debug, PartialEq, Eq)]
// pub struct GSysAuthToken {}

// helper functions
#[inline]
pub(crate) fn union_inplace(this: &mut MyBitVec, that: &BitSlice) {
    let that_slice = that.as_ref();
    if that_slice.len() == 0 {
        return;
    }
    if this.len() == 0 {
        this.extend_from_bitslice(that_slice);
        return;
    }
    *this.as_mut_bitslice() |= that_slice;
}

#[inline]
pub(crate) fn union(this: impl AsRef<BitSlice>, that: impl AsRef<BitSlice>) -> MyBitVec {
    let this_slice = this.as_ref();
    if this_slice.is_empty() {
        let that_slice = that.as_ref();
        if that_slice.is_empty() {
            return BitVec::new().into();
        }
        return BitVec::from_bitslice(that_slice).into();
    }
    let mut result = BitVec::from_bitslice(this_slice);
    result |= that.as_ref();
    result.into()
}

#[inline]
fn single_bit(pos: usize) -> MyBitVec {
    let mut res = BitVec::repeat(false, pos + 1);
    res.set(pos, true);
    res.into()
}

#[inline]
pub(crate) fn adjust_bitset(slice: &BitSlice, from: usize) -> &BitSlice {
    if from > slice.len() {
        BitSlice::empty()
    } else {
        &slice[from..]
    }
}

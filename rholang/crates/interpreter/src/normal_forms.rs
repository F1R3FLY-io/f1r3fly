use std::{
    cmp::Ordering,
    collections::{BTreeMap, BTreeSet},
};

use bitvec::{slice::BitSlice, vec::BitVec};
use itertools::Itertools;

use super::{
    sort_matcher::{ScoredTerm, Sortable, Sorted},
    sorter::*,
};

#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct Par {
    pub sends: Vec<Send>,
    pub receives: Vec<Receive>,
    pub news: Vec<New>,
    pub exprs: Vec<Expr>,
    pub matches: Vec<Match>,
    pub unforgeables: Vec<GUnforgeable>,
    pub bundles: Vec<Bundle>,
    pub connectives: Vec<Connective>,
    pub locally_free: BitVec,
    pub connective_used: bool,
}

impl Par {
    pub fn concat_with(&mut self, other: Par) {
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
        union_inplace(&mut self.locally_free, e.locally_free(depth));
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
        union_inplace(&mut self.locally_free, &s.locally_free);
        self.connective_used = self.connective_used || s.connective_used;
        self.sends.push(s);
    }

    pub fn push_new(&mut self, new: New) {
        union_inplace(&mut self.locally_free, &new.locally_free);
        self.connective_used = self.connective_used || new.p.connective_used;
        self.news.push(new);
    }

    pub const NIL: Par = Par {
        sends: Vec::new(),
        receives: Vec::new(),
        news: Vec::new(),
        exprs: Vec::new(),
        matches: Vec::new(),
        unforgeables: Vec::new(),
        bundles: Vec::new(),
        connectives: Vec::new(),
        locally_free: BitVec::EMPTY,
        connective_used: false,
    };

    pub fn gtrue() -> Par {
        Par {
            exprs: vec![Expr::GTRUE],
            ..Default::default()
        }
    }

    pub fn gfalse() -> Par {
        Par {
            exprs: vec![Expr::GFALSE],
            ..Default::default()
        }
    }

    pub fn gint(value: i64) -> Par {
        Par {
            exprs: vec![Expr::GInt(value)],
            ..Default::default()
        }
    }

    pub fn gstr(value: String) -> Par {
        Par {
            exprs: vec![Expr::GString(value)],
            ..Default::default()
        }
    }

    pub fn wild() -> Par {
        Par {
            exprs: vec![Expr::WILDCARD],
            connective_used: true,
            ..Default::default()
        }
    }

    pub fn bound_var(idx: u32) -> Par {
        Par {
            exprs: vec![Expr::new_bound_var(idx)],
            locally_free: single_bit(idx as usize),
            ..Default::default()
        }
    }

    pub fn free_var(idx: u32) -> Par {
        Par {
            exprs: vec![Expr::new_free_var(idx)],
            connective_used: true,
            ..Default::default()
        }
    }

    pub fn free_vars(n: u32) -> Vec<Par> {
        (0..n).map(Par::free_var).collect()
    }

    pub fn bound_vars(n: u32) -> Vec<Par> {
        (0..n).rev().map(Par::bound_var).collect_vec()
    }

    pub fn elist(body: EListBody) -> Par {
        let connective_used = body.connective_used;
        let locally_free = body.locally_free.clone();
        Par {
            exprs: vec![Expr::EList(body)],
            locally_free,
            connective_used,
            ..Default::default()
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
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Var {
    BoundVar(u32),
    FreeVar(u32),
    Wildcard,
}

impl Var {
    pub fn connective_used(&self) -> bool {
        match self {
            Var::BoundVar(_) => false,
            _ => true,
        }
    }

    pub fn locally_free(&self, depth: u32) -> BitVec {
        match self {
            Var::BoundVar(index) if depth == 0 => single_bit(*index as usize),
            _ => BitVec::EMPTY,
        }
    }
}

/// *
/// Nothing can be received from a (quoted) bundle with `readFlag = false`.
/// Likewise nothing can be sent to a (quoted) bundle with `writeFlag = false`.
///
/// If both flags are set to false, bundle allows only for equivalance check.
#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct Bundle {
    pub body: Par,
    /// flag indicating whether bundle is writeable
    pub write_flag: bool,
    /// flag indicating whether bundle is readable
    pub read_flag: bool,
}

impl Sortable for Bundle {
    type Sorter<'a> = BundleSorter<'a>;

    fn sorter(&mut self) -> Self::Sorter<'_> {
        BundleSorter::new(self)
    }
}

/// *
/// A send is written `chan!(data)` or `chan!!(data)` for a persistent send.
///
/// Upon send, all free variables in data are substituted with their values.
#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct Send {
    pub chan: Par,
    pub data: Vec<Par>,
    pub persistent: bool,
    pub locally_free: BitVec,
    pub connective_used: bool,
}

impl Sortable for Send {
    type Sorter<'a> = SendSorter<'a>;

    fn sorter(&mut self) -> Self::Sorter<'_> {
        SendSorter::new(self)
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ReceiveBind {
    pub patterns: Vec<Par>,
    pub source: Par,
    pub remainder: Option<Var>,
    pub free_count: u32,
}

#[derive(Debug, PartialEq, Eq, Clone)]
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
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Receive {
    pub binds: Vec<ReceiveBind>,
    pub body: Par,
    pub persistent: bool,
    pub peek: bool,
    pub bind_count: u32,
    pub locally_free: BitVec,
    pub connective_used: bool,
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
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct New {
    /// Includes any uris listed below. This makes it easier to substitute or walk
    /// a term.
    pub bind_count: u32,
    pub p: Par,
    /// For normalization, uri-referenced variables come at the end, and in
    /// lexicographical order.
    pub uris: Vec<String>,

    pub locally_free: BitVec,
}

impl Sortable for New {
    type Sorter<'a> = NewSorter<'a>;

    fn sorter(&mut self) -> Self::Sorter<'_> {
        NewSorter::new(self)
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct MatchCase {
    pub pattern: Par,
    pub source: Par,
    pub free_count: u32,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Match {
    pub target: Par,
    pub cases: Vec<MatchCase>,
    pub locally_free: BitVec,
    pub connective_used: bool,
}

impl Sortable for Match {
    type Sorter<'a> = MatchSorter<'a>;

    fn sorter(&mut self) -> Self::Sorter<'_> {
        MatchSorter::new(self)
    }
}

/// Any process may be an operand to an expression.
/// Only processes equivalent to a ground process of compatible type will reduce.
#[derive(Debug, PartialEq, Eq, Clone)]
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

impl Sortable for Expr {
    type Sorter<'a> = ExprSorter<'a>;

    fn sorter(&mut self) -> Self::Sorter<'_> {
        ExprSorter::new(self)
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct EListBody {
    pub ps: Vec<Par>,
    pub locally_free: BitVec,
    pub connective_used: bool,
    pub remainder: Option<Var>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ETupleBody {
    pub ps: Vec<Par>,
    pub locally_free: BitVec,
    pub connective_used: bool,
}

#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct ESetBody {
    pub ps: BTreeSet<ScoredTerm<Par>>,
    pub locally_free: BitVec,
    pub connective_used: bool,
    pub remainder: Option<Var>,
}

#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct EMapBody {
    pub ps: BTreeMap<ScoredTerm<Par>, Sorted<Par>>,
    pub locally_free: BitVec,
    pub connective_used: bool,

    pub remainder: Option<Var>,
}

// #[derive(Debug, PartialEq, Eq, Clone, Copy)]
// pub struct KeyValuePair {
//     pub key: Par,
//     pub value: Par,
// }

/// *
/// `target.method(arguments)`
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct EMethodBody {
    pub method_name: String,
    pub target: Par,
    pub arguments: Vec<Par>,
    pub locally_free: BitVec,
    pub connective_used: bool,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct EMatchesBody {
    pub pattern: Par,
    pub target: Par,
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

    pub fn locally_free(&self, depth: u32) -> BitVec {
        match self {
            Expr::GBool(_)
            | Expr::GInt(_)
            | Expr::GString(_)
            | Expr::GUri(_)
            | Expr::GByteArray(_) => BitVec::EMPTY,

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

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Connective {
    ConnAnd(Vec<Par>),
    ConnOr(Vec<Par>),
    ConnNot(Par),
    VarRef(VarRef),
    ConnBool,
    ConnInt,
    ConnString,
    ConnUri,
    ConnByteArray,
}

impl Connective {
    pub fn connective_used(&self) -> bool {
        match self {
            Connective::VarRef(_) => false,
            _ => true,
        }
    }
    pub fn locally_free(&self, depth: u32) -> BitVec {
        match self {
            Connective::VarRef(VarRef {
                index,
                depth: var_depth,
            }) if depth == *var_depth => single_bit(*index as usize),
            _ => BitVec::EMPTY,
        }
    }
}

impl Sortable for Connective {
    type Sorter<'a> = ConnectiveSorter<'a>;

    fn sorter(&mut self) -> Self::Sorter<'_> {
        ConnectiveSorter::new(self)
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct VarRef {
    pub index: u32,
    pub depth: u32,
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
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct GUnforgeable(pub [i8; 32]);

impl GUnforgeable {
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

impl PartialOrd for GUnforgeable {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for GUnforgeable {
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
pub(crate) fn union_inplace(this: &mut BitVec, that: impl AsRef<BitSlice>) {
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
pub(crate) fn union(this: impl AsRef<BitSlice>, that: impl AsRef<BitSlice>) -> BitVec {
    let this_slice = this.as_ref();
    if this_slice.is_empty() {
        let that_slice = that.as_ref();
        if that_slice.is_empty() {
            return BitVec::new();
        }
        return BitVec::from_bitslice(that_slice);
    }
    let mut result = BitVec::from_bitslice(this_slice);
    result |= that.as_ref();
    return result;
}

#[inline]
fn single_bit(pos: usize) -> BitVec {
    let mut res = BitVec::repeat(false, pos + 1);
    res.set(pos, true);
    res
}

#[inline]
pub(crate) fn adjust_bitset(slice: &BitSlice, from: usize) -> &BitSlice {
    if from > slice.len() {
        BitSlice::empty()
    } else {
        &slice[from..]
    }
}

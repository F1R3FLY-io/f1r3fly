use std::{cell::RefCell, cmp::Ordering, num::NonZeroUsize, ops::ControlFlow, rc::Rc};

use indextree::{Arena, Descendants, NodeEdge, NodeId, Traverse};
use smallvec::SmallVec;

#[derive(Debug, Clone)]
pub struct ScoreTree {
    root: NodeId,
    indextree: Rc<RefCell<Arena<Atom>>>,
}

impl ScoreTree {
    pub fn traverse<'a>(&'a self) -> impl Iterator<Item = VisitEvent<'a>> {
        // There is currently no way to get a &'a T reference from RefCell, except this. The danger
        // is that a builder might be modifying the tree inside this iterator. The
        // `ScoreBuilder::to_tree` function consumes the builder, so this is not easy to achieve,
        // but consider that any builder might be forked, so the following call-chain is potentially
        // a problem:
        //
        //      builder -> fork -> ... -> toTree -> traverse { use builder }
        //
        // This could be disallowed by using a phantom type parameter on `ScoreTree`, e.g.
        // ScoreTree<{Own, Foreign}>, but let's just not complicate it further.
        let borrowed = unsafe {
            self.indextree
                .try_borrow_unguarded()
                .expect("unexpected borrow error")
        };
        TraverseWrapper::new(self.root, borrowed)
    }

    pub fn descendants<'a>(&'a self) -> impl Iterator<Item = TaggedAtom<'a>> {
        // See the comment in `traverse`
        let borrowed = unsafe {
            self.indextree
                .try_borrow_unguarded()
                .expect("unexpected borrow error")
        };
        DescendantsWrapper::new(self.root, borrowed)
    }

    pub fn count(&self) -> usize {
        self.indextree.borrow().count() // This also count removed nodes, but we never remove nodes.
    }
}

impl PartialEq for ScoreTree {
    fn eq(&self, other: &Self) -> bool {
        if self.count() != other.count() {
            return false;
        }
        self.descendants()
            .zip(other.descendants())
            .all(|(this, that)| this == that)
    }
}

impl Eq for ScoreTree {}

impl PartialOrd for ScoreTree {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ScoreTree {
    fn cmp(&self, other: &Self) -> Ordering {
        fn cf(result: Ordering) -> ControlFlow<Ordering> {
            match result {
                Ordering::Equal => ControlFlow::Continue(()),
                other => ControlFlow::Break(other),
            }
        }

        let r = self
            .descendants()
            .zip(other.descendants())
            .try_for_each(|pair| {
                cf(match pair {
                    (TaggedAtom::Int(this), TaggedAtom::Int(that)) => this.cmp(&that),
                    (TaggedAtom::Str(this), TaggedAtom::Str(that)) => this.cmp(that),
                    (TaggedAtom::U8(this), TaggedAtom::U8(that)) => this.cmp(that),
                    (TaggedAtom::Score(this), TaggedAtom::Score(that)) => this.cmp(&that),
                    (TaggedAtom::Score(_), _ /*leaf*/) => Ordering::Greater,
                    (TaggedAtom::Int(_), _ /*leaf or node*/) => Ordering::Less,
                    (TaggedAtom::Str(_), TaggedAtom::Int(_)) => Ordering::Greater,
                    (TaggedAtom::Str(_), _ /*u8 or node*/) => Ordering::Less,
                    (TaggedAtom::U8(_), TaggedAtom::Score(_)) => Ordering::Less,
                    (TaggedAtom::U8(_), _ /*int or str*/) => Ordering::Greater,
                })
            });
        match r {
            ControlFlow::Continue(_) => self.count().cmp(&other.count()),
            ControlFlow::Break(order) => order,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
enum Atom {
    Int(i64),
    String(String),
    Bytes(Vec<u8>),
    Score(i32),
}

pub trait ScoreBuilder {
    fn append_new_node(&mut self, data: Atom) -> NodeId;
    fn descend(&mut self, path: NodeId);
    fn ascend(&mut self) -> Option<NodeId>;
    fn fork(&self) -> Self;
    fn attach(&mut self, tree: ScoreTree);
    fn focus(&mut self, loc: NodeId);
    fn to_tree(self) -> ScoreTree;

    fn begin(&mut self, score: i32) -> NodeId {
        let new = self.append_new_node(Atom::Score(score));
        self.descend(new);
        return new;
    }

    fn leaf_int(&mut self, value: i64) -> NodeId {
        self.append_new_node(Atom::Int(value))
    }

    fn leaf_bool(&mut self, value: bool) -> NodeId {
        self.leaf_int(if value { 1 } else { 0 })
    }

    fn leaf_string(&mut self, value: String) -> NodeId {
        self.append_new_node(Atom::String(value))
    }

    fn leaf_bytes(&mut self, value: Vec<u8>) -> NodeId {
        self.append_new_node(Atom::Bytes(value))
    }

    fn done(&mut self) {
        self.ascend();
    }

    fn graft(&mut self, tree: &ScoreTree) -> Option<NodeId> {
        let mut result = None;
        tree.traverse().for_each(|event| match event {
            VisitEvent::Enter(score) => {
                self.begin(score);
            }
            VisitEvent::Leave => {
                result = self.ascend();
            }
            VisitEvent::VisitInt(value) => {
                self.leaf_int(value);
            }
            VisitEvent::VisitStr(value) => {
                self.leaf_string(value.to_owned());
            }
            VisitEvent::VisitU8(value) => {
                self.leaf_bytes(value.to_owned());
            }
        });
        return result;
    }
}

pub(crate) struct ArenaBuilder {
    index: Rc<RefCell<Arena<Atom>>>,
    root_id: NonZeroUsize,
    path: SmallVec<[NodeId; 4]>,
}

impl ArenaBuilder {
    pub(crate) fn new() -> ArenaBuilder {
        ArenaBuilder {
            index: Rc::new(RefCell::new(Arena::with_capacity(10))),
            root_id: NonZeroUsize::MIN,
            path: SmallVec::new(),
        }
    }
}

impl ScoreBuilder for ArenaBuilder {
    fn append_new_node(&mut self, data: Atom) -> NodeId {
        let mut index = self.index.borrow_mut();
        let new_node = index.new_node(data);
        if let Some(current) = self.path.last() {
            current.append(new_node, &mut index);
        }
        new_node
    }

    fn to_tree(self) -> ScoreTree {
        let index = self.index.borrow();
        let root = index
            .get_node_id_at(self.root_id)
            .expect("expected non-empty ScoreTree");
        ScoreTree {
            indextree: self.index.clone(),
            root,
        }
    }

    fn descend(&mut self, path: NodeId) {
        self.path.push(path);
    }

    fn ascend(&mut self) -> Option<NodeId> {
        self.path.pop()
    }

    fn fork(&self) -> Self {
        ArenaBuilder {
            index: self.index.clone(),
            root_id: unsafe { NonZeroUsize::new_unchecked(self.index.borrow().count() + 1) },
            path: SmallVec::new(),
        }
    }

    fn attach(&mut self, tree: ScoreTree) {
        if let Some(current) = self.path.last() {
            let mut index = self.index.borrow_mut();
            current.append(tree.root, &mut index);
        }
    }

    fn focus(&mut self, loc: NodeId) {
        self.path.clear();
        self.path.push(loc);
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum VisitEvent<'a> {
    Enter(i32),
    Leave,
    VisitInt(i64),
    VisitStr(&'a str),
    VisitU8(&'a [u8]),
}

#[derive(Debug, PartialEq, Eq)]
pub enum TaggedAtom<'a> {
    Score(i32),
    Int(i64),
    Str(&'a str),
    U8(&'a [u8]),
}

struct TraverseWrapper<'a> {
    wrapped: Traverse<'a, Atom>,
    arena: &'a Arena<Atom>,
    skip_next_end: bool,
}

impl<'a> TraverseWrapper<'a> {
    fn new(start_id: NodeId, arena: &Arena<Atom>) -> TraverseWrapper<'_> {
        TraverseWrapper {
            wrapped: start_id.traverse(arena),
            arena,
            skip_next_end: false,
        }
    }

    fn get_atom(&self, id: NodeId) -> &'a Atom {
        self.arena[id].get()
    }
}

impl<'a> Iterator for TraverseWrapper<'a> {
    type Item = VisitEvent<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        self.wrapped.next().and_then(|e| match e {
            NodeEdge::Start(id) => {
                let a = self.get_atom(id);
                match a {
                    Atom::Int(value) => {
                        self.skip_next_end = true;
                        Some(VisitEvent::VisitInt(*value))
                    }
                    Atom::String(value) => {
                        self.skip_next_end = true;
                        Some(VisitEvent::VisitStr(value))
                    }
                    Atom::Bytes(value) => {
                        self.skip_next_end = true;
                        Some(VisitEvent::VisitU8(value))
                    }
                    Atom::Score(score) => Some(VisitEvent::Enter(*score)),
                }
            }
            NodeEdge::End(_) if !self.skip_next_end => Some(VisitEvent::Leave),
            _ => {
                self.skip_next_end = false;
                self.next()
            }
        })
    }
}

struct DescendantsWrapper<'a> {
    wrapped: Descendants<'a, Atom>,
    arena: &'a Arena<Atom>,
}

impl<'a> DescendantsWrapper<'a> {
    fn new(start_id: NodeId, arena: &Arena<Atom>) -> DescendantsWrapper<'_> {
        DescendantsWrapper {
            wrapped: start_id.descendants(arena),
            arena,
        }
    }

    fn get_atom(&self, id: NodeId) -> &'a Atom {
        self.arena[id].get()
    }
}

impl<'a> Iterator for DescendantsWrapper<'a> {
    type Item = TaggedAtom<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        self.wrapped.next().map(|e| {
            let a = self.get_atom(e);
            match a {
                Atom::Int(value) => TaggedAtom::Int(*value),
                Atom::Score(score) => TaggedAtom::Score(*score),
                Atom::String(value) => TaggedAtom::Str(value),
                Atom::Bytes(value) => TaggedAtom::U8(value),
            }
        })
    }
}

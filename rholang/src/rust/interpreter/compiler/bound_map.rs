use indexmap::IndexMap;

use super::{
    exports::*,
    free_map::VarSort,
    rholang_ast::{Id, Uri},
};

#[derive(Debug, PartialEq, Eq)]
pub struct BoundMap(IndexMap<String, Context<BoundContext>>);

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BoundContext {
    Proc,
    Name(Option<String>),
}

impl BoundMap {
    pub fn new() -> Self {
        BoundMap(IndexMap::new())
    }

    pub fn get(&self, name: &str) -> Option<(u32, &Context<BoundContext>)> {
        self.0
            .get_full(name)
            .map(|(level, _, ctx)| (self.next_index() - (level - 1) as u32, ctx))
    }

    pub fn put(
        &mut self,
        binding: (String, Context<BoundContext>),
    ) -> Option<Context<BoundContext>> {
        let (name, ctx) = binding;
        self.0.insert(name, ctx)
    }

    pub fn absorb_free(&mut self, free_map: FreeMap) {
        for (name, context) in free_map {
            self.0.insert(
                name,
                Context {
                    item: match context.item {
                        VarSort::NameSort => BoundContext::Name(None),
                        VarSort::ProcSort => BoundContext::Proc,
                    },
                    source_position: context.source_position,
                },
            );
        }
    }

    pub fn shift(&mut self, n: usize) {
        let m = self.0.len();
        if n >= m {
            self.0.clear();
        } else {
            self.0.truncate(m - n);
        }
    }

    // Rename this method to avoid conflict with Iterator::count
    pub fn count(&self) -> u32 {
        self.next_index()
    }

    pub fn next_index(&self) -> u32 {
        self.0.len() as u32
    }

    pub fn iter(
        &self,
    ) -> impl Iterator<Item = (&String, &Context<BoundContext>)> + ExactSizeIterator {
        self.0.iter()
    }

    pub fn put_id_as_proc(&mut self, id: &Id) -> Option<Context<BoundContext>> {
        self.put((
            id.name.to_owned(),
            Context {
                item: BoundContext::Proc,
                source_position: id.pos,
            },
        ))
    }

    pub fn put_id_as_name(&mut self, id: &Id) -> Option<Context<BoundContext>> {
        self.put((
            id.name.to_owned(),
            Context {
                item: BoundContext::Name(None),
                source_position: id.pos,
            },
        ))
    }

    pub fn put_name(&mut self, id: &Id, uri: &Option<Uri>) -> Option<Context<BoundContext>> {
        self.put((
            id.name.to_owned(),
            Context {
                item: BoundContext::Name(uri.map(|val| val.to_string())),
                source_position: id.pos,
            },
        ))
    }
}

impl IntoIterator for BoundMap {
    type Item = (String, Context<BoundContext>);
    type IntoIter = indexmap::map::IntoIter<String, Context<BoundContext>>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a> IntoIterator for &'a BoundMap {
    type Item = (&'a String, &'a Context<BoundContext>);
    type IntoIter = indexmap::map::Iter<'a, String, Context<BoundContext>>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

use std::fmt::Display;

use indexmap::IndexMap;

use super::{exports::*, rholang_ast::Id};

#[derive(Clone, Debug, PartialEq, Eq, Copy)]
pub enum VarSort {
    ProcSort,
    NameSort,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct FreeMap {
    level_bindings: IndexMap<String, Context<VarSort>>,
    wildcards: Vec<SourcePosition>,
    connectives: Vec<Context<ConnectiveInstance>>,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum ConnectiveInstance {
    And,
    Or,
    Not,
}

impl Display for ConnectiveInstance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConnectiveInstance::And => f.write_str("/\\ (conjunction)"),
            ConnectiveInstance::Or => f.write_str("\\/ (disjunction)"),
            ConnectiveInstance::Not => f.write_str("~ (negation)"),
        }
    }
}

impl FreeMap {
    pub fn new() -> Self {
        FreeMap {
            level_bindings: IndexMap::new(),
            wildcards: Vec::new(),
            connectives: Vec::new(),
        }
    }

    pub fn get(&self, name: &str) -> Option<(usize, &Context<VarSort>)> {
        self.level_bindings
            .get_full(name)
            .map(|(level, _, context)| (level, context))
    }

    pub fn get_by_index(&self, level: usize) -> Option<(&String, &Context<VarSort>)> {
        self.level_bindings.get_index(level)
    }

    pub fn put(&mut self, binding: (String, Context<VarSort>)) -> Result<u32, Context<VarSort>> {
        let (name, context) = binding;
        let maybe_old = self.level_bindings.insert(name, context);
        match maybe_old {
            Some(old) => Err(old),
            None => Ok(self.next_level() - 1),
        }
    }

    // Returns a list of the shadowed variables
    pub fn merge(&mut self, free_map: FreeMap) -> Vec<Context<usize>> {
        let mut shadowed = Vec::new();
        for (name, new_free) in free_map.level_bindings {
            let bindings = &mut self.level_bindings;
            let (idx, maybe_old_free) = bindings.insert_full(name, new_free);
            if let Some(old_free) = maybe_old_free {
                shadowed.push(Context {
                    item: idx,
                    source_position: old_free.source_position,
                })
            }
        }

        self.wildcards.extend(&free_map.wildcards);
        self.connectives.extend(&free_map.connectives);
        return shadowed;
    }

    pub(crate) fn add_wildcard(&mut self, source_position: SourcePosition) {
        self.wildcards.push(source_position);
    }

    pub fn remember_connective(
        &mut self,
        connective_instance: ConnectiveInstance,
        source_position: SourcePosition,
    ) {
        self.connectives.push(Context {
            item: connective_instance,
            source_position,
        });
    }

    pub fn count(&self) -> u32 {
        self.count_no_wildcards() + ((self.wildcards.len() + self.connectives.len()) as u32)
    }

    pub fn count_no_wildcards(&self) -> u32 {
        self.level_bindings.len() as u32
    }

    pub fn next_level(&self) -> u32 {
        self.level_bindings.len() as u32
    }

    pub fn has_wildcards(&self) -> bool {
        !self.wildcards.is_empty()
    }
    pub fn has_connectives(&self) -> bool {
        !self.connectives.is_empty()
    }
    pub fn is_empty(&self) -> bool {
        self.count() == 0
    }

    pub fn iter_wildcards(&self) -> impl Iterator<Item = SourcePosition> + ExactSizeIterator + '_ {
        self.wildcards.iter().copied()
    }

    pub fn iter_connectives(
        &self,
    ) -> impl Iterator<Item = Context<ConnectiveInstance>> + ExactSizeIterator + '_ {
        self.connectives.iter().copied()
    }

    pub fn iter(&self) -> impl Iterator<Item = (&String, &Context<VarSort>)> + ExactSizeIterator {
        self.level_bindings.iter()
    }
    pub fn iter_free_vars(&self) -> impl Iterator<Item = Context<String>> + ExactSizeIterator + '_ {
        self.iter().map(|(name, ctx)| Context {
            item: name.to_owned(),
            source_position: ctx.source_position,
        })
    }

    pub fn as_free_vec(&self) -> Vec<(&str, Context<VarSort>)> {
        self.level_bindings
            .iter()
            .map(|(key, context)| (key.as_str(), *context))
            .collect()
    }

    pub fn clear(&mut self) {
        self.level_bindings.clear();
        self.wildcards.clear();
        self.connectives.clear();
    }

    pub fn put_id_in_proc_context(&mut self, id: &Id) -> Result<u32, Context<VarSort>> {
        self.put((
            id.name.to_owned(),
            Context {
                item: VarSort::ProcSort,
                source_position: id.pos,
            },
        ))
    }

    pub fn put_id_in_name_context(&mut self, id: &Id) -> Result<u32, Context<VarSort>> {
        self.put((
            id.name.to_owned(),
            Context {
                item: VarSort::NameSort,
                source_position: id.pos,
            },
        ))
    }
}

impl Default for FreeMap {
    fn default() -> Self {
        Self::new()
    }
}

impl IntoIterator for FreeMap {
    type Item = (String, Context<VarSort>);
    type IntoIter = indexmap::map::IntoIter<String, Context<VarSort>>;

    fn into_iter(self) -> Self::IntoIter {
        self.level_bindings.into_iter()
    }
}

impl<'a> IntoIterator for &'a FreeMap {
    type Item = (&'a String, &'a Context<VarSort>);
    type IntoIter = indexmap::map::Iter<'a, String, Context<VarSort>>;

    fn into_iter(self) -> Self::IntoIter {
        self.level_bindings.iter()
    }
}

use super::exports::*;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub struct FreeMap<T: Clone> {
    pub next_level: usize,
    pub level_bindings: HashMap<String, FreeContext<T>>,
    pub wildcards: Vec<SourcePosition>,
    pub connectives: Vec<(ConnectiveInstance, SourcePosition)>,
}

impl<T: Clone> FreeMap<T> {
    pub fn new() -> Self {
        FreeMap {
            next_level: 0,
            level_bindings: HashMap::new(),
            wildcards: Vec::new(),
            connectives: Vec::new(),
        }
    }

    pub(crate) fn get(&self, name: &str) -> Option<FreeContext<T>>
    where
        T: Clone,
    {
        self.level_bindings.get(name).cloned()
    }

    pub fn put(&self, binding: IdContext<T>) -> Self {
        let (name, typ, source_position) = binding;

        let mut new_level_bindings = self.level_bindings.clone();
        new_level_bindings.insert(
            name,
            FreeContext {
                level: self.next_level,
                typ,
                source_position,
            },
        );

        FreeMap {
            next_level: self.next_level + 1,
            level_bindings: new_level_bindings,
            wildcards: self.wildcards.clone(),
            connectives: self.connectives.clone(),
        }
    }

    pub fn put_all(&self, bindings: Vec<IdContext<T>>) -> Self {
        let mut new_free_map = self.clone();
        for binding in bindings {
            new_free_map = new_free_map.put(binding);
        }
        new_free_map
    }

    // Returns the new map, and a list of the shadowed variables
    pub fn merge(&self, free_map: FreeMap<T>) -> (FreeMap<T>, Vec<(String, SourcePosition)>) {
        let (acc_env, shadowed) = free_map.level_bindings.into_iter().fold(
            (self.level_bindings.clone(), Vec::new()),
            |(mut acc_env, mut shadowed), (name, free_context)| {
                acc_env.insert(
                    name.clone(),
                    FreeContext {
                        level: free_context.level + self.next_level,
                        typ: free_context.typ,
                        source_position: free_context.source_position.clone(),
                    },
                );

                (acc_env, {
                    if self.level_bindings.contains_key(&name) {
                        shadowed.insert(0, (name, free_context.source_position));
                        shadowed
                    } else {
                        shadowed
                    }
                })
            },
        );

        let mut new_wildcards = self.wildcards.clone();
        new_wildcards.extend(free_map.wildcards.into_iter());
        let mut new_connectives = self.connectives.clone();
        new_connectives.extend(free_map.connectives);

        (
            FreeMap {
                next_level: self.next_level + free_map.next_level,
                level_bindings: acc_env,
                wildcards: new_wildcards,
                connectives: new_connectives,
            },
            shadowed,
        )
    }

    pub(crate) fn add_wildcard(&self, source_position: SourcePosition) -> Self {
        let mut updated_wildcards = self.wildcards.clone();
        updated_wildcards.push(source_position);

        FreeMap {
            next_level: self.next_level,
            level_bindings: self.level_bindings.clone(),
            wildcards: updated_wildcards,
            connectives: self.connectives.clone(),
        }
    }

    pub fn add_connective(
        &self,
        connective: ConnectiveInstance,
        source_position: SourcePosition,
    ) -> Self {
        let mut updated_connectives = self.connectives.clone();
        updated_connectives.push((connective, source_position));

        FreeMap {
            next_level: self.next_level,
            level_bindings: self.level_bindings.clone(),
            wildcards: self.wildcards.clone(),
            connectives: updated_connectives,
        }
    }

    pub fn count(&self) -> usize {
        self.next_level + self.wildcards.len() + self.connectives.len()
    }

    pub fn count_no_wildcards(&self) -> usize {
        self.next_level
    }
}

impl<T: Clone> Default for FreeMap<T> {
    fn default() -> Self {
        Self::new()
    }
}

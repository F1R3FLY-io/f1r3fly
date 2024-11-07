use super::exports::*;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub struct BoundMap<T> {
    next_index: usize,
    index_bindings: HashMap<String, BoundContext<T>>,
}

impl<T: Clone> BoundMap<T> {
    pub fn new() -> Self {
        BoundMap {
            next_index: 0,
            index_bindings: HashMap::new(),
        }
    }

    pub fn get(&self, name: &str) -> Option<BoundContext<T>>
    where
        T: Clone,
    {
        self.index_bindings.get(name).map(|context| BoundContext {
            index: self.next_index - context.index - 1,
            typ: context.typ.clone(),
            source_position: context.source_position.clone(),
        })
    }

    pub fn put(&self, binding: IdContext<T>) -> BoundMap<T> {
        let (name, typ, source_position) = binding;
        let mut new_bindings = self.index_bindings.clone();
        new_bindings.insert(
            name,
            BoundContext {
                index: self.next_index,
                typ,
                source_position,
            },
        );
        BoundMap {
            next_index: self.next_index + 1,
            index_bindings: new_bindings,
        }
    }

    pub fn put_all(&self, bindings: Vec<IdContext<T>>) -> BoundMap<T> {
        let mut new_map = self.clone();
        for binding in bindings {
            new_map = new_map.put(binding);
        }
        new_map
    }

    pub fn absorb_free(&self, free_map: FreeMap<T>) -> BoundMap<T> {
        let mut new_bindings = self.index_bindings.clone();
        for (name, context) in free_map.level_bindings {
            new_bindings.insert(
                name,
                BoundContext {
                    index: context.level + self.next_index,
                    typ: context.typ,
                    source_position: context.source_position,
                },
            );
        }
        BoundMap {
            next_index: self.next_index + free_map.next_level,
            index_bindings: new_bindings,
        }
    }

    // Rename this method to avoid conflict with Iterator::count
    pub fn get_count(&self) -> usize {
        self.next_index
    }
}

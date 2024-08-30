use std::collections::HashMap;
use super::exports::*;

#[derive(Debug, Clone)]
pub struct BoundMap<T> {
  next_index: usize,
  index_bindings: HashMap<String, BoundContext<T>>,
}

impl<T> BoundMap<T> {
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

  pub fn put(&mut self, binding: IdContext<T>) {
    let (name, typ, source_position) = binding;
    self.index_bindings.insert(
      name,
      BoundContext {
        index: self.next_index,
        typ,
        source_position,
      },
    );
    self.next_index += 1;
  }

  pub fn put_all(&mut self, bindings: Vec<IdContext<T>>) {
    for binding in bindings {
      self.put(binding);
    }
  }

  pub fn absorb_free(&mut self, free_map: FreeMap<T>) {
    for (name, context) in free_map.level_bindings {
      self.index_bindings.insert(
        name,
        BoundContext {
          index: context.level + self.next_index,
          typ: context.typ,
          source_position: context.source_position,
        },
      );
    }
    self.next_index += free_map.next_level;
  }

  pub fn count(&self) -> usize {
    self.next_index
  }
}
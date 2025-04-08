use std::fmt::Display;

use exports::SourcePosition;

pub mod bound_map;
pub mod bound_map_chain;
pub mod compiler;
pub mod exports;
pub mod free_map;
pub mod normalize;
pub mod normalizer;
pub mod receive_binds_sort_matcher;
pub mod rholang_ast;
pub mod source_position;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct Context<T> {
    pub item: T,
    pub source_position: SourcePosition,
}

impl<T: Display> Display for Context<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "`{}` at {}", self.item, self.source_position)
    }
}

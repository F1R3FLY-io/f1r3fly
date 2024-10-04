use models::rhoapi::{EList, ETuple};
use super::exports::*;
use tree_sitter::Node;
use std::error::Error;
use std::result::Result;
use std::collections::HashSet;
use models::rust::par_map::ParMap;
use models::rust::par_set::ParSet;

pub fn normalize_collection(
  node: Node,
  input: CollectVisitInputs,
  source_code: &[u8],
) -> Result<CollectVisitOutputs, Box<dyn Error>> {
    println!("Normalizing bundle node of kind: {}", node.kind());

    todo!()
}

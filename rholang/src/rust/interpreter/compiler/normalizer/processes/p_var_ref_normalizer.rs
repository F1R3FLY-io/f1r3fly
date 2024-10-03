use super::exports::*;
use std::error::Error;
use std::result::Result;
use tree_sitter::Node;

/*

*/
pub fn normalize_p_var_ref(
    node: Node,
    input: ProcVisitInputs,
    source_code: &[u8],
) -> Result<ProcVisitOutputs, Box<dyn Error>> {
    println!("Normalizing bundle node of kind: {}", node.kind());

    todo!()
}

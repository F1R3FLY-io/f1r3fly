use blake2::{
    digest::{Update, VariableOutput},
    Blake2bVar,
};
use rspace_plus_plus::rhotypes::rhotypes::SortedSetElement;
use std::error::Error;

// Playground
fn main() -> Result<(), Box<dyn Error>> {
    let protobuf = SortedSetElement { value: 24 };
    let bytes = bincode::serialize(&protobuf).unwrap();

    let mut hasher = Blake2bVar::new(32).unwrap();
    hasher.update(&bytes);
    let hash = hasher.finalize_boxed();
    let hash_hex: String = hash.iter().map(|byte| format!("{:02x}", byte)).collect();

    println!("\nSerialized bytes: {:?}", bytes);
    println!("\nHash (hex): {}", hash_hex);

    Ok(())
}

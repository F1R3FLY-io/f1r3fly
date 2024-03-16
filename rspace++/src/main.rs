use blake2::{digest::consts::U32, Blake2b, Digest};
use hex::ToHex;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let mut hasher = Blake2b::<U32>::new();
    hasher.update(b"hello world");
    let hash = hasher.finalize();
    hash.to_vec();

    let hex_hash = hash.encode_hex::<String>();
    println!("\nHash: {:?}", hex_hash);

    Ok(())
}

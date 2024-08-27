use super::blake2b512_block::Blake2b512Block;

/** Blake2b512 based splittable and mergeable random number generator
 * specialized for generating 256-bit unforgeable names.
 * splitByte and splitShort are the interfaces to make the random number
 * generator diverge.
 * Blake2b512.merge uses online tree hashing to merge two random generator
 * states.
 *
 * TODO: REVIEW
 */

// See crypto/src/main/scala/coop/rchain/crypto/hash/Blake2b512Random.scala

#[derive(Clone)]
pub struct Blake2b512Random {
    digest: Blake2b512Block,
    last_block: Vec<u8>,
    path_view: Vec<u8>,
    count_view: [u64; 2],
    hash_array: [u8; 64],
    position: usize,
}

impl Blake2b512Random {
    pub fn create(init: &[u8], offset: usize, length: usize) -> Blake2b512Random {
        let mut result = Blake2b512Random {
            digest: Blake2b512Block::new(0),
            last_block: vec![0; 128],
            path_view: vec![0; 112],
            count_view: [0; 2],
            hash_array: [0; 64],
            position: 0,
        };

        let range: Vec<usize> = (offset..(offset + length - 127)).step_by(128).collect();

        for &base in &range {
            result.digest.update(&init, base);
        }

        let partial_base = if range.is_empty() {
            offset
        } else {
            range.last().unwrap() + 128
        };

        // If there is any remainder:
        if offset + length != partial_base {
            let mut padded = vec![0; 128];

            padded.copy_from_slice(&init[partial_base..(offset + length)]);
            result.digest.update(&padded, 0);
        }

        result
    }

    pub fn new(init: &[u8]) -> Blake2b512Random {
        Blake2b512Random::create(init, 0, init.len())
    }

    pub fn split_short(&self, index: u16) -> Blake2b512Random {
        let mut split = self.clone(); // Assuming you have implemented Clone for Blake2b512Random
        let packed = index.to_le_bytes(); // Convert the short to little-endian bytes
        split.add_byte(packed[0]);
        split.add_byte(packed[1]);
        split
    }

    pub fn split_byte(&self, index: u8) -> Blake2b512Random {
        let mut split = self.clone(); // Create a copy of the current instance
        split.add_byte(index); // Add the byte to the path_view
        split // Return the new instance
    }

    fn add_byte(&mut self, index: u8) {
        if self.path_view.len() == 112 {
            self.digest.update(&self.last_block, 0);
            self.last_block.fill(0); // Reset last_block
            self.path_view.clear(); // Reset path_view
        }
        self.path_view.push(index);
    }

    pub fn to_vec(&self) -> Vec<u8> {
        self.digest.to_vec() // Assuming Blake2b512Block has a to_vec method
    }

    pub fn from_vec(data: Vec<u8>) -> Blake2b512Block {
        Blake2b512Block::from_vec(data) // Assuming Blake2b512Block has a from_vec method
    }
}

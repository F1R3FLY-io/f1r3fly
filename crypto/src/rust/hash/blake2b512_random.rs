// See crypto/src/main/scala/coop/rchain/crypto/hash/Blake2b512Random.scala

use super::blake2b512_block::Blake2b512Block;

/** Blake2b512 based splittable and mergeable random number generator
 * specialized for generating 256-bit unforgeable names.
 * splitByte and splitShort are the interfaces to make the random number
 * generator diverge.
 * Blake2b512.merge uses online tree hashing to merge two random generator
 * states.
 *
 * TODO: Investigate this as the custom type in RhoTypes.proto
 */

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
}

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

#[derive(Clone, Debug)]
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

        if length > 127 {
            let range: Vec<usize> = (offset..(offset + length - 127)).step_by(128).collect();

            for &base in &range {
                result.digest.update(&init, base);
            }

            let partial_base = range.last().unwrap() + 128;

            // If there is any remainder:
            if offset + length != partial_base {
                let mut padded = vec![0; 128];
                let remainder_length = (offset + length) - partial_base;
                padded.copy_from_slice(&init[partial_base..(partial_base + remainder_length)]);
                result.digest.update(&padded, 0);
            }
        } else {
            // Handle the case where length is less than or equal to 127
            result.digest.update(&init[offset..(offset + length)], 0);
        }

        result
    }

    pub fn new(init: &[u8]) -> Blake2b512Random {
        Blake2b512Random::create(init, 0, init.len())
    }

    pub fn split_short(&self, index: u16) -> Blake2b512Random {
        let mut split = self.clone();
        let packed = index.to_le_bytes();
        split.add_byte(packed[0]);
        split.add_byte(packed[1]);
        split
    }

    pub fn split_byte(&self, index: u8) -> Blake2b512Random {
        let mut split = self.clone();
        split.add_byte(index);
        split
    }

    fn add_byte(&mut self, index: u8) {
        if self.path_view.len() == 112 {
            self.digest.update(&self.last_block, 0);
            self.last_block.fill(0);
            self.path_view.clear();
        }
        self.path_view.push(index);
    }

    pub fn next(&mut self) -> Vec<u8> {
        if self.position == 0 {
            self.hash();
            self.position = 32;
            self.hash_array[0..32].to_vec()
        } else {
            let result = self.hash_array[32..64].to_vec();
            self.position = 0;
            result
        }
    }

    fn hash(&mut self) {
        self.digest
            .peek_final_root(self.last_block.as_slice(), 0, &mut self.hash_array, 0);
        let low = self.count_view[0];
        if low == u64::MAX {
            let high = self.count_view[1];
            self.count_view[0] = 0;
            self.count_view[1] = high + 1;
        } else {
            self.count_view[0] = low + 1;
        }
    }

    pub fn merge(mut children: Vec<Blake2b512Random>) -> Blake2b512Random {
        assert!(
            children.len() >= 2,
            "Blake2b512Random should have at least 2 inputs to merge."
        );

        let mut squashed_builder = Vec::new();
        let mut chain_block = vec![0; 128];

        while children.len() > 1 {
            let mut result = Blake2b512Random::new(&[0; 0]);
            let chunks = children.chunks_mut(255).collect::<Vec<_>>();

            for slice in chunks {
                for quad in slice.chunks_mut(4) {
                    for (i, child) in quad.iter_mut().enumerate() {
                        child.digest.finalize_internal(
                            child.last_block.as_slice(),
                            0,
                            &mut chain_block,
                            i * 32,
                        );
                    }
                    if quad.len() != 4 {
                        let blank_length = (4 - quad.len()) * 32;
                        if blank_length > 0 {
                            let mut blank = Blake2b512Random::BLANK_BLOCK;
                            blank
                                .get_mut(..blank_length)
                                .unwrap()
                                .copy_from_slice(&chain_block[quad.len() * 32..]);
                        }
                    }
                    result.digest.update(&chain_block, 0);
                }
                squashed_builder.push(result.clone());
            }
            children = squashed_builder;
            squashed_builder = Vec::new();
        }

        children.remove(0)
    }

    const BLANK_BLOCK: [u8; 128] = [0; 128];

    pub fn to_vec(&self) -> Vec<u8> {
        self.digest.to_vec()
    }

    pub fn from_vec(data: Vec<u8>) -> Blake2b512Block {
        Blake2b512Block::from_vec(data)
    }
}

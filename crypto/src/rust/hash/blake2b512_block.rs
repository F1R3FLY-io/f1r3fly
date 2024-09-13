/* See crypto/src/main/scala/coop/rchain/crypto/hash/Blake2b512Block.scala */

/**
Block oriented Blake2b512 class.

We're using some of the tree hashing parameters to achieve online tree hashing,
where the structure of the tree is provided by the application. This precludes
using the depth counters. Also, because we are online, we can't know before-hand
that a node is the last in a level, or is offset. When we hash the root, we know
that it is the root, but since we are only publishing the root hash, and not the
sub-tree, this is sufficient protection against extension.

I've (not spreston8, old dev) checked that we should be fine against the criteria here.
See: "Sufficient conditions for sound tree and sequential hashing modes" by
Guido Bertoni, Joan Daemen, Michaël Peeters, and Gilles Van Assche

    We also have data at every level, so we're using the fanout parameter to
distinguish child-hashes from data. To make it convenient for block-orientation,
we will null-pad an odd number of child hashes. This means that the fanout varies per-node
rather than being set for the whole instance. Where applicable, we are setting
the other tree-parameters accordingly. Namely, max-depth to 255 (unlimited) and
inner hash length to 64.

This class is an abbreviated version of Blake2bDigest.java from BouncyCastle
https://github.com/bcgit/bc-java/blob/master/core/src/main/java/org/bouncycastle/crypto/digests/Blake2bDigest.java
  */

// TODO: REVIEW
#[derive(Clone, Debug)]
pub struct Blake2b512Block {
    chain_value: [u64; 8],
    t0: u64,
    t1: u64,
}

// Depth = 255, Fanout = ??, Keylength = 0, Digest length = 64 bytes
const PARAM_VALUE_0: u64 = 0xFF000040;

// Inner length = 32 bytes
const PARAM_VALUE_2: u64 = 0x2000;

// Produced from the square root of primes 2, 3, 5, 7, 11, 13, 17, 19.
// The same as SHA-512 IV.
const IV: [u64; 8] = [
    0x6A09E667F3BCC908,
    0xBB67AE8584CAA73B,
    0x3C6EF372FE94F82B,
    0xA54FF53A5F1D36F1,
    0x510E527FADE682D1,
    0x9B05688C2B3E6C1F,
    0x1F83D9ABFB41BD6B,
    0x5BE0CD19137E2179,
];

impl Blake2b512Block {
    pub fn new(fanout: u8) -> Blake2b512Block {
        let param0_with_fanout = PARAM_VALUE_0 | ((fanout as u64 & 0xff) << 16);

        let mut result = Blake2b512Block {
            chain_value: [0; 8],
            t0: 0,
            t1: 0,
        };

        result.chain_value.copy_from_slice(&IV);
        result.chain_value[0] ^= param0_with_fanout;
        result.chain_value[2] ^= PARAM_VALUE_2;
        result
    }

    pub fn update(&mut self, block: &[u8], offset: usize) {
        let cv = &mut self.chain_value.clone();
        self.compress(block, offset, cv, false, false, false);
    }

    pub fn peek_final_root(
        &mut self,
        block: &[u8],
        in_offset: usize,
        output: &mut [u8],
        out_offset: usize,
    ) {
        let mut temp_chain_value = [0u64; 8];
        self.compress(block, in_offset, &mut temp_chain_value, true, true, true);
        self.long_to_little_endian(&temp_chain_value, output, out_offset);
    }

    pub fn finalize_internal(
        &mut self,
        block: &[u8],
        in_offset: usize,
        output: &mut [u8],
        out_offset: usize,
    ) {
        let mut temp_chain_value = [0u64; 8];
        self.compress(block, in_offset, &mut temp_chain_value, true, true, false);
        for i in 0..4 {
            let start = out_offset + i * 8;
            output[start..start + 8].copy_from_slice(&temp_chain_value[i].to_le_bytes());
        }
    }

    fn long_to_little_endian(&self, input: &[u64], output: &mut [u8], out_offset: usize) {
        for (i, &value) in input.iter().enumerate() {
            let start = out_offset + i * 8;
            output[start..start + 8].copy_from_slice(&value.to_le_bytes());
        }
    }

    pub fn compress(
        &mut self,
        msg: &[u8],
        offset: usize,
        new_chain_value: &mut [u64; 8],
        peek: bool,
        finalize: bool,
        root_finalize: bool,
    ) {
        let mut internal_state = [0u64; 16];
        let new_t0 = self.t0 + 128; // BLOCK_LENGTH_BYTES
        let new_t1 = if new_t0 == 0 { self.t1 + 1 } else { self.t1 };

        // Initialize internal state
        internal_state[..8].copy_from_slice(&self.chain_value);
        internal_state[8..16].copy_from_slice(&IV); // Adjusted to copy the full IV

        let f0 = if finalize { 0xFFFFFFFFFFFFFFFF } else { 0 };
        let f1 = if root_finalize { 0xFFFFFFFFFFFFFFFF } else { 0 };

        internal_state[12] = new_t0 ^ IV[4];
        internal_state[13] = new_t1 ^ IV[5];
        internal_state[14] = f0 ^ IV[6];
        internal_state[15] = f1 ^ IV[7];

        let mut m = [0u64; 16];

        for i in 0..16 {
            if offset + i * 8 + 8 <= msg.len() {
                m[i] = u64::from_le_bytes(
                    msg[offset + i * 8..offset + (i + 1) * 8]
                        .try_into()
                        .unwrap(),
                );
            } else {
                m[i] = 0; // Handle the case where there is not enough data
            }
        }

        for _ in 0..12 {
            // ROUNDS
            // Columns
            self.g(m[0], m[1], 0, 4, 8, 12, &mut internal_state);
            self.g(m[2], m[3], 1, 5, 9, 13, &mut internal_state);
            self.g(m[4], m[5], 2, 6, 10, 14, &mut internal_state);
            self.g(m[6], m[7], 3, 7, 11, 15, &mut internal_state);

            // Diagonals
            self.g(m[8], m[9], 0, 5, 10, 15, &mut internal_state);
            self.g(m[10], m[11], 1, 6, 11, 12, &mut internal_state);
            self.g(m[12], m[13], 2, 7, 8, 13, &mut internal_state);
            self.g(m[14], m[15], 3, 4, 9, 14, &mut internal_state);
        }

        if !peek {
            self.t0 = new_t0;
            self.t1 = new_t1;

            for i in 0..8 {
                new_chain_value[i] =
                    self.chain_value[i] ^ internal_state[i] ^ internal_state[i + 8];
            }
        } else {
            for i in 0..8 {
                new_chain_value[i] =
                    self.chain_value[i] ^ internal_state[i] ^ internal_state[i + 8];
            }
        }
    }

    fn g(
        &self,
        m1: u64,
        m2: u64,
        pos_a: usize,
        pos_b: usize,
        pos_c: usize,
        pos_d: usize,
        internal_state: &mut [u64; 16],
    ) {
        let rotr64 = |x: u64, rot: u32| x.rotate_right(rot);

        internal_state[pos_a] = internal_state[pos_a]
            .wrapping_add(internal_state[pos_b])
            .wrapping_add(m1);

        internal_state[pos_d] = rotr64(internal_state[pos_d] ^ internal_state[pos_a], 32);
        internal_state[pos_c] = internal_state[pos_c].wrapping_add(internal_state[pos_d]);
        internal_state[pos_b] = rotr64(internal_state[pos_b] ^ internal_state[pos_c], 24);

        internal_state[pos_a] = internal_state[pos_a]
            .wrapping_add(internal_state[pos_b])
            .wrapping_add(m2);

        internal_state[pos_d] = rotr64(internal_state[pos_d] ^ internal_state[pos_a], 16);
        internal_state[pos_c] = internal_state[pos_c].wrapping_add(internal_state[pos_d]);
        internal_state[pos_b] = rotr64(internal_state[pos_b] ^ internal_state[pos_c], 63);
    }

    pub fn to_vec(&self) -> Vec<u8> {
        let mut vec = Vec::with_capacity(8 * 3); // 8 u64 for chain_value, t0, t1

        for &value in &self.chain_value {
            vec.extend_from_slice(&value.to_le_bytes());
        }

        vec.extend_from_slice(&self.t0.to_le_bytes());
        vec.extend_from_slice(&self.t1.to_le_bytes());

        vec
    }

    pub fn from_vec(data: Vec<u8>) -> Blake2b512Block {
        let mut chain_value = [0u64; 8];

        for i in 0..8 {
            chain_value[i] = u64::from_le_bytes(data[i * 8..(i + 1) * 8].try_into().unwrap());
        }

        let t0 = u64::from_le_bytes(data[64..72].try_into().unwrap());
        let t1 = u64::from_le_bytes(data[72..80].try_into().unwrap());

        Blake2b512Block {
            chain_value,
            t0,
            t1,
        }
    }
}

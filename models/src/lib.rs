pub mod rust;

pub mod rhoapi {
    include!(concat!(env!("OUT_DIR"), "/rhoapi.rs"));
}

pub mod rspace_plus_plus_types {
    include!(concat!(env!("OUT_DIR"), "/rspace_plus_plus_types.rs"));
}

pub type ByteVector = Vec<u8>;
pub type ByteBuffer = Vec<u8>;
pub type Byte = u8;
pub type ByteString = Vec<u8>;
pub type BitSet = Vec<u8>;

pub fn create_bit_vector(indices: &[usize]) -> BitSet {
    let max_index = *indices.iter().max().unwrap_or(&0);
    let mut bit_vector = vec![0; max_index + 1];
    for &index in indices {
        bit_vector[index] = 1;
    }
    // println!("\nbitvector: {:?}", bit_vector);
    bit_vector
}

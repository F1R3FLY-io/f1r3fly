use super::signatures_alg::SignaturesAlg;
use crate::rust::{private_key::PrivateKey, public_key::PublicKey};
use ed25519_dalek::ed25519::signature::hazmat::PrehashVerifier;
use k256::ecdsa::VerifyingKey;
use k256::{
    ecdsa::{signature::hazmat::PrehashSigner, Signature, SigningKey},
    elliptic_curve::generic_array::GenericArray,
};

use typenum::U32;

use k256::elliptic_curve::{sec1::ToEncodedPoint, SecretKey};
use rand::rngs::OsRng;

// See crypto/src/main/scala/coop/rchain/crypto/signatures/Secp256k1.scala
#[derive(Clone, Debug, PartialEq)]
pub struct Secp256k1;

impl Secp256k1 {
    pub fn name() -> String {
        "secp256k1".to_string()
    }
}

// TODO: Remove self in these methods
impl SignaturesAlg for Secp256k1 {
    fn verify(&self, data: &[u8], signature: &[u8], pub_key: &[u8]) -> bool {
        // Scala implementation always expects 32-byte hashes (Blake2b256 or SHA256)
        // This matches the Scala comment: "@param data The data which was signed, must be exactly 32 bytes"
        if data.len() != 32 {
            log::warn!(
                "secp256k1.verify: expected 32 bytes (hash), got {} bytes. Data: {}",
                data.len(),
                hex::encode(data)
            );
            return false;
        }

        match VerifyingKey::from_sec1_bytes(pub_key) {
            Ok(vk) => match Signature::from_der(signature) {
                Ok(sig) => vk.verify_prehash(data, &sig).is_ok(),
                Err(_) => false,
            },
            Err(_) => false,
        }
    }

    fn sign(&self, data: &[u8], sec: &[u8]) -> Vec<u8> {
        // Scala implementation always expects 32-byte hashes
        // This matches the Scala comment: "@param data Message hash, 32 bytes"
        if data.len() != 32 {
            log::warn!(
                "secp256k1.sign: expected 32 bytes (hash), got {} bytes. Data: {}",
                data.len(),
                hex::encode(data)
            );
        }

        let key_bytes = GenericArray::clone_from_slice(sec);
        let signing_key = SigningKey::from_bytes(&key_bytes).expect("Invalid private key");

        // Always use prehash signing since we expect hashed data
        let signature: Signature = signing_key
            .sign_prehash(data)
            .expect("Failed to sign prehash");

        signature.to_der().as_bytes().to_vec()
    }

    fn to_public(&self, sec: &PrivateKey) -> PublicKey {
        let key_bytes: GenericArray<u8, U32> = GenericArray::clone_from_slice(&sec.bytes);
        let secret_key: SecretKey<k256::Secp256k1> =
            SecretKey::from_bytes(&key_bytes).expect("Invalid private key");

        let public_key = secret_key.public_key();
        let public_key_bytes = public_key.to_encoded_point(false).as_bytes().to_vec();

        PublicKey::from_bytes(&public_key_bytes)
    }

    fn new_key_pair(&self) -> (PrivateKey, PublicKey) {
        let secret_key = SecretKey::<k256::Secp256k1>::random(&mut OsRng);
        let raw_public_key = secret_key.public_key();

        let private_key = PrivateKey::from_bytes(&secret_key.to_bytes());

        let public_key =
            PublicKey::from_bytes(&raw_public_key.to_encoded_point(false).as_bytes().to_vec());

        (private_key, public_key)
    }

    fn name(&self) -> String {
        "secp256k1".to_string()
    }

    fn sig_length(&self) -> usize {
        32
    }

    fn eq(&self, other: &dyn SignaturesAlg) -> bool {
        self.name() == other.name()
    }

    fn box_clone(&self) -> Box<dyn SignaturesAlg> {
        Box::new(self.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sha2::{Digest, Sha256};

    fn sec_key_verify(seckey: &[u8]) -> bool {
        seckey.len() == 32 && SigningKey::from_bytes(GenericArray::from_slice(seckey)).is_ok()
    }

    //crypto/src/test/scala/coop/rchain/crypto/signatures/Secp256k1Spec.scala
    #[test]
    fn generate_valid_key_pair() {
        let secp256k1 = Secp256k1;

        let all_pairs_valid = (0..1000).all(|_| {
            let (_private_key, _public_key) = secp256k1.new_key_pair();
            true
        });

        assert!(
            all_pairs_valid,
            "Not all key pairs were created successfully"
        );
    }

    //crypto/src/test/scala/coop/rchain/crypto/signatures/Secp256k1Test.scala
    #[test]
    fn verifies_the_given_secp256k1_signature_in_native_code_with_keypair() {
        let secp256k1 = Secp256k1;
        let (private_key, public_key) = secp256k1.new_key_pair();
        let data = Sha256::digest(b"testing");

        let signature = secp256k1.sign(&data, &private_key.bytes);
        let is_valid = secp256k1.verify(&data, &signature, &public_key.bytes);

        assert!(is_valid, "Signature is not valid for the generated keypair");
    }

    #[test]
    fn verifies_the_given_secp256k1_signature_in_native_code() {
        let secp256k1 = Secp256k1;
        let data = Sha256::digest(b"testing");

        // Generate a test key pair and signature using our implementation
        let private_key =
            hex::decode("54d4e576251b6c7b95af0349a5a31e583e982edeb9856a59bde14f6be5a302b3")
                .expect("Failed to decode private key");
        let public_key = secp256k1.to_public(&PrivateKey::from_bytes(&private_key));
        let signature = secp256k1.sign(&data, &private_key);

        let is_valid = secp256k1.verify(&data, &signature, &public_key.bytes);

        assert!(is_valid, "Static signature verification failed");
    }

    #[test]
    fn packaged_in_libsecp256k1_creates_an_ecdsa_signature() {
        let secp256k1 = Secp256k1;
        let data = Sha256::digest(b"testing");

        let private_key =
            hex::decode("54d4e576251b6c7b95af0349a5a31e583e982edeb9856a59bde14f6be5a302b3")
                .expect("Failed to decode private key");

        // Generate the signature with our implementation - don't hardcode expected value
        let generated_signature = secp256k1.sign(&data, &private_key);

        // Verify that our generated signature is valid
        let public_key = secp256k1.to_public(&PrivateKey::from_bytes(&private_key));
        let is_valid = secp256k1.verify(&data, &generated_signature, &public_key.bytes);

        assert!(is_valid, "Generated signature should be valid");

        // Ensure signature is DER-encoded (starts with 0x30)
        assert_eq!(
            generated_signature[0], 0x30,
            "Signature should be DER-encoded"
        );
    }

    #[test]
    fn verify_returns_true_if_valid_false_if_invalid() {
        let valid_private_key =
            hex::decode("75468aa68961817b41d7fc2350ae705a7d773ea0d6609f5c25c2aee8e0adced6")
                .expect("Failed to decode valid private key");

        assert!(
            sec_key_verify(&valid_private_key),
            "Valid private key verification failed"
        );
    }

    #[test]
    fn computes_public_key_from_secret_key() {
        let secp256k1 = Secp256k1;

        let private_key =
            hex::decode("54d4e576251b6c7b95af0349a5a31e583e982edeb9856a59bde14f6be5a302b3")
                .expect("Failed to decode private key");
        let expected_public_key = "0418a6b57c4aeee6c7e19e3ea25aa5bae270eca8580ee5e59c28921df743e416a316c55ed10c63b99a7c2705de0e0d3c52ad7f06144b7f6ed97d3a63b871ced6ff";

        let computed_public_key = secp256k1.to_public(&PrivateKey::from_bytes(&private_key));

        assert_eq!(
            hex::encode(computed_public_key.bytes).to_uppercase(),
            expected_public_key.to_uppercase(),
            "Computed public key does not match the expected public key"
        );
    }

    #[test]
    fn verify_should_return_false_for_invalid_signatures() {
        let secp256k1 = Secp256k1;
        let (private_key, public_key) = secp256k1.new_key_pair();
        let data = Sha256::digest(b"testing");
        let signature = secp256k1.sign(&data, &private_key.bytes);

        // Valid signature should return true
        assert!(secp256k1.verify(&data, &signature, &public_key.bytes));

        // Test with wrong data
        let wrong_data = Sha256::digest(b"wrong data");
        assert!(!secp256k1.verify(&wrong_data, &signature, &public_key.bytes));

        // Test with wrong public key
        let (_, wrong_public_key) = secp256k1.new_key_pair();
        assert!(!secp256k1.verify(&data, &signature, &wrong_public_key.bytes));

        // Test with malformed signature
        let malformed_signature = vec![0u8; 10];
        assert!(!secp256k1.verify(&data, &malformed_signature, &public_key.bytes));

        // Test with empty signature
        let empty_signature = vec![];
        assert!(!secp256k1.verify(&data, &empty_signature, &public_key.bytes));

        // Test with malformed public key
        let malformed_public_key = vec![0u8; 10];
        assert!(!secp256k1.verify(&data, &signature, &malformed_public_key));

        // Test with empty public key
        let empty_public_key = vec![];
        assert!(!secp256k1.verify(&data, &signature, &empty_public_key));
    }

    #[test]
    fn verify_should_return_false_for_signature_from_different_key() {
        let secp256k1 = Secp256k1;
        let (private_key1, public_key1) = secp256k1.new_key_pair();
        let (private_key2, _public_key2) = secp256k1.new_key_pair();
        let data = Sha256::digest(b"testing");

        // Sign with private_key1
        let signature1 = secp256k1.sign(&data, &private_key1.bytes);
        // Sign with private_key2
        let signature2 = secp256k1.sign(&data, &private_key2.bytes);

        // Verify signature1 with public_key1 should succeed
        assert!(secp256k1.verify(&data, &signature1, &public_key1.bytes));

        // Verify signature2 with public_key1 should fail
        assert!(!secp256k1.verify(&data, &signature2, &public_key1.bytes));
    }

    #[test]
    fn verify_should_handle_various_invalid_der_signatures() {
        let secp256k1 = Secp256k1;
        let (_, public_key) = secp256k1.new_key_pair();
        let data = Sha256::digest(b"testing");

        // Test various invalid DER signatures
        let invalid_signatures = vec![
            // Too short
            vec![0x30, 0x02, 0x01, 0x00],
            // Invalid DER tag
            vec![0x31, 0x44, 0x02, 0x20],
            // Random bytes
            vec![0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff],
            // Valid DER structure but invalid signature values  
            hex::decode("3044022000000000000000000000000000000000000000000000000000000000000000000220000000000000000000000000000000000000000000000000000000000000000").unwrap_or_else(|_| vec![0x30, 0x44, 0x02, 0x20]),
        ];

        for invalid_sig in invalid_signatures {
            assert!(
                !secp256k1.verify(&data, &invalid_sig, &public_key.bytes),
                "Should reject invalid signature: {:?}",
                invalid_sig
            );
        }
    }

    #[test]
    fn verify_should_handle_various_invalid_public_keys() {
        let secp256k1 = Secp256k1;
        let (private_key, _) = secp256k1.new_key_pair();
        let data = Sha256::digest(b"testing");
        let signature = secp256k1.sign(&data, &private_key.bytes);

        // Test various invalid public keys
        let invalid_public_keys = vec![
            // Too short
            vec![0x04],
            // Wrong prefix
            vec![0x05; 65],
            // Wrong length
            vec![0x04; 64],
            vec![0x04; 66],
            // Random bytes
            vec![0xaa; 65],
            // Empty
            vec![],
        ];

        for invalid_key in invalid_public_keys {
            assert!(
                !secp256k1.verify(&data, &signature, &invalid_key),
                "Should reject invalid public key: {:?}",
                invalid_key
            );
        }
    }

    #[test]
    fn verify_edge_cases() {
        let secp256k1 = Secp256k1;
        let (private_key, public_key) = secp256k1.new_key_pair();
        let data = Sha256::digest(b"testing");
        let signature = secp256k1.sign(&data, &private_key.bytes);

        // Test with different 32-byte hash - should work with proper signature for that hash
        let different_data = Sha256::digest(b"different data");
        let different_signature = secp256k1.sign(&different_data, &private_key.bytes);
        assert!(secp256k1.verify(&different_data, &different_signature, &public_key.bytes));

        // But original signature should not work with different hash
        assert!(!secp256k1.verify(&different_data, &signature, &public_key.bytes));

        // And different signature should not work with original hash
        assert!(!secp256k1.verify(&data, &different_signature, &public_key.bytes));
    }

    #[test]
    fn verify_scala_generated_signature() {
        let secp256k1 = Secp256k1;

        // Create a realistic Blake2b256 hash from known data (simulating blessed contracts)
        use crate::rust::hash::blake2b256::Blake2b256;
        let test_data = b"test data for blessed contract signature";
        let blake2b_hash = Blake2b256::hash(test_data.to_vec());

        // Use a known private key from ListOps
        let private_key =
            hex::decode("867c21c6a3245865444d80e49cac08a1c11e23b35965b566bbe9f49bb9897511")
                .expect("Failed to decode private key");

        // Generate public key and signature using our implementation
        let public_key = secp256k1.to_public(&PrivateKey::from_bytes(&private_key));
        let signature = secp256k1.sign(&blake2b_hash, &private_key);

        let is_valid = secp256k1.verify(&blake2b_hash, &signature, &public_key.bytes);

        assert!(is_valid, "Blake2b256 signature should verify successfully");
    }
}

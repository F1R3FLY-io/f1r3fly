use crate::rust::private_key::PrivateKey;
use crate::rust::public_key::PublicKey;
use crate::rust::signatures::secp256k1::Secp256k1;
use crate::rust::signatures::signatures_alg::SignaturesAlg;
use num_bigint::BigUint;

// See crypto/src/main/scala/coop/rchain/crypto/signatures/Secp256k1Eth.scala
pub struct Secp256k1Eth;

//TODO need review
impl SignaturesAlg for Secp256k1Eth {
    fn verify(&self, data: &[u8], signature: &[u8], pub_key: Vec<u8>) -> bool {
        if let Ok(signature_der) = encode_signature_rs_to_der(signature) {
            Secp256k1.verify(data, &signature_der, pub_key)
        } else {
            false
        }
    }

    fn sign(&self, data: &[u8], sec: &[u8]) -> Vec<u8> {
        let sig_der = Secp256k1.sign(data, sec);
        let rs_signature = decode_signature_der_to_rs(&sig_der);

        match rs_signature {
            Some(sig) if sig.len() == 64 => sig,
            _ => vec![0u8; 64], //sign should always return 64
        }
    }

    fn to_public(&self, sec: &PrivateKey) -> PublicKey {
        Secp256k1.to_public(sec)
    }

    fn new_key_pair(&self) -> (PrivateKey, PublicKey) {
        Secp256k1.new_key_pair()
    }

    fn name(&self) -> String {
        format!("{}:eth", Secp256k1.name())
    }

    fn sig_length(&self) -> usize {
        Secp256k1.sig_length()
    }
}

fn encode_signature_rs_to_der(signature_rs: &[u8]) -> Result<Vec<u8>, &'static str> {
    if signature_rs.len() != 64 {
        return Err("Signature in RS format must be 64 bytes");
    }

    let r = &signature_rs[0..32];
    let s = &signature_rs[32..64];

    let der_signature = yasna::construct_der(|writer| {
        writer.write_sequence(|writer| {
            writer.next().write_biguint(&BigUint::from_bytes_be(r));
            writer.next().write_biguint(&BigUint::from_bytes_be(s));
        });
    });

    Ok(der_signature)
}

fn decode_signature_der_to_rs(signature_der: &[u8]) -> Option<Vec<u8>> {
    yasna::parse_der(signature_der, |reader| {
        reader.read_sequence(|reader| {
            let r = reader.next().read_biguint()?;
            let s = reader.next().read_biguint()?;

            let mut r_bytes = r.to_bytes_be();
            let mut s_bytes = s.to_bytes_be();

            while r_bytes.len() < 32 {
                r_bytes.insert(0, 0);
            }
            while s_bytes.len() < 32 {
                s_bytes.insert(0, 0);
            }

            if r_bytes.len() > 32 || s_bytes.len() > 32 {
                eprintln!(
                    "Decoded R or S length exceeds 32 bytes: R = {}, S = {}",
                    r_bytes.len(),
                    s_bytes.len()
                );
                return Err(yasna::ASN1Error::new(yasna::ASN1ErrorKind::Invalid));
            }

            let mut signature_rs = Vec::with_capacity(64);
            signature_rs.extend_from_slice(&r_bytes);
            signature_rs.extend_from_slice(&s_bytes);

            Ok(signature_rs)
        })
    })
    .ok()
}

//crypto/src/test/scala/coop/rchain/crypto/util/DERConverterSpec.scala
#[cfg(test)]
mod tests {
    use super::{decode_signature_der_to_rs, encode_signature_rs_to_der, Secp256k1Eth};
    use crate::rust::signatures::secp256k1::Secp256k1;
    use crate::rust::signatures::signatures_alg::SignaturesAlg;

    #[test]
    fn der_converter_check_with_valid_and_non_empty_input() {
        let secp256k1_eth = Secp256k1Eth;
        let secp256k1 = Secp256k1;

        for _ in 0..100 {
            let (private_key, _public_key) = secp256k1.new_key_pair();
            let data: Vec<u8> = (0..32).map(|_| rand::random::<u8>()).collect();
            let sig_rs = secp256k1_eth.sign(&data, &private_key.bytes);

            assert_eq!(sig_rs.len(), 64, "Generated RS signature must be 64 bytes");

            let sig_der = encode_signature_rs_to_der(&sig_rs).expect("Failed to encode RS to DER");
            let expected_rs =
                decode_signature_der_to_rs(&sig_der).expect("Failed to decode DER to RS");
            assert_eq!(sig_rs, expected_rs, "Mismatch after encode/decode");

            let invalid_bytes: Vec<u8> = (0..rand::random::<u8>() % 100)
                .map(|_| rand::random::<u8>())
                .collect();

            if invalid_bytes.len() != 64 {
                let encoded = encode_signature_rs_to_der(&invalid_bytes);
                assert!(encoded.is_err(), "Encoder did not reject invalid RS length");

                let decoded = decode_signature_der_to_rs(&invalid_bytes);
                assert!(
                    decoded.is_none(),
                    "Decoder did not reject invalid DER input"
                );
            }
        }
    }

    #[test]
    fn encoder_should_fail_on_empty_input() {
        let empty_bytes: Vec<u8> = vec![];
        let result = encode_signature_rs_to_der(&empty_bytes);
        assert!(result.is_err());
    }

    #[test]
    fn decoder_should_fail_on_empty_input() {
        let empty_bytes: Vec<u8> = vec![];
        let result = decode_signature_der_to_rs(&empty_bytes);
        assert!(result.is_none());
    }

    #[test]
    fn test_encode_decode_rs_der() {
        let signature_rs = vec![0u8; 64];
        let der_signature =
            encode_signature_rs_to_der(&signature_rs).expect("Failed to encode RS to DER");
        let decoded_rs =
            decode_signature_der_to_rs(&der_signature).expect("Failed to decode DER to RS");
        assert_eq!(
            signature_rs, decoded_rs,
            "RS signatures do not match after encode/decode"
        );
    }
}

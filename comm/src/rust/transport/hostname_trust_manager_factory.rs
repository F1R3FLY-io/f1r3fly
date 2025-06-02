// comm/src/main/scala/coop/rchain/comm/transport/HostnameTrustManagerFactory.scala

use crypto::rust::util::certificate_helper::CertificateHelper;
use p256::elliptic_curve::sec1::FromEncodedPoint;
use rustls::client::danger::{ServerCertVerifier, ServerCertVerified, HandshakeSignatureValid};
use rustls::{SignatureScheme, DigitallySignedStruct, Error as RustlsError};
use rustls::pki_types::{CertificateDer, ServerName};
use std::sync::Arc;

/// Custom error type for certificate validation
#[derive(Debug, Clone)]
pub enum CertificateValidationError {
    ValidationFailed(String),
    NotAllowedMethod,
    NoHandshakeSession,
    WrongAlgorithm,
    AddressHostnameMismatch,
    NoEndpointIdentification,
    UnknownIdentificationAlgorithm(String),
    ParsingError(String),
}

impl std::fmt::Display for CertificateValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ValidationFailed(msg) => write!(f, "Certificate validation failed: {}", msg),
            Self::NotAllowedMethod => write!(f, "Not allowed validation method"),
            Self::NoHandshakeSession => write!(f, "No handshake session"),
            Self::WrongAlgorithm => write!(f, "Certificate's public key has the wrong algorithm"),
            Self::AddressHostnameMismatch => {
                write!(f, "Certificate's public address doesn't match the hostname")
            }
            Self::NoEndpointIdentification => write!(f, "No endpoint identification algorithm"),
            Self::UnknownIdentificationAlgorithm(alg) => {
                write!(f, "Unknown identification algorithm: {}", alg)
            }
            Self::ParsingError(msg) => write!(f, "Certificate parsing error: {}", msg),
        }
    }
}

impl std::error::Error for CertificateValidationError {}

/// HostnameTrustManagerFactory - creates custom certificate verifiers for F1r3fly
pub struct HostnameTrustManagerFactory;

impl HostnameTrustManagerFactory {
    /// Singleton instance
    pub fn instance() -> &'static Self {
        static INSTANCE: HostnameTrustManagerFactory = HostnameTrustManagerFactory;
        &INSTANCE
    }

    /// Create a hostname trust manager
    pub fn create_trust_manager(&self) -> Arc<HostnameTrustManager> {
        Arc::new(HostnameTrustManager::new())
    }
}

/// HostnameTrustManager - implements F1r3fly's custom certificate validation logic
/// Directly implements rustls ServerCertVerifier for seamless integration
#[derive(Debug)]
pub struct HostnameTrustManager;

impl HostnameTrustManager {
    pub fn new() -> Self {
        Self
    }

    /// Check client certificate trust
    pub fn check_client_trusted(
        &self,
        cert_der: &[u8],
        _auth_type: &str,
    ) -> Result<String, CertificateValidationError> {
        // Extract F1r3fly public address from certificate
        let peer_host = self.extract_f1r3fly_address(cert_der)?;

        // Perform identity check with HTTPS algorithm
        self.check_identity(Some(&peer_host), cert_der, "https")?;

        Ok(peer_host)
    }

    /// Check server certificate trust  
    pub fn check_server_trusted(
        &self,
        cert_der: &[u8],
        _auth_type: &str,
        peer_hostname: Option<&str>,
    ) -> Result<(), CertificateValidationError> {
        // Perform hostname verification
        self.check_identity(peer_hostname, cert_der, "https")?;

        // Validate F1r3fly address matches hostname
        if let Some(hostname) = peer_hostname {
            let f1r3fly_address = self.extract_f1r3fly_address(cert_der)?;
            if f1r3fly_address != hostname {
                return Err(CertificateValidationError::AddressHostnameMismatch);
            }
        }

        Ok(())
    }

    /// Extract F1r3fly public address from certificate
    fn extract_f1r3fly_address(
        &self,
        cert_der: &[u8],
    ) -> Result<String, CertificateValidationError> {
        // Parse the certificate to extract the public key
        let public_key = self.extract_public_key_from_cert(cert_der)?;

        // Use CertificateHelper to compute the F1r3fly address
        let address = CertificateHelper::public_address(&public_key)
            .ok_or(CertificateValidationError::WrongAlgorithm)?;

        // Encode as hex (Base16)
        Ok(hex::encode(&address))
    }

    /// Extract secp256r1 public key from X.509 certificate
    fn extract_public_key_from_cert(
        &self,
        cert_der: &[u8],
    ) -> Result<p256::PublicKey, CertificateValidationError> {
        // Parse certificate
        let (_, cert) = x509_parser::parse_x509_certificate(cert_der)
            .map_err(|e| CertificateValidationError::ParsingError(format!("Invalid DER: {}", e)))?;

        // Extract public key bytes
        let public_key_info = cert.public_key();
        let public_key_bytes = public_key_info.subject_public_key.data.as_ref();

        // Convert to p256::PublicKey
        if public_key_bytes.len() == 65 && public_key_bytes[0] == 0x04 {
            // Uncompressed secp256r1 point format
            let encoded_point = p256::EncodedPoint::from_bytes(public_key_bytes)
                .map_err(|_| CertificateValidationError::WrongAlgorithm)?;

            let public_key = p256::PublicKey::from_encoded_point(&encoded_point);
            if public_key.is_some().into() {
                Ok(public_key.unwrap())
            } else {
                Err(CertificateValidationError::WrongAlgorithm)
            }
        } else {
            Err(CertificateValidationError::WrongAlgorithm)
        }
    }

    /// Perform hostname verification
    fn check_identity(
        &self,
        hostname: Option<&str>,
        cert_der: &[u8],
        algorithm: &str,
    ) -> Result<(), CertificateValidationError> {
        match algorithm.to_lowercase().as_str() {
            "https" => {
                if let Some(host) = hostname {
                    // Parse certificate to verify hostname against CN and SAN
                    let (_, cert) = x509_parser::parse_x509_certificate(cert_der).map_err(|e| {
                        CertificateValidationError::ParsingError(format!(
                            "Invalid certificate: {}",
                            e
                        ))
                    })?;

                    // Handle IPv6 brackets
                    let normalized_host = if host.starts_with('[') && host.ends_with(']') {
                        &host[1..host.len() - 1]
                    } else {
                        host
                    };

                    let mut hostname_match = false;

                    // Check Common Name (CN) in subject
                    if let Some(subject_cn) = cert.subject().iter_common_name().next() {
                        if let Ok(cn_str) = subject_cn.as_str() {
                            if cn_str == normalized_host {
                                hostname_match = true;
                            }
                        }
                    }

                    // Check Subject Alternative Names (SAN) if CN didn't match
                    if !hostname_match {
                        for ext in cert.extensions() {
                            if ext.oid == x509_parser::oid_registry::OID_X509_EXT_SUBJECT_ALT_NAME {
                                if let x509_parser::extensions::ParsedExtension::SubjectAlternativeName(san) = ext.parsed_extension() {
                                    for name in &san.general_names {
                                        match name {
                                            x509_parser::extensions::GeneralName::DNSName(dns_name) => {
                                                if dns_name.as_ref() as &str == normalized_host {
                                                    hostname_match = true;
                                                    break;
                                                }
                                            }
                                            x509_parser::extensions::GeneralName::IPAddress(ip_bytes) => {
                                                let ip_str = match ip_bytes.len() {
                                                    4 => format!("{}.{}.{}.{}", ip_bytes[0], ip_bytes[1], ip_bytes[2], ip_bytes[3]),
                                                    16 => {
                                                        // Format IPv6 as colon-separated hex groups
                                                        let mut ipv6_parts = Vec::new();
                                                        for chunk in ip_bytes.chunks(2) {
                                                            let part = u16::from_be_bytes([chunk[0], chunk.get(1).copied().unwrap_or(0)]);
                                                            ipv6_parts.push(format!("{:x}", part));
                                                        }
                                                        ipv6_parts.join(":")
                                                    }
                                                    _ => continue,
                                                };
                                                if ip_str == normalized_host {
                                                    hostname_match = true;
                                                    break;
                                                }
                                            }
                                            _ => continue,
                                        }
                                    }
                                }
                                if hostname_match {
                                    break;
                                }
                            }
                        }
                    }

                    if !hostname_match {
                        return Err(CertificateValidationError::ValidationFailed(format!(
                            "Hostname '{}' does not match certificate",
                            normalized_host
                        )));
                    }
                }
                Ok(())
            }
            _ => Err(CertificateValidationError::UnknownIdentificationAlgorithm(
                algorithm.to_string(),
            )),
        }
    }

    /// Get accepted issuers
    pub fn get_accepted_issuers(&self) -> Vec<Vec<u8>> {
        // Return empty list - we use custom validation, not CA-based
        Vec::new()
    }
}

/// Implement rustls ServerCertVerifier directly
impl ServerCertVerifier for HostnameTrustManager {
    fn verify_server_cert(
        &self,
        end_entity: &CertificateDer<'_>,
        _intermediates: &[CertificateDer<'_>],
        server_name: &ServerName<'_>,
        _ocsp_response: &[u8],
        _now: rustls::pki_types::UnixTime,
    ) -> Result<ServerCertVerified, RustlsError> {
        // Convert server_name to hostname string for our existing validation logic
        let hostname = match server_name {
            ServerName::DnsName(dns_name) => Some(dns_name.as_ref()),
            ServerName::IpAddress(ip) => {
                // Convert IP to string for hostname verification
                let ip_str = match ip {
                    rustls::pki_types::IpAddr::V4(ipv4) => {
                        format!("{:?}", ipv4) // Use Debug format since Display isn't implemented
                    },
                    rustls::pki_types::IpAddr::V6(ipv6) => {
                        format!("{:?}", ipv6) // Use Debug format since Display isn't implemented
                    },
                };
                Some(ip_str.leak() as &str) // Leak string to get 'static lifetime
            },
            _ => None,
        };
        
        // Use our existing server trust validation logic
        self.check_server_trusted(end_entity.as_ref(), "RSA", hostname)
            .map_err(|_| RustlsError::InvalidCertificate(
                rustls::CertificateError::ApplicationVerificationFailure
            ))?;
        
        Ok(ServerCertVerified::assertion())
    }
    
    fn verify_tls12_signature(
        &self,
        _message: &[u8],
        _cert: &CertificateDer<'_>,
        _dss: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, RustlsError> {
        Ok(HandshakeSignatureValid::assertion())
    }
    
    fn verify_tls13_signature(
        &self,
        _message: &[u8],
        _cert: &CertificateDer<'_>,
        _dss: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, RustlsError> {
        Ok(HandshakeSignatureValid::assertion())
    }
    
    fn supported_verify_schemes(&self) -> Vec<SignatureScheme> {
        vec![
            SignatureScheme::ECDSA_NISTP256_SHA256,
            SignatureScheme::RSA_PSS_SHA256,
            SignatureScheme::RSA_PKCS1_SHA256,
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hostname_trust_manager_factory_singleton() {
        let factory1 = HostnameTrustManagerFactory::instance();
        let factory2 = HostnameTrustManagerFactory::instance();

        // Should be the same instance (singleton pattern)
        assert!(std::ptr::eq(factory1, factory2));
    }

    #[test]
    fn test_create_trust_manager() {
        let factory = HostnameTrustManagerFactory::instance();
        let _trust_manager = factory.create_trust_manager();

        // Should create successfully without panicking
    }

    #[test]
    fn test_f1r3fly_address_extraction() {
        let manager = HostnameTrustManager::new();

        // Generate test certificate and address
        let (secret_key, public_key) = CertificateHelper::generate_key_pair(false);

        if let (Ok(cert_der), Some(expected_address)) = (
            CertificateHelper::generate_certificate(&secret_key, &public_key),
            CertificateHelper::public_address(&public_key),
        ) {
            let expected_hex = hex::encode(&expected_address);

            // Test F1r3fly address extraction
            let result = manager.extract_f1r3fly_address(&cert_der);
            assert!(
                result.is_ok(),
                "Should extract F1r3fly address successfully"
            );
            assert_eq!(result.unwrap(), expected_hex);
        }
    }

    #[test]
    fn test_client_trusted_validation() {
        let manager = HostnameTrustManager::new();

        // Generate test certificate
        let (secret_key, public_key) = CertificateHelper::generate_key_pair(false);

        if let Ok(cert_der) = CertificateHelper::generate_certificate(&secret_key, &public_key) {
            // Test client certificate validation
            let result = manager.check_client_trusted(&cert_der, "RSA");
            assert!(
                result.is_ok(),
                "Should validate client certificate successfully"
            );
        }
    }

    #[test]
    fn test_server_trusted_validation() {
        let manager = HostnameTrustManager::new();

        // Generate test certificate
        let (secret_key, public_key) = CertificateHelper::generate_key_pair(false);

        if let (Ok(cert_der), Some(expected_address)) = (
            CertificateHelper::generate_certificate(&secret_key, &public_key),
            CertificateHelper::public_address(&public_key),
        ) {
            let expected_hex = hex::encode(&expected_address);

            // Test server certificate validation with matching hostname
            let result = manager.check_server_trusted(&cert_der, "RSA", Some(&expected_hex));
            assert!(
                result.is_ok(),
                "Should validate server certificate with matching hostname"
            );

            // Test server certificate validation with non-matching hostname
            let result = manager.check_server_trusted(&cert_der, "RSA", Some("wrong_hostname"));
            assert!(
                result.is_err(),
                "Should reject server certificate with non-matching hostname"
            );
        }
    }
}

use blake2::{Blake2b, Digest};
use models::casper::v1::deploy_response::Message as DeployResponseMessage;
use models::casper::v1::deploy_service_client::DeployServiceClient;
use models::casper::v1::propose_response::Message as ProposeResponseMessage;
use models::casper::v1::propose_service_client::ProposeServiceClient;
use models::casper::ProposeQuery;
use models::casper::DeployDataProto;
use models::ByteString;
use prost::Message;
use secp256k1::{Message as Secp256k1Message, Secp256k1, SecretKey};
use typenum::U32;
use std::time::{SystemTime, UNIX_EPOCH};

/// Client for interacting with the F1r3fly API
pub struct F1r3flyApi<'a> {
    signing_key: SecretKey,
    node_host: &'a str,
    grpc_port: u16,
}

impl<'a> F1r3flyApi<'a> {
    /// Creates a new F1r3fly API client
    ///
    /// # Arguments
    ///
    /// * `signing_key` - Hex-encoded private key for signing deploys
    /// * `node_host` - Hostname or IP address of the F1r3fly node
    /// * `grpc_port` - gRPC port for the node's API service
    ///
    /// # Returns
    ///
    /// A new `F1r3flyApi` instance
    pub fn new(signing_key: &str, node_host: &'a str, grpc_port: u16) -> Self {
        F1r3flyApi {
            signing_key: SecretKey::from_slice(&hex::decode(signing_key).unwrap()).unwrap(),
            node_host,
            grpc_port,
        }
    }

    /// Deploys Rholang code to the F1r3fly node
    ///
    /// # Arguments
    ///
    /// * `rho_code` - Rholang source code to deploy
    /// * `use_bigger_phlo_price` - Whether to use a larger phlo limit
    /// * `language` - Language of the deploy (typically "rholang")
    ///
    /// # Returns
    ///
    /// The deploy ID if successful, otherwise an error
    pub async fn deploy(
        &self,
        rho_code: &str,
        use_bigger_phlo_price: bool,
        language: &str,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let phlo_limit: i64 = if use_bigger_phlo_price {
            5_000_000_000
        } else {
            50_000
        };

        // Build and sign the deployment
        let deployment = self.build_deploy_msg(rho_code.to_string(), phlo_limit, language.to_string());

        // Connect to the F1r3fly node
        let mut deploy_service_client =
            DeployServiceClient::connect(format!("http://{}:{}/", self.node_host, self.grpc_port))
                .await?;

        // Send the deploy
        let deploy_response = deploy_service_client.do_deploy(deployment).await?;
        
        // Process the response
        let deploy_message = deploy_response
            .get_ref()
            .message
            .as_ref()
            .ok_or("Deploy result not found")?;

        match deploy_message {
            DeployResponseMessage::Error(service_error) => {
                Err(service_error.clone().into())
            }
            DeployResponseMessage::Result(result) => {
                Ok(result.clone())
            }
        }
    }

    /// Sends a proposal to the network to create a new block
    ///
    /// # Returns
    ///
    /// The block hash of the proposed block if successful, otherwise an error
    pub async fn propose(&self) -> Result<String, Box<dyn std::error::Error>> {
        // Connect to the F1r3fly node's propose service
        let mut propose_client = ProposeServiceClient::connect(
            format!("http://{}:{}/", self.node_host, self.grpc_port)
        ).await?;

        // Send the propose request
        let propose_response = propose_client
            .propose(ProposeQuery { is_async: false })
            .await?
            .into_inner();
            
        // Process the response
        let message = propose_response
            .message
            .ok_or("Missing propose response")?;

        match message {
            ProposeResponseMessage::Result(block_hash) => {
                // Extract the block hash from the response
                if let Some(hash) = block_hash
                    .strip_prefix("Success! Block ")
                    .and_then(|s| s.strip_suffix(" created and added."))
                {
                    Ok(hash.to_string())
                } else {
                    Ok(block_hash) // Return the full message if we can't extract the hash
                }
            }
            ProposeResponseMessage::Error(error) => {
                Err(format!("Propose error: {:?}", error).into())
            }
        }
    }

    /// Performs a full deployment cycle: deploy and propose
    ///
    /// # Arguments
    ///
    /// * `rho_code` - Rholang source code to deploy
    /// * `use_bigger_phlo_price` - Whether to use a larger phlo limit
    /// * `language` - Language of the deploy (typically "rholang")
    ///
    /// # Returns
    ///
    /// The block hash if successful, otherwise an error
    pub async fn full_deploy(
        &self,
        rho_code: &str,
        use_bigger_phlo_price: bool,
        language: &str,
    ) -> Result<String, Box<dyn std::error::Error>> {
        // First deploy the code
        self.deploy(rho_code, use_bigger_phlo_price, language).await?;
        
        // Then propose a block
        self.propose().await
    }

    /// Builds and signs a deploy message
    ///
    /// # Arguments
    ///
    /// * `code` - Rholang source code to deploy
    /// * `phlo_limit` - Maximum amount of phlo to use for execution
    /// * `language` - Language of the deploy (typically "rholang")
    ///
    /// # Returns
    ///
    /// A signed `DeployDataProto` ready to be sent to the node
    fn build_deploy_msg(&self, code: String, phlo_limit: i64, language: String) -> DeployDataProto {
        // Get current timestamp in milliseconds
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Failed to get system time")
            .as_millis() as i64;
            
        // Create a projection with only the fields used for signature calculation
        // IMPORTANT: The language field is deliberately excluded from signature calculation
        let projection = DeployDataProto {
            term: code.clone(),
            timestamp,
            phlo_price: 1,
            phlo_limit,
            valid_after_block_number: 0,
            shard_id: "root".into(),
            language: String::new(), // Excluded from signature calculation
            sig: ByteString::new(),
            deployer: ByteString::new(),
            sig_algorithm: String::new(),
        };
        
        // Serialize the projection for hashing
        let serialized = projection.encode_to_vec();
        
        // Hash with Blake2b256
        let digest = blake2b_256_hash(&serialized);
        
        // Sign the digest with secp256k1
        let secp = Secp256k1::new();
        let message = Secp256k1Message::from_digest(digest.into());
        let signature = secp.sign_ecdsa(&message, &self.signing_key);
        
        // Get signature in DER format
        let sig_bytes = signature.serialize_der().to_vec();
        
        // Get the public key in uncompressed format
        let public_key = self.signing_key.public_key(&secp);
        let pub_key_bytes = public_key.serialize_uncompressed().to_vec();
        
        // Return the complete deploy message
        DeployDataProto {
            term: code,
            timestamp,
            phlo_price: 1,
            phlo_limit,
            valid_after_block_number: 0,
            shard_id: "root".into(),
            language,
            sig: ByteString::from(sig_bytes),
            sig_algorithm: "secp256k1".into(),
            deployer: ByteString::from(pub_key_bytes),
        }
    }
}

/// Computes a Blake2b 256-bit hash of the provided data
fn blake2b_256_hash(data: &[u8]) -> [u8; 32] {
    let mut blake = Blake2b::<U32>::new();
    blake.update(data);
    let hash = blake.finalize();
    let mut result = [0u8; 32];
    result.copy_from_slice(&hash);
    result
}

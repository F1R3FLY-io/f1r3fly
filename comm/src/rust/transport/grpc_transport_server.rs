// See comm/src/main/scala/coop/rchain/comm/transport/GrpcTransportServer.scala

use async_trait::async_trait;
use rustls::pki_types::pem::PemObject;
use rustls::pki_types::{CertificateDer, PrivateKeyDer};
use rustls::ServerConfig;
use std::collections::HashMap;
use std::future::Future;
use std::path::Path;
use std::pin::Pin;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::sync::{Mutex, OnceCell};
use tokio::task::JoinHandle;

use models::routing::Protocol;

use crate::rust::{
    errors::CommError,
    peer_node::PeerNode,
    rp::rp_conf::RPConf,
    transport::{
        communication_response::CommunicationResponse,
        grpc_transport_receiver::{GrpcTransportReceiver, MessageBuffers, MessageHandlers},
        hostname_trust_manager_factory::HostnameTrustManagerFactory,
        packet_ops::StreamCache,
        stream_handler::StreamHandler,
        transport_layer::Blob,
    },
};

/// Cancelable resource that can be cancelled/stopped
pub type Cancelable = JoinHandle<()>;

/// Type alias for dispatch function that handles Protocol messages
pub type DispatchFn = Arc<
    dyn Fn(
            Protocol,
        )
            -> Pin<Box<dyn Future<Output = Result<CommunicationResponse, CommError>> + Send>>
        + Send
        + Sync,
>;

/// Type alias for stream handler function that processes Blob messages  
pub type HandleStreamedFn =
    Arc<dyn Fn(Blob) -> Pin<Box<dyn Future<Output = Result<(), CommError>> + Send>> + Send + Sync>;

/// Transport Layer Server trait for handling incoming gRPC connections
#[async_trait]
pub trait TransportLayerServer {
    /// Set up message handlers and start receiving connections
    ///
    /// This method configures the server with handlers for different types of messages
    /// and starts listening for incoming gRPC connections.
    async fn handle_receive(
        &self,
        dispatch: DispatchFn,
        handle_streamed: HandleStreamedFn,
    ) -> Result<Cancelable, CommError>;
}

/// GrpcTransportServer - gRPC server implementation for handling transport layer messages
#[derive(Debug, Clone)]
pub struct GrpcTransportServer {
    /// Local peer node
    pub local_peer: RPConf,
    /// Network identifier for this node
    pub network_id: String,
    /// Port number to bind the gRPC server to
    pub port: u16,
    /// TLS certificate in PEM format
    pub cert: String,
    /// TLS private key in PEM format
    pub key: String,
    /// Maximum message size for regular gRPC messages in bytes
    pub max_message_size: i32,
    /// Maximum message size for streamed messages in bytes
    pub max_stream_message_size: u64,
    /// Number of parallel message processing tasks
    pub parallelism: usize,
    /// Cache to store received partial data (streaming packets)
    pub cache: StreamCache,
}

impl GrpcTransportServer {
    /// Create a new GrpcTransportServer instance
    pub fn new(
        local_peer: RPConf,
        network_id: String,
        port: u16,
        cert: String,
        key: String,
        max_message_size: i32,
        max_stream_message_size: u64,
        parallelism: usize,
    ) -> Self {
        Self {
            local_peer,
            network_id,
            port,
            cert,
            key,
            max_message_size,
            max_stream_message_size,
            parallelism,
            // Create cache for storing received partial data (streaming packets)
            cache: Arc::new(dashmap::DashMap::new()),
        }
    }

    /// Create a GrpcTransportServer by reading certificate and key from files
    pub async fn acquire_server(
        network_id: String,
        port: u16,
        cert_path: &Path,
        key_path: &Path,
        max_message_size: i32,
        max_stream_message_size: u64,
        parallelism: usize,
        local_peer: RPConf,
    ) -> Result<TransportServer, CommError> {
        // Read certificate file
        let cert = tokio::fs::read_to_string(cert_path).await.map_err(|e| {
            CommError::ConfigError(format!("Failed to read cert file {:?}: {}", cert_path, e))
        })?;

        // Read key file
        let key = tokio::fs::read_to_string(key_path).await.map_err(|e| {
            CommError::ConfigError(format!("Failed to read key file {:?}: {}", key_path, e))
        })?;

        let server = Self::new(
            local_peer,
            network_id,
            port,
            cert,
            key,
            max_message_size,
            max_stream_message_size,
            parallelism,
        );

        Ok(TransportServer::new(server))
    }

    /// Get certificate as input stream
    pub fn cert_input_stream(&self) -> std::io::Cursor<Vec<u8>> {
        std::io::Cursor::new(self.cert.as_bytes().to_vec())
    }

    /// Get private key as input stream
    pub fn key_input_stream(&self) -> std::io::Cursor<Vec<u8>> {
        std::io::Cursor::new(self.key.as_bytes().to_vec())
    }

    /// Get or create the server SSL context with F1r3fly hostname verification
    ///
    /// The verification process:
    /// 1. Extracts F1r3fly public address from client certificate  
    /// 2. Performs hostname/identity verification using "https" algorithm
    /// 3. Validates certificate structure and public key format
    /// 4. Requires client certificates
    pub async fn get_server_ssl_context(&self) -> Result<ServerConfig, CommError> {
        // Parse server certificate and key for mutual TLS
        let cert_der: Vec<CertificateDer> =
            CertificateDer::pem_reader_iter(&mut self.cert_input_stream())
                .collect::<Result<Vec<_>, _>>()
                .map_err(|e| {
                    CommError::ConfigError(format!("Failed to parse server certificate: {}", e))
                })?;

        let key_der =
            PrivateKeyDer::from_pem_reader(&mut self.key_input_stream()).map_err(|e| {
                CommError::ConfigError(format!("Failed to parse server private key: {}", e))
            })?;

        // Create client cert verifier using factory
        let client_cert_verifier =
            HostnameTrustManagerFactory::instance().create_client_cert_verifier();

        // Build the server config with F1r3fly client authentication
        let config = ServerConfig::builder()
            .with_client_cert_verifier(client_cert_verifier)
            .with_single_cert(cert_der, key_der)
            .map_err(|e| {
                CommError::ConfigError(format!("Failed to configure server TLS: {}", e))
            })?;

        Ok(config)
    }
}

/// TransportServer manages the lifecycle of a GrpcTransportServer
///
/// Provides atomic start/stop operations to ensure only one server instance
/// runs at a time, matching the Scala implementation's behavior.
pub struct TransportServer {
    server: GrpcTransportServer,
    server_handle: Arc<Mutex<Option<Cancelable>>>,
    is_running: Arc<AtomicBool>,
}

impl TransportServer {
    /// Create a new TransportServer
    pub fn new(server: GrpcTransportServer) -> Self {
        Self {
            server,
            server_handle: Arc::new(Mutex::new(None)),
            is_running: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Start the server with the given message handlers
    ///
    /// This method is idempotent - if the server is already running, it returns immediately.
    /// If a previous server instance exists, it will be stopped before starting the new one.
    pub async fn start(
        &self,
        dispatch: DispatchFn,
        handle_streamed: HandleStreamedFn,
    ) -> Result<(), CommError> {
        // Check if already running
        if self.is_running.load(Ordering::Acquire) {
            return Ok(());
        }

        // Get exclusive access to server handle
        let mut handle_guard = self.server_handle.lock().await;

        // Double-check pattern: verify still not running after acquiring lock
        if self.is_running.load(Ordering::Acquire) {
            return Ok(());
        }

        // Stop any existing server
        if let Some(old_handle) = handle_guard.take() {
            old_handle.abort();
        }

        // Start new server
        let new_handle = self
            .server
            .handle_receive(dispatch, handle_streamed)
            .await?;

        // Update state atomically
        *handle_guard = Some(new_handle);
        self.is_running.store(true, Ordering::Release);

        Ok(())
    }

    /// Stop the server
    ///
    /// This method is idempotent - if the server is not running, it returns immediately.
    pub async fn stop(&self) -> Result<(), CommError> {
        // Check if not running
        if !self.is_running.load(Ordering::Acquire) {
            return Ok(());
        }

        // Get exclusive access to server handle
        let mut handle_guard = self.server_handle.lock().await;

        // Double-check pattern: verify still running after acquiring lock
        if !self.is_running.load(Ordering::Acquire) {
            return Ok(());
        }

        // Stop server if running
        if let Some(handle) = handle_guard.take() {
            handle.abort();
        }

        // Update state atomically
        self.is_running.store(false, Ordering::Release);

        Ok(())
    }

    /// Check if the server is currently running
    pub fn is_running(&self) -> bool {
        self.is_running.load(Ordering::Acquire)
    }
}

#[async_trait]
impl TransportLayerServer for GrpcTransportServer {
    /// Set up message handlers and start receiving connections
    ///
    /// This method configures the server with handlers for different types of messages
    /// and starts listening for incoming gRPC connections.
    async fn handle_receive(
        &self,
        dispatch: DispatchFn,
        handle_streamed: HandleStreamedFn,
    ) -> Result<Cancelable, CommError> {
        // Create SSL context for the server
        let server_ssl_context = self.get_server_ssl_context().await?;

        // Create buffers map for per-peer message queues
        let buffers_map: Arc<Mutex<HashMap<PeerNode, Arc<OnceCell<MessageBuffers>>>>> =
            Arc::new(Mutex::new(HashMap::new()));

        // Create message handlers that bridge to the provided dispatch functions
        let message_handlers: MessageHandlers = (
            // Handler for Send messages (Protocol messages)
            Arc::new({
                let dispatch = dispatch.clone();
                move |send_msg| {
                    let protocol = send_msg.msg;
                    let dispatch = dispatch.clone();

                    Box::pin(async move {
                        // Execute the dispatch function
                        match dispatch(protocol).await {
                            Ok(_communication_response) => {
                                // TODO: Add metrics increment for "dispatched.messages"
                                // Metrics[F].incrementCounter("dispatched.messages")
                                Ok(())
                            }
                            Err(e) => {
                                log::error!("Sending gRPC message failed: {}", e);
                                Err(e)
                            }
                        }
                    })
                }
            }),
            // Handler for StreamMessage (Blob streaming)
            Arc::new({
                let handle_streamed = handle_streamed.clone();
                let cache = self.cache.clone();
                move |stream_msg| {
                    let cache = cache.clone();
                    let handle_streamed = handle_streamed.clone();

                    Box::pin(async move {
                        match StreamHandler::restore(&stream_msg, &cache).await {
                            Ok(blob) => {
                                // Execute the stream handler function
                                match handle_streamed(blob).await {
                                    Ok(()) => {
                                        // TODO: Add metrics increment for "dispatched.packets"
                                        // Metrics[F].incrementCounter("dispatched.packets")
                                        Ok(())
                                    }
                                    Err(e) => {
                                        log::error!("Error in stream handler: {}", e);
                                        Err(e)
                                    }
                                }
                            }
                            Err(e) => {
                                log::error!(
                                    "Could not restore data from file while handling stream for key {}: {}",
                                    stream_msg.key,
                                    e
                                );
                                Err(e)
                            }
                        }
                    })
                }
            }),
        );

        // Create and start the gRPC transport receiver
        let server_task = GrpcTransportReceiver::create(
            self.network_id.clone(),
            self.local_peer.clone(),
            self.port,
            server_ssl_context,
            self.cert.clone(),
            self.key.clone(),
            self.max_message_size,
            self.max_stream_message_size,
            buffers_map,
            message_handlers,
            self.parallelism,
            self.cache.clone(),
        )
        .await?;

        Ok(server_task)
    }
}

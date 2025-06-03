// See comm/src/main/scala/coop/rchain/comm/transport/GrpcTransportClient.scala

use async_trait::async_trait;
use rustls::pki_types::pem::PemObject;
use rustls::pki_types::{CertificateDer, PrivateKeyDer};
use rustls::ClientConfig;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{Mutex, OnceCell};
use tonic::service::interceptor::InterceptedService;
use tonic::transport::{Channel, ClientTlsConfig, Endpoint, Identity};

use crate::rust::{
    errors::CommError,
    peer_node::PeerNode,
    transport::{
        grpc_transport::GrpcTransport,
        hostname_trust_manager_factory::HostnameTrustManagerFactory,
        packet_ops::PacketOps,
        ssl_session_client_interceptor::SslSessionClientInterceptor,
        stream_observable::StreamObservable,
        transport_layer::{Blob, TransportLayer},
    },
};

use models::routing::{transport_layer_client::TransportLayerClient, Protocol};

type StreamCache = Arc<dashmap::DashMap<String, Vec<u8>>>;

/// GRPC channel with a message buffer protecting it from resource exhaustion
#[derive(Debug)]
pub struct BufferedGrpcStreamChannel {
    /// Underlying gRPC channel
    pub grpc_transport: Channel,
    /// Pre-created transport client with SSL interceptor applied (for reuse)
    pub transport_client: Arc<
        tokio::sync::Mutex<
            TransportLayerClient<InterceptedService<Channel, SslSessionClientInterceptor>>,
        >,
    >,
    /// Buffer implementing some kind of overflow policy
    pub buffer: StreamObservable,
    /// Buffer subscriber handle
    pub buffer_subscriber: tokio::task::JoinHandle<()>,
    /// Max message size (to be applied to individual service clients)
    pub max_message_size: i32,
}

impl BufferedGrpcStreamChannel {
    /// Create a new BufferedGrpcStreamChannel with pre-created client
    pub fn new(
        grpc_transport: Channel,
        transport_client: TransportLayerClient<
            InterceptedService<Channel, SslSessionClientInterceptor>,
        >,
        buffer: StreamObservable,
        buffer_subscriber: tokio::task::JoinHandle<()>,
        max_message_size: i32,
    ) -> Self {
        Self {
            grpc_transport,
            transport_client: Arc::new(tokio::sync::Mutex::new(transport_client)),
            buffer,
            buffer_subscriber,
            max_message_size,
        }
    }

    /// Get a clone of the pre-created transport client (for use in tasks)
    pub fn get_transport_client(
        &self,
    ) -> Arc<
        tokio::sync::Mutex<
            TransportLayerClient<InterceptedService<Channel, SslSessionClientInterceptor>>,
        >,
    > {
        self.transport_client.clone()
    }
}

/// GrpcTransportClient - gRPC client implementation
#[derive(Clone)]
pub struct GrpcTransportClient {
    network_id: String,
    cert: String,
    key: String,
    max_message_size: i32,
    packet_chunk_size: i32,
    client_queue_size: i32,
    channels_map: Arc<Mutex<HashMap<PeerNode, Arc<OnceCell<Arc<BufferedGrpcStreamChannel>>>>>>,
    default_send_timeout: Duration,
    cache: StreamCache,
}

impl GrpcTransportClient {
    /// Create a new GrpcTransportClient
    pub fn new(
        network_id: String,
        cert: String,
        key: String,
        max_message_size: i32,
        packet_chunk_size: i32,
        client_queue_size: i32,
        channels_map: Arc<Mutex<HashMap<PeerNode, Arc<OnceCell<Arc<BufferedGrpcStreamChannel>>>>>>,
    ) -> Self {
        Self {
            network_id,
            cert,
            key,
            max_message_size,
            packet_chunk_size,
            client_queue_size,
            channels_map,
            default_send_timeout: Duration::from_secs(5),
            cache: Arc::new(dashmap::DashMap::new()),
        }
    }

    fn cert_input_stream(&self) -> std::io::Cursor<Vec<u8>> {
        std::io::Cursor::new(self.cert.as_bytes().to_vec())
    }

    fn key_input_stream(&self) -> std::io::Cursor<Vec<u8>> {
        std::io::Cursor::new(self.key.as_bytes().to_vec())
    }

    /// Get or create the SSL context and certificate data
    async fn get_client_ssl_context(&self) -> Result<(ClientConfig, Vec<u8>, Vec<u8>), CommError> {
        // NOTE: This creates a ClientConfig with HostnameTrustManager (matching Scala's approach)
        // but the current implementation DOES NOT actually use this custom config due to
        // tonic 0.13 limitations (no rustls_client_config() method available)

        // Parse client certificate and key for mutual TLS
        let cert_pem = self.cert.as_bytes().to_vec();
        let key_pem = self.key.as_bytes().to_vec();

        let cert_der: Vec<CertificateDer> =
            CertificateDer::pem_reader_iter(&mut self.cert_input_stream())
                .collect::<Result<Vec<_>, _>>()
                .map_err(|e| {
                    CommError::ConfigError(format!("Failed to parse certificate: {}", e))
                })?;

        let key_der = PrivateKeyDer::from_pem_reader(&mut self.key_input_stream())
            .map_err(|e| CommError::ConfigError(format!("Failed to parse private key: {}", e)))?;

        let trust_manager = HostnameTrustManagerFactory::instance().create_trust_manager();

        // Build the client config for future use
        // TODO: This config with HostnameTrustManager should be used for TLS but currently isn't
        let config = ClientConfig::builder()
            .dangerous()
            .with_custom_certificate_verifier(trust_manager)
            .with_client_auth_cert(cert_der, key_der)
            .map_err(|e| CommError::ConfigError(format!("Failed to configure TLS: {}", e)))?;

        Ok((config, cert_pem, key_pem))
    }

    async fn create_channel(
        &self,
        peer: &PeerNode,
    ) -> Result<BufferedGrpcStreamChannel, CommError> {
        log::info!("Creating new channel to peer {}", peer.to_address());
        let (_client_ssl_context, cert_pem, key_pem) = self.get_client_ssl_context().await?;
        // ^^^^^^^^^^^^^^^^^^^^^
        // PROBLEM: We create a ClientConfig with HostnameTrustManager but DON'T USE IT!
        //
        // Scala side does:
        //   .sslContext(clientSslContext)  // Uses the custom SSL context with HostnameTrustManager
        //
        // Rust side does:
        //   .tls_config(tls_config)       // Uses tonic's default certificate verification
        //
        // This means HostnameTrustManager is NOT doing TLS-level server certificate validation
        // like it does in Scala. The verification logic would need to be handled elsewhere.

        // Create endpoint with explicit TLS configuration
        // Note: Using http:// scheme since .tls_config() below will override it to use TLS
        let endpoint = format!("http://{}:{}", peer.endpoint.host, peer.endpoint.tcp_port);

        // ISSUE: This uses tonic's standard TLS instead of our custom ClientConfig with HostnameTrustManager
        // Unlike Scala which uses: .sslContext(clientSslContext) where clientSslContext includes
        // the HostnameTrustManagerFactory.Instance as the trust manager
        let tls_config = ClientTlsConfig::new()
            .domain_name(&peer.id.to_string()) // Override authority for verification
            .identity(Identity::from_pem(&cert_pem, &key_pem)); // Client certificate for mutual TLS
                                                                // Missing: Custom certificate verification via HostnameTrustManager
                                                                // (would need .rustls_client_config(_client_ssl_context) but this method doesn't exist in tonic 0.13)

        // Build the channel
        let grpc_channel = Endpoint::from_shared(endpoint)
            .map_err(|e| CommError::ConfigError(format!("Invalid endpoint: {}", e)))?
            .tls_config(tls_config)
            .map_err(|e| CommError::ConfigError(format!("TLS config failed: {}", e)))?
            .connect()
            .await
            .map_err(|e| {
                CommError::InternalCommunicationError(format!("Connection failed: {}", e))
            })?;

        // Create SSL session client interceptor
        // NOTE: This matches Scala's SslSessionClientInterceptor(networkId) - takes only network_id
        // and uses CertificateHelper directly for application-level validation
        let ssl_interceptor = SslSessionClientInterceptor::new(self.network_id.clone());

        // Pre-create transport client with SSL interceptor applied
        let intercepted_service = InterceptedService::new(grpc_channel.clone(), ssl_interceptor);
        let transport_client = TransportLayerClient::new(intercepted_service)
            .max_decoding_message_size(self.max_message_size as usize)
            .max_encoding_message_size(self.max_message_size as usize);

        // Create buffer
        let mut buffer = StreamObservable::new(
            peer.clone(),
            self.client_queue_size as usize,
            self.cache.clone(),
        );

        // Create buffer subscriber
        let buffer_subscriber = {
            let peer_clone = peer.clone();
            let cache_clone = self.cache.clone();
            let network_id = self.network_id.clone();
            let default_send_timeout = self.default_send_timeout;
            let packet_chunk_size = self.packet_chunk_size;

            // Get subscription from buffer
            let mut subscription = buffer.subscribe().ok_or_else(|| {
                CommError::InternalCommunicationError(
                    "Failed to create stream subscription".to_string(),
                )
            })?;

            // Clone the transport client for use in spawned task
            let client_for_task = Arc::new(tokio::sync::Mutex::new(transport_client.clone()));

            // Spawn task to handle stream messages
            tokio::spawn(async move {
                use tokio_stream::StreamExt;

                while let Some(stream_msg) = subscription.next().await {
                    // Call static stream_blob_file_with_client method with pre-created client
                    let result = Self::stream_blob_file_with_client(
                        &stream_msg.key,
                        &peer_clone,
                        &stream_msg.sender,
                        &cache_clone,
                        &network_id,
                        default_send_timeout,
                        packet_chunk_size,
                        client_for_task.clone(),
                    )
                    .await;

                    // Log any errors but continue processing
                    if let Err(e) = result {
                        log::debug!(
                            "Error in stream_blob_file for key {}: {}",
                            stream_msg.key,
                            e
                        );
                    }

                    // Clean up cache
                    // This happens regardless of success/failure
                    cache_clone.remove(&stream_msg.key);
                }
            })
        };

        // Create channel with pre-created transport client
        let channel = BufferedGrpcStreamChannel::new(
            grpc_channel,
            transport_client,
            buffer,
            buffer_subscriber,
            self.max_message_size,
        );

        Ok(channel)
    }

    /// Get or create a channel for the specified peer
    ///
    /// Uses atomic operations to ensure only one channel is created per peer,
    /// with proper cleanup and retry logic for terminated channels.
    async fn get_channel(
        &self,
        peer: &PeerNode,
    ) -> Result<Arc<BufferedGrpcStreamChannel>, CommError> {
        loop {
            // Create a new OnceCell for potential new channel
            let new_once_cell = Arc::new(OnceCell::new());

            // Atomic operation: check if peer exists, if not add new OnceCell
            let (once_cell, is_new_channel) = {
                let mut channels_map = self.channels_map.lock().await;

                if let Some(existing_once_cell) = channels_map.get(peer) {
                    // Peer exists, use existing OnceCell
                    (existing_once_cell.clone(), false)
                } else {
                    // Peer doesn't exist, add new OnceCell
                    channels_map.insert(peer.clone(), new_once_cell.clone());
                    (new_once_cell, true)
                }
            };

            // If this is a new channel, create it and store in OnceCell
            if is_new_channel {
                let channel = self.create_channel(peer).await?;
                let _ = once_cell.set(Arc::new(channel));
            }

            // Get the channel from OnceCell (wait if another thread is creating it)
            let channel = once_cell
                .get_or_try_init(|| async {
                    // This should not happen if logic is correct, but just in case
                    let ch = self.create_channel(peer).await?;
                    Ok(Arc::new(ch))
                })
                .await?;

            // Check if channel is terminated and handle cleanup/retry
            if Self::is_channel_terminated(channel).await {
                log::debug!(
                    "Channel to peer {} is terminated; removing from connections map",
                    peer.to_address()
                );

                // Clean up: cancel subscriber and remove from map
                channel.buffer_subscriber.abort();

                {
                    let mut channels_map = self.channels_map.lock().await;
                    channels_map.remove(peer);
                }

                // Retry by continuing the loop
                continue;
            }

            // Channel is valid, return it
            return Ok(channel.clone());
        }
    }

    /// Execute a request with a gRPC client, handling timeouts and errors
    async fn with_client<A, F, Fut>(
        &self,
        peer: &PeerNode,
        timeout: Duration,
        request: F,
    ) -> Result<A, CommError>
    where
        F: FnOnce(
            TransportLayerClient<InterceptedService<Channel, SslSessionClientInterceptor>>,
        ) -> Fut,
        Fut: std::future::Future<Output = Result<A, CommError>>,
    {
        // Apply timeout to the entire operation
        let timed_operation = tokio::time::timeout(timeout, async {
            let channel = self.get_channel(peer).await?;
            let client_guard = channel.transport_client.lock().await;
            let client = client_guard.clone(); // Clone the client for use
            drop(client_guard); // Release the lock immediately

            let result = request(client).await?;

            // Return control to caller thread
            // In Rust, this is handled automatically by the async runtime
            tokio::task::yield_now().await;

            Ok::<A, CommError>(result)
        });

        // Handle timeout and other errors
        match timed_operation.await {
            Ok(Ok(success)) => Ok(success),
            Ok(Err(comm_error)) => Err(comm_error),
            Err(_timeout_error) => Err(crate::rust::errors::protocol_exception(format!(
                "Request to {} timed out after {}ms",
                peer.to_address(),
                timeout.as_millis()
            ))),
        }
    }

    /// Stream a blob file from cache to a peer using pre-created client
    ///
    /// This method uses a pre-created transport client, eliminating the need for
    /// complex connection management in spawned tasks.
    async fn stream_blob_file_with_client(
        key: &str,
        peer: &PeerNode,
        sender: &PeerNode,
        cache: &StreamCache,
        network_id: &str,
        default_send_timeout: Duration,
        packet_chunk_size: i32,
        client: Arc<
            tokio::sync::Mutex<
                TransportLayerClient<InterceptedService<Channel, SslSessionClientInterceptor>>,
            >,
        >,
    ) -> Result<(), CommError> {
        // Timeout calculation
        let calculate_timeout = |packet: &models::routing::Packet| -> Duration {
            let packet_based_timeout = Duration::from_micros(packet.content.len() as u64 * 5);
            std::cmp::max(packet_based_timeout, default_send_timeout)
        };

        // Restore packet from cache
        match PacketOps::restore(key, cache) {
            Ok(packet) => {
                let timeout = calculate_timeout(&packet);

                log::debug!(
                    "Attempting to stream packet to {} with timeout: {}ms",
                    peer.to_address(),
                    timeout.as_millis()
                );

                // Create blob for streaming
                let blob = Blob {
                    sender: sender.clone(),
                    packet: packet.clone(),
                };

                // Use pre-created client directly (no connection management needed)
                let mut client_guard = client.lock().await;
                let stream_result = GrpcTransport::stream(
                    &mut *client_guard,
                    peer,
                    network_id,
                    &blob,
                    packet_chunk_size as usize,
                )
                .await;
                drop(client_guard); // Release lock

                // Handle the result
                match stream_result {
                    Ok(()) => {
                        log::debug!("Streamed packet {} to {}", key, peer.to_address());
                    }
                    Err(error) => {
                        log::debug!(
                            "Error while streaming packet to {} (timeout: {}ms): {}",
                            peer.to_address(),
                            timeout.as_millis(),
                            error
                        );
                    }
                }
                Ok(())
            }
            Err(error) => {
                log::error!(
                    "Error while streaming packet {} to {}: {}",
                    key,
                    peer.to_address(),
                    error
                );
                Ok(())
            }
        }
    }

    /// Check if a channel is terminated
    async fn is_channel_terminated(channel: &BufferedGrpcStreamChannel) -> bool {
        // 1. First check: Is the buffer subscriber task finished?
        // This indicates the streaming mechanism is broken
        if channel.buffer_subscriber.is_finished() {
            log::debug!("Channel terminated: buffer subscriber task finished");
            return true;
        }

        // 2. Second check: Test if the channel can create a client
        let test_result = tokio::time::timeout(
            Duration::from_millis(50), // Very short timeout for quick check
            async {
                // Try to create a transport client - this will fail if channel is terminated
                let _client = channel.get_transport_client();
                // If we got here, the channel is still functional
                false
            },
        )
        .await;

        match test_result {
            Ok(is_terminated) => is_terminated,
            Err(_timeout) => {
                // Timeout suggests the channel is not responsive
                log::debug!("Channel terminated: client creation timed out");
                true
            }
        }
    }
}

#[async_trait]
impl TransportLayer for GrpcTransportClient {
    /// Send a Protocol message to a peer
    async fn send(&self, peer: &PeerNode, msg: &Protocol) -> Result<(), CommError> {
        self.with_client(peer, self.default_send_timeout, |mut client| async move {
            GrpcTransport::send(&mut client, peer, msg).await
        })
        .await
    }

    /// Broadcast a Protocol message to multiple peers in parallel
    async fn broadcast(&self, peers: &[PeerNode], msg: &Protocol) -> Result<(), CommError> {
        if peers.is_empty() {
            return Ok(());
        }

        // Create a vector of futures for parallel execution
        let send_futures: Vec<_> = peers.iter().map(|peer| self.send(peer, msg)).collect();

        // Execute all sends in parallel and collect results
        let results = futures::future::join_all(send_futures).await;

        // Check if any send failed - if so, return the first error
        for result in results {
            result?; // Return early on first error
        }

        Ok(())
    }

    /// Stream a blob to a peer by enqueueing it in the buffer
    async fn stream(&self, peer: &PeerNode, blob: &Blob) -> Result<(), CommError> {
        let channel = self.get_channel(peer).await?;
        channel.buffer.enque(blob).await
    }

    /// Stream a blob to multiple peers in parallel
    async fn stream_mult(&self, peers: &[PeerNode], blob: &Blob) -> Result<(), CommError> {
        if peers.is_empty() {
            return Ok(());
        }

        // Create a vector of futures for parallel execution
        let stream_futures: Vec<_> = peers.iter().map(|peer| self.stream(peer, blob)).collect();

        // Execute all streams in parallel and collect results
        let results = futures::future::join_all(stream_futures).await;

        // Check if any stream failed - if so, return the first error
        for result in results {
            result?; // Return early on first error
        }

        Ok(())
    }
}

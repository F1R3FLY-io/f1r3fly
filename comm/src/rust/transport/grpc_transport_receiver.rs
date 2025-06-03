// See comm/src/main/scala/coop/rchain/comm/transport/GrpcTransportReceiver.scala

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{Mutex, OnceCell};
use tokio::task::JoinHandle;

use crate::rust::{errors::CommError, peer_node::PeerNode};
use models::routing::transport_layer_server::{TransportLayer, TransportLayerServer};

use super::limited_buffer::FlumeLimitedBuffer;
use super::messages::{Send as CommSend, StreamMessage};
use super::packet_ops::StreamCache;
use super::ssl_session_server_interceptor::SslSessionServerInterceptor;

/// Type alias for message buffers
pub type MessageBuffers = (
    FlumeLimitedBuffer<CommSend>,
    FlumeLimitedBuffer<StreamMessage>,
    JoinHandle<()>, // Cancelable equivalent
);

/// Type alias for message handlers
pub type MessageHandlers = (
    Arc<dyn Fn(CommSend) -> Result<(), CommError> + Send + Sync>,
    Arc<dyn Fn(StreamMessage) -> Result<(), CommError> + Send + Sync>,
);

/// GrpcTransportReceiver for handling incoming gRPC messages
pub struct GrpcTransportReceiver;

impl GrpcTransportReceiver {
    /// Create a new gRPC transport receiver
    pub async fn create(
        network_id: String,
        port: u16,
        server_ssl_context: rustls::ServerConfig,
        max_message_size: i32,
        max_stream_message_size: u64,
        buffers_map: Arc<Mutex<HashMap<PeerNode, Arc<OnceCell<MessageBuffers>>>>>,
        message_handlers: MessageHandlers,
        parallelism: usize,
        cache: StreamCache,
    ) -> Result<JoinHandle<()>, CommError> {
        use std::net::SocketAddr;
        use tonic::transport::Server;

        let addr: SocketAddr = format!("0.0.0.0:{}", port)
            .parse()
            .map_err(|e| CommError::ConfigError(format!("Invalid address: {}", e)))?;

        // Create SSL session server interceptor
        let ssl_interceptor = SslSessionServerInterceptor::new(network_id.clone());

        // TODO: Create service implementation (equivalent to RoutingGrpcMonix.bindService(service, mainScheduler))

        // Create the gRPC server with interceptor
        let server_task = tokio::spawn(async move {
            log::info!("Starting gRPC transport receiver on {}", addr);

            let _server = Server::builder()
                // Request timeout (30s): Maximum time for a single gRPC request to complete.
                // Prevents hanging requests from consuming resources indefinitely.
                // Essential for blockchain P2P networks where nodes can be slow or unresponsive.
                // 30 seconds allows time for large block transfers but prevents infinite waits.
                .timeout(std::time::Duration::from_secs(30))
                // TCP keepalive (10 minutes): TCP-level keepalive to detect dead connections.
                // Critical for P2P reliability - detects if peer nodes crash or become unreachable.
                // Enables resource cleanup by closing zombie connections that consume memory.
                // Essential for maintaining healthy peer connections in distributed networks.
                .tcp_keepalive(Some(std::time::Duration::from_secs(600)))
                // TCP nodelay (true): Disables Nagle's algorithm for immediate packet transmission.
                // Important for blockchain consensus which requires fast message propagation.
                // Ensures block proposals and votes have minimal delay (vs 200ms+ with buffering).
                // Optimizes small message performance which is common in blockchain protocols.
                .tcp_nodelay(true)
                // HTTP/2 keepalive interval (30s): Sends HTTP/2 PING frames every 30 seconds.
                // Essential for connection validation - ensures gRPC connections are actually alive.
                // Critical for proxy traversal as many deployments go through load balancers.
                // Provides faster failure detection than TCP keepalive (30s vs 10 minutes).
                .http2_keepalive_interval(Some(std::time::Duration::from_secs(30)))
                // HTTP/2 keepalive timeout (5s): How long to wait for PING response.
                // Short timeout for quick failure detection - don't wait for clearly dead connections.
                // Enables resource protection by freeing up connections quickly for new peers.
                // Improves network efficiency by avoiding sends to unresponsive peers.
                .http2_keepalive_timeout(Some(std::time::Duration::from_secs(5)));
            // TODO: Add service and interceptor:
            // .add_service(
            //     transport_layer_service
            //         .max_decoding_message_size(max_message_size as usize)
            //         .max_encoding_message_size(max_message_size as usize)
            //         .with_interceptor(ssl_interceptor)
            // )
            // TODO: Add TLS configuration
            // .tls_config(tls_config)
            // .serve(addr)
            // .await;

            // For now, just sleep to keep the task alive
            tokio::time::sleep(std::time::Duration::from_secs(3600)).await;
        });

        Ok(server_task)
    }
}

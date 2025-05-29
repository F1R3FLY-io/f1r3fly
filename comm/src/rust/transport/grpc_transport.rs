// See comm/src/main/scala/coop/rchain/comm/transport/GrpcTransport.scala

use tokio_stream::wrappers::ReceiverStream;
use tonic::transport::Channel;
use tonic::{Request, Status};

use models::routing::tl_response::Payload;
use models::routing::transport_layer_client::TransportLayerClient;
use models::routing::{Protocol, TlRequest, TlResponse};

use crate::rust::{
    errors::{self, CommError},
    peer_node::PeerNode,
    transport::{chunker::Chunker, transport_layer::Blob},
};

/// GrpcTransport module providing send and stream functionality
pub struct GrpcTransport;

impl GrpcTransport {
    /// Error pattern matching: Check if error indicates peer unavailable
    fn is_peer_unavailable(error: &Status) -> bool {
        matches!(error.code(), tonic::Code::Unavailable)
    }

    /// Error pattern matching: Check if error indicates timeout
    fn is_peer_timeout(error: &Status) -> bool {
        matches!(error.code(), tonic::Code::DeadlineExceeded)
    }

    /// Error pattern matching: Check if error indicates wrong network
    fn is_peer_wrong_network(error: &Status) -> Option<String> {
        if matches!(error.code(), tonic::Code::PermissionDenied) {
            Some(error.message().to_string())
        } else {
            None
        }
    }

    /// Error pattern matching: Check if error indicates message too large
    fn is_peer_message_too_large(error: &Status) -> bool {
        matches!(error.code(), tonic::Code::ResourceExhausted)
    }

    /// Process TLResponse payload
    fn process_response(
        peer: &PeerNode,
        response: Result<TlResponse, Status>,
    ) -> Result<(), CommError> {
        let tl_response = Self::process_error(peer, response)?;

        match tl_response.payload {
            Some(Payload::Ack(_)) => Ok(()),
            Some(Payload::InternalServerError(ise)) => {
                let error_msg = String::from_utf8_lossy(&ise.error);
                Err(errors::internal_communication_error(format!(
                    "Got response: {}",
                    error_msg
                )))
            }
            None => Err(errors::internal_communication_error(
                "Empty response payload".to_string(),
            )),
        }
    }

    /// Process errors by mapping gRPC Status to CommError
    fn process_error<R>(peer: &PeerNode, response: Result<R, Status>) -> Result<R, CommError> {
        response.map_err(|error| {
            if Self::is_peer_timeout(&error) {
                errors::timeout()
            } else if Self::is_peer_unavailable(&error) {
                errors::peer_unavailable(peer.endpoint.host.clone())
            } else if Self::is_peer_message_too_large(&error) {
                errors::message_too_large(peer.endpoint.host.clone())
            } else if let Some(msg) = Self::is_peer_wrong_network(&error) {
                errors::wrong_network(peer.endpoint.host.clone(), msg)
            } else {
                errors::protocol_exception(error.to_string())
            }
        })
    }

    /// Create a transport layer client for the given peer
    pub async fn create_client(
        peer: &PeerNode,
    ) -> Result<TransportLayerClient<Channel>, CommError> {
        let endpoint = format!("http://{}:{}", peer.endpoint.host, peer.endpoint.tcp_port);

        let channel = Channel::from_shared(endpoint)
            .map_err(|e| errors::internal_communication_error(format!("Invalid endpoint: {}", e)))?
            .connect()
            .await
            .map_err(|e| {
                errors::internal_communication_error(format!("Connection failed: {}", e))
            })?;

        Ok(TransportLayerClient::new(channel))
    }

    /// Send a Protocol message to a peer via gRPC
    pub async fn send(peer: &PeerNode, msg: &Protocol) -> Result<(), CommError> {
        let mut transport = Self::create_client(peer).await?;

        // Create TLRequest with the protocol message
        let request = TlRequest {
            protocol: Some(msg.clone()),
        };

        // Send request with timeout and process response
        // TODO: Add metrics integration (incrementCounter("send"), timer("send-time"))
        let response = transport
            .send(Request::new(request))
            .await
            .map(|res| res.into_inner());

        Self::process_response(peer, response)
    }

    /// Stream a Blob to a peer via gRPC using chunking
    pub async fn stream(
        peer: &PeerNode,
        network_id: &str,
        blob: &Blob,
        packet_chunk_size: usize,
    ) -> Result<(), CommError> {
        let mut transport = Self::create_client(peer).await?;

        // Generate chunks using our Chunker
        let chunks = Chunker::chunk_it(network_id, blob, packet_chunk_size);

        // Create a stream of chunks
        let (tx, rx) = tokio::sync::mpsc::channel(chunks.len());

        // Send all chunks through the channel
        for chunk in chunks {
            if tx.send(chunk).await.is_err() {
                return Err(errors::internal_communication_error(
                    "Failed to send chunk to stream".to_string(),
                ));
            }
        }
        drop(tx); // Close the sender to signal end of stream

        // Convert receiver to stream and send
        let chunk_stream = ReceiverStream::new(rx);
        let response = transport
            .stream(Request::new(chunk_stream))
            .await
            .map(|res| res.into_inner());

        Self::process_response(peer, response)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rust::{
        peer_node::{Endpoint, NodeIdentifier},
        rp::protocol_helper,
    };
    use models::routing::{Ack, Header, InternalServerError, Packet};
    use prost::bytes::Bytes;

    fn create_test_peer() -> PeerNode {
        PeerNode {
            id: NodeIdentifier {
                key: Bytes::from("test_peer".as_bytes()),
            },
            endpoint: Endpoint::new("127.0.0.1".to_string(), 8080, 8080),
        }
    }

    #[test]
    fn test_error_pattern_matching() {
        // Test unavailable error
        let unavailable_error = Status::unavailable("Service unavailable");
        assert!(GrpcTransport::is_peer_unavailable(&unavailable_error));
        assert!(!GrpcTransport::is_peer_timeout(&unavailable_error));

        // Test timeout error
        let timeout_error = Status::deadline_exceeded("Request timeout");
        assert!(GrpcTransport::is_peer_timeout(&timeout_error));
        assert!(!GrpcTransport::is_peer_unavailable(&timeout_error));

        // Test permission denied (wrong network)
        let permission_error = Status::permission_denied("Wrong network");
        if let Some(msg) = GrpcTransport::is_peer_wrong_network(&permission_error) {
            assert_eq!(msg, "Wrong network");
        } else {
            panic!("Should detect wrong network error");
        }

        // Test resource exhausted (message too large)
        let resource_error = Status::resource_exhausted("Message too large");
        assert!(GrpcTransport::is_peer_message_too_large(&resource_error));
    }

    #[test]
    fn test_process_response_ack() {
        let peer = create_test_peer();
        let ack_response = TlResponse {
            payload: Some(Payload::Ack(Ack {
                header: Some(Header {
                    sender: Some(protocol_helper::node(&peer)),
                    network_id: "test".to_string(),
                }),
            })),
        };

        let result = GrpcTransport::process_response(&peer, Ok(ack_response));
        assert!(result.is_ok());
    }

    #[test]
    fn test_process_response_internal_error() {
        let peer = create_test_peer();
        let error_response = TlResponse {
            payload: Some(Payload::InternalServerError(InternalServerError {
                error: Bytes::from("Test error message"),
            })),
        };

        let result = GrpcTransport::process_response(&peer, Ok(error_response));
        assert!(result.is_err());

        if let Err(CommError::InternalCommunicationError(msg)) = result {
            assert!(msg.contains("Got response: Test error message"));
        } else {
            panic!("Expected InternalCommunicationError");
        }
    }

    #[test]
    fn test_process_error_mapping() {
        let peer = create_test_peer();

        // Test timeout mapping
        let timeout_result: Result<(), Status> = Err(Status::deadline_exceeded("timeout"));
        let mapped = GrpcTransport::process_error(&peer, timeout_result);
        assert!(matches!(mapped, Err(CommError::TimeOut)));

        // Test unavailable mapping
        let unavailable_result: Result<(), Status> = Err(Status::unavailable("unavailable"));
        let mapped = GrpcTransport::process_error(&peer, unavailable_result);
        assert!(matches!(mapped, Err(CommError::PeerUnavailable(_))));

        // Test permission denied mapping
        let permission_result: Result<(), Status> = Err(Status::permission_denied("wrong network"));
        let mapped = GrpcTransport::process_error(&peer, permission_result);
        assert!(matches!(mapped, Err(CommError::WrongNetwork(_, _))));

        // Test resource exhausted mapping
        let resource_result: Result<(), Status> = Err(Status::resource_exhausted("too large"));
        let mapped = GrpcTransport::process_error(&peer, resource_result);
        assert!(matches!(mapped, Err(CommError::MessageToLarge(_))));
    }

    #[test]
    fn test_create_blob_for_streaming() {
        let peer = create_test_peer();
        let packet = Packet {
            type_id: "TestPacket".to_string(),
            content: Bytes::from(vec![1u8; 1000]),
        };
        let blob = Blob {
            sender: peer,
            packet,
        };

        // Test that we can create chunks (functionality tested in chunker module)
        let chunks = Chunker::chunk_it("test_network", &blob, 4096);
        assert!(!chunks.is_empty());
        assert!(chunks.len() >= 1); // At least header chunk
    }
}

// See rholang/src/main/scala/coop/rchain/rholang/interpreter/GrpcClient.scala

use std::fmt;
use tonic::transport::{Channel, Endpoint};
use tonic::Request;

use crate::casper::v1::external_communication_service_client::ExternalCommunicationServiceClient;
use crate::casper::UpdateNotification;

#[derive(Debug)]
pub enum GrpcClientError {
    ConnectionError(String),
    RequestError(String),
}

impl fmt::Display for GrpcClientError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GrpcClientError::ConnectionError(msg) => write!(f, "Connection error: {}", msg),
            GrpcClientError::RequestError(msg) => write!(f, "Request error: {}", msg),
        }
    }
}

impl std::error::Error for GrpcClientError {}

pub struct GrpcClient {}

impl GrpcClient {
    /// Initialize a client and send a notification
    ///
    /// Note: client_host should include the scheme (http:// or https://)
    pub async fn init_client_and_tell(
        client_host: &str,
        client_port: u64,
        folder_id: &str,
    ) -> Result<(), GrpcClientError> {
        let channel = Self::create_channel(client_host, client_port).await?;
        let client = ExternalCommunicationServiceClient::new(channel);

        Self::grpc_tell(client, client_host, client_port as i32, folder_id).await
    }

    /// Create a channel for gRPC communication
    ///
    /// Note: host should include the scheme (http:// or https://)
    async fn create_channel(host: &str, port: u64) -> Result<Channel, GrpcClientError> {
        let endpoint = format!("{}:{}", host, port);
        Endpoint::from_shared(endpoint)
            .map_err(|e| GrpcClientError::ConnectionError(e.to_string()))?
            .connect()
            .await
            .map_err(|e| GrpcClientError::ConnectionError(e.to_string()))
    }

    /// Send a notification to the gRPC server
    async fn grpc_tell(
        mut client: ExternalCommunicationServiceClient<Channel>,
        client_host: &str,
        client_port: i32,
        folder_id: &str,
    ) -> Result<(), GrpcClientError> {
        // For the notification object, we should strip the scheme if present
        let host = client_host.replace("http://", "").replace("https://", "");

        let notification = UpdateNotification {
            client_host: host.to_string(),
            client_port,
            payload: folder_id.to_string(),
        };

        let request = Request::new(notification);

        println!("Waiting for response from gRPC server...");

        let response = client
            .send_notification(request)
            .await
            .map_err(|e| GrpcClientError::RequestError(e.to_string()))?;

        println!("Response from gRPC server: {:?}", response);

        Ok(())
    }
}

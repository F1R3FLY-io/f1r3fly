// See casper/src/test/scala/coop/rchain/casper/engine/ApproveBlockProtocolTest.scala

use std::collections::{HashMap};
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime};
use tokio::time::{sleep, timeout};

use casper::rust::engine::approve_block_protocol::{
    ApproveBlockProtocol, ApproveBlockProtocolFactory, Metrics, EventLog,
};
use casper::rust::genesis::contracts::{
    proof_of_stake::ProofOfStake, validator::Validator,
};
use casper::rust::genesis::genesis::Genesis;
use casper::rust::errors::CasperError;
use comm::rust::{
    peer_node::{Endpoint, NodeIdentifier, PeerNode},
    rp::{connect::{Connections, ConnectionsCell}, rp_conf::{ClearConnectionsConf, RPConf}},
    transport::transport_layer::{Blob, TransportLayer},
};
use crypto::rust::{
    hash::blake2b256::Blake2b256,
    private_key::PrivateKey,
    public_key::PublicKey,
    signatures::{secp256k1::Secp256k1, signatures_alg::SignaturesAlg},
};
use models::casper::Signature as ProtoSignature;
use models::rust::casper::protocol::casper_message::{
    ApprovedBlockCandidate, BlockApproval, BlockMessage,
};
use async_trait::async_trait;
use prost::{bytes, Message};

// Test implementations
#[derive(Clone)]
struct MetricsTestImpl {
    counters: Arc<Mutex<HashMap<String, i32>>>,
}

impl MetricsTestImpl {
    fn new() -> Self {
        Self {
            counters: Arc::new(Mutex::new(HashMap::new())),
        }
    }
    
    fn get_counter(&self, name: &str) -> i32 {
        let counters = self.counters.lock().unwrap();
        counters
            .get(&format!("approve-block.{}", name))
            .copied()
            .unwrap_or(0)
    }
}

impl Metrics for MetricsTestImpl {
    fn increment_counter(&self, name: &str) -> Result<(), CasperError> {
        let mut counters = self.counters.lock().unwrap();
        let full_name = format!("approve-block.{}", name);
        let current = counters.get(&full_name).copied().unwrap_or(0);
        counters.insert(full_name, current + 1);
        Ok(())
    }
}

#[derive(Clone)]
struct EventLogTestImpl {
    events: Arc<Mutex<Vec<String>>>,
}

impl EventLogTestImpl {
    fn new() -> Self {
        Self {
            events: Arc::new(Mutex::new(Vec::new())),
        }
    }
    
    fn get_events(&self) -> Vec<String> {
        self.events.lock().unwrap().clone()
    }
}

impl EventLog for EventLogTestImpl {
    fn publish_block_approval_received(&self, block_hash: &str, sender: &str) -> Result<(), CasperError> {
        let mut events = self.events.lock().unwrap();
        events.push(format!("BlockApprovalReceived({}, {})", block_hash, sender));
        Ok(())
    }

    fn publish_sent_unapproved_block(&self, candidate_hash: &str) -> Result<(), CasperError> {
        let mut events = self.events.lock().unwrap();
        events.push(format!("SentUnapprovedBlock({})", candidate_hash));
        Ok(())
    }
    
    fn publish_sent_approved_block(&self, candidate_hash: &str) -> Result<(), CasperError> {
        let mut events = self.events.lock().unwrap();
        events.push(format!("SentApprovedBlock({})", candidate_hash));
        Ok(())
    }
}

impl EventLogTestImpl {
    fn events_contain(&self, event_prefix: &str, expected_count: usize) -> bool {
        let events = self.events.lock().unwrap();
        let count = events.iter().filter(|e| e.starts_with(event_prefix)).count();
        count == expected_count
    }
}

#[derive(Default)]
struct TransportLayerTestImpl {
    messages: Arc<Mutex<Vec<Blob>>>,
}

impl TransportLayerTestImpl {
    fn new() -> Self {
        Self {
            messages: Arc::new(Mutex::new(Vec::new())),
        }
    }
    
    fn get_messages(&self) -> Vec<Blob> {
        self.messages.lock().unwrap().clone()
    }
}

#[async_trait]
impl TransportLayer for TransportLayerTestImpl {
    async fn send(&self, _peer: &PeerNode, _msg: &models::routing::Protocol) -> Result<(), comm::rust::errors::CommError> {
        Ok(())
    }

    async fn broadcast(&self, _peers: &[PeerNode], _msg: &models::routing::Protocol) -> Result<(), comm::rust::errors::CommError> {
        Ok(())
    }

    async fn stream(&self, _peer: &PeerNode, blob: &Blob) -> Result<(), comm::rust::errors::CommError> {
        let mut messages = self.messages.lock().unwrap();
        messages.push(blob.clone());
        Ok(())
    }

    async fn stream_mult(&self, _peers: &[PeerNode], blob: &Blob) -> Result<(), comm::rust::errors::CommError> {
        let mut messages = self.messages.lock().unwrap();
        messages.push(blob.clone());
        Ok(())
    }
}

struct TestFixture {
    protocol: Box<dyn ApproveBlockProtocol>,
    metrics: Arc<MetricsTestImpl>,
    event_log: Arc<EventLogTestImpl>,
    transport: Arc<TransportLayerTestImpl>,
    genesis_block: BlockMessage,
    candidate: ApprovedBlockCandidate,
    last_approved_block: Arc<Mutex<Option<models::rust::casper::protocol::casper_message::ApprovedBlock>>>,
}

impl TestFixture {
    fn new(
        required_sigs: i32,
        duration: Duration,
        interval: Duration,
        key_pairs: Vec<(PrivateKey, PublicKey)>,
    ) -> Self {
        // Create a simple test case
        let genesis_block = create_test_genesis_block(&key_pairs);

        let metrics = Arc::new(MetricsTestImpl::new());
        let event_log = Arc::new(EventLogTestImpl::new());
        let transport = Arc::new(TransportLayerTestImpl::new());
        let last_approved_block = Arc::new(Mutex::new(None));
        
        let test_peer = PeerNode {
            id: NodeIdentifier { 
                key: bytes::Bytes::from("test_peer".as_bytes().to_vec()) 
            },
            endpoint: Endpoint {
                host: "localhost".to_string(),
                tcp_port: 40400,
                udp_port: 40400,
            },
        };

        let connections_cell = Arc::new(ConnectionsCell {
            peers: Arc::new(Mutex::new(Connections(vec![test_peer.clone()]))),
        });

        let conf = Arc::new(RPConf {
            local: test_peer.clone(),
            network_id: "test_network".to_string(),
            bootstrap: None,
            clear_connections: ClearConnectionsConf { num_of_connections_pinged: 0 },
            max_num_of_connections: 100,
            default_timeout: Duration::from_secs(30),
        });

        let protocol = ApproveBlockProtocolFactory::unsafe_new_with_infrastructure(
            genesis_block.clone(),
            required_sigs,
            duration,
            interval,
            metrics.clone(),
            event_log.clone(),
            transport.clone(),
            Some(connections_cell),
            Some(conf),
            last_approved_block.clone(),
        );

        let candidate = ApprovedBlockCandidate {
            block: genesis_block.clone(),
            required_sigs,
        };

        Self {
            protocol,
            metrics,
            event_log,
            transport,
            genesis_block,
            candidate,
            last_approved_block,
        }
    }
}

fn create_test_genesis_block(validator_key_pairs: &[(PrivateKey, PublicKey)]) -> BlockMessage {
    // Create a simplified test genesis block
    use models::rust::casper::protocol::casper_message::{Body, F1r3flyState, Header, Bond};

    let bonds: Vec<Bond> = validator_key_pairs
        .iter()
        .map(|(_, public_key)| Bond {
            validator: public_key.bytes.clone(),
            stake: 1000000,
        })
        .collect();

    let state = F1r3flyState {
        pre_state_hash: bytes::Bytes::new(),
        post_state_hash: bytes::Bytes::new(),
        block_number: 0,
        bonds,
    };

    let body = Body {
        state,
        deploys: vec![],
        rejected_deploys: vec![],
        system_deploys: vec![],
        extra_bytes: bytes::Bytes::new(),
    };

    let header = Header {
        parents_hash_list: vec![],
        timestamp: 1559156071321,
        version: 1,
        extra_bytes: bytes::Bytes::new(),
    };

    BlockMessage {
        block_hash: bytes::Bytes::from(vec![1, 2, 3, 4]), // Dummy hash for testing
        header,
        body,
        justifications: vec![],
        sender: bytes::Bytes::new(),
        seq_num: 0,
        sig: bytes::Bytes::new(),
        sig_algorithm: "secp256k1".to_string(),
        shard_id: "test".to_string(),
        extra_bytes: bytes::Bytes::new(),
    }
}

fn create_approval(
    candidate: &ApprovedBlockCandidate,
    private_key: &PrivateKey,
    public_key: &PublicKey,
) -> BlockApproval {
    let secp256k1 = Secp256k1;
    let sig_data = Blake2b256::hash(candidate.clone().to_proto().encode_to_vec());
    let signature_bytes = secp256k1.sign_with_private_key(&sig_data, private_key);

    let signature = ProtoSignature {
        public_key: public_key.bytes.clone(),
        algorithm: "secp256k1".to_string(),
        sig: bytes::Bytes::from(signature_bytes),
    };

    BlockApproval {
        candidate: candidate.clone(),
        sig: signature,
    }
}

fn create_invalid_approval(candidate: &ApprovedBlockCandidate) -> BlockApproval {
    let secp256k1 = Secp256k1;
    let (private_key, public_key) = secp256k1.new_key_pair();

    let mut data_to_sign = candidate.clone().to_proto().encode_to_vec();
    data_to_sign.extend_from_slice(b"wrong data");
    let sig_data = Blake2b256::hash(data_to_sign);

    let signature_bytes = secp256k1.sign_with_private_key(&sig_data, &private_key);

    let signature = ProtoSignature {
        public_key: public_key.bytes.clone(),
        algorithm: "secp256k1".to_string(),
        sig: signature_bytes.into(),
    };

    BlockApproval {
        candidate: candidate.clone(),
        sig: signature,
    }
}

#[tokio::test]
async fn should_add_valid_signatures_to_state() {
    let secp256k1 = Secp256k1;
    let key_pair = secp256k1.new_key_pair();
    let key_pairs = vec![key_pair.clone()];

    let fixture = TestFixture::new(10, Duration::from_millis(100), Duration::from_millis(1), key_pairs);
    let approval = create_approval(&fixture.candidate, &key_pair.0, &key_pair.1);

    // Start the protocol in the background using a different approach
    let protocol = Arc::new(fixture.protocol);
    let protocol_clone = protocol.clone();
    let protocol_handle = tokio::spawn(async move {
        let _ = protocol_clone.run().await;
    });

    // Initially no metrics counter
    assert_eq!(fixture.metrics.get_counter("genesis"), 0);

    // Add approval
    protocol.add_approval(approval).await.unwrap();

    // Give some time for processing
    sleep(Duration::from_millis(10)).await;

    // Verify metrics were incremented like in Scala test
    assert_eq!(fixture.metrics.get_counter("genesis"), 1);

    // Cancel the background task
    protocol_handle.abort();
}

#[tokio::test]
async fn should_not_change_signatures_on_duplicate_approval() {
    let secp256k1 = Secp256k1;
    let key_pair = secp256k1.new_key_pair();
    let key_pairs = vec![key_pair.clone()];

    let fixture = TestFixture::new(10, Duration::from_millis(100), Duration::from_millis(1), key_pairs);
    let approval1 = create_approval(&fixture.candidate, &key_pair.0, &key_pair.1);
    let approval2 = create_approval(&fixture.candidate, &key_pair.0, &key_pair.1);

    // Start the protocol in the background
    let protocol = Arc::new(fixture.protocol);
    let protocol_clone = protocol.clone();
    let protocol_handle = tokio::spawn(async move {
        let _ = protocol_clone.run().await;
    });

    // Initially no metrics counter
    assert_eq!(fixture.metrics.get_counter("genesis"), 0);

    // Add approval twice
    protocol.add_approval(approval1).await.unwrap();
    sleep(Duration::from_millis(10)).await;
    protocol.add_approval(approval2).await.unwrap();
    sleep(Duration::from_millis(10)).await;

    // Verify counter only incremented once (duplicate ignored)
    assert_eq!(fixture.metrics.get_counter("genesis"), 1);

    // Cancel the background task
    protocol_handle.abort();
}

#[tokio::test]
async fn should_not_add_invalid_signatures() {
    let secp256k1 = Secp256k1;
    let key_pair = secp256k1.new_key_pair();
    let key_pairs = vec![key_pair];

    let fixture = TestFixture::new(10, Duration::from_millis(100), Duration::from_millis(1), key_pairs);
    let invalid_approval = create_invalid_approval(&fixture.candidate);

    // Start the protocol in the background
    let protocol = Arc::new(fixture.protocol);
    let protocol_clone = protocol.clone();
    let protocol_handle = tokio::spawn(async move {
        let _ = protocol_clone.run().await;
    });

    // Add invalid approval
    protocol.add_approval(invalid_approval).await.unwrap();
    sleep(Duration::from_millis(10)).await;

    // Verify no metrics counter incremented for invalid signature
    assert_eq!(fixture.metrics.get_counter("genesis"), 0);

    // Cancel the background task
    protocol_handle.abort();
}

#[tokio::test]
async fn should_create_approved_block_when_enough_signatures_collected() {
    let secp256k1 = Secp256k1;
    let n = 10;
    let key_pairs: Vec<_> = (0..n).map(|_| secp256k1.new_key_pair()).collect();

    let fixture = TestFixture::new(
        n as i32,
        Duration::from_millis(30),
        Duration::from_millis(1),
        key_pairs.clone(),
    );

    // Start the protocol in the background
    let protocol = Arc::new(fixture.protocol);
    let protocol_clone = protocol.clone();
    let protocol_handle = tokio::spawn(async move {
        let _ = protocol_clone.run().await;
    });

    // Add all approvals
    for (private_key, public_key) in &key_pairs {
        let approval = create_approval(&fixture.candidate, private_key, public_key);
        protocol.add_approval(approval).await.unwrap();
        sleep(Duration::from_millis(1)).await;
    }

    // Wait for the duration to elapse
    sleep(Duration::from_millis(35)).await;

    // Verify all approvals were counted
    assert_eq!(fixture.metrics.get_counter("genesis"), n as i32);

    // Verify approved block was created
    let approved_block = {
        let last_approved = fixture.last_approved_block.lock().unwrap();
        last_approved.clone()
    };
    assert!(approved_block.is_some());

    // Cancel the background task
    protocol_handle.abort();
}

#[tokio::test]
async fn should_continue_collecting_if_not_enough_signatures() {
    let secp256k1 = Secp256k1;
    let n = 10;
    let key_pairs: Vec<_> = (0..n).map(|_| secp256k1.new_key_pair()).collect();

    let fixture = TestFixture::new(
        n as i32,
        Duration::from_millis(30),
        Duration::from_millis(1),
        key_pairs.clone(),
    );

    // Start the protocol in the background
    let protocol = Arc::new(fixture.protocol);
    let protocol_clone = protocol.clone();
    let protocol_handle = tokio::spawn(async move {
        let _ = protocol_clone.run().await;
    });

    // Add only half the required approvals
    for (private_key, public_key) in key_pairs.iter().take(n / 2) {
        let approval = create_approval(&fixture.candidate, private_key, public_key);
        protocol.add_approval(approval).await.unwrap();
        sleep(Duration::from_millis(1)).await;
    }

    // Should not have approved block yet
    sleep(Duration::from_millis(35)).await;
    assert_eq!(fixture.metrics.get_counter("genesis"), (n / 2) as i32);

    // Add the remaining approvals
    for (private_key, public_key) in key_pairs.iter().skip(n / 2) {
        let approval = create_approval(&fixture.candidate, private_key, public_key);
        protocol.add_approval(approval).await.unwrap();
        sleep(Duration::from_millis(1)).await;
    }

    // Now should create approved block
    sleep(Duration::from_millis(35)).await;
    assert_eq!(fixture.metrics.get_counter("genesis"), n as i32);

    // Cancel the background task
    protocol_handle.abort();
}

#[tokio::test]
async fn should_skip_duration_when_required_signatures_is_zero() {
    let secp256k1 = Secp256k1;
    let key_pair = secp256k1.new_key_pair();
    let key_pairs = vec![key_pair];

    let fixture = TestFixture::new(
        0, // Zero required signatures
        Duration::from_millis(30),
        Duration::from_millis(1),
        key_pairs,
    );

    // Start the protocol
    let start = std::time::Instant::now();

    // Use timeout to ensure the protocol completes quickly
    let result = timeout(Duration::from_millis(100), fixture.protocol.run()).await;

    let elapsed = start.elapsed();

    // Should complete immediately, not wait for duration
    assert!(elapsed < Duration::from_millis(10));
    assert!(result.is_ok());
    
    // Should have no metrics counter for zero required sigs
    assert_eq!(fixture.metrics.get_counter("genesis"), 0);
}

#[tokio::test]
async fn should_not_accept_approval_from_untrusted_validator() {
    let secp256k1 = Secp256k1;
    let trusted_key_pair = secp256k1.new_key_pair();
    let untrusted_key_pair = secp256k1.new_key_pair();

    // Only the trusted key pair is in the genesis block
    let key_pairs = vec![trusted_key_pair];

    let fixture = TestFixture::new(10, Duration::from_millis(100), Duration::from_millis(1), key_pairs);

    // Create approval from untrusted validator
    let approval = create_approval(&fixture.candidate, &untrusted_key_pair.0, &untrusted_key_pair.1);

    // Start the protocol in the background
    let protocol = Arc::new(fixture.protocol);
    let protocol_clone = protocol.clone();
    let protocol_handle = tokio::spawn(async move {
        let _ = protocol_clone.run().await;
    });

    // Add approval from untrusted validator
    protocol.add_approval(approval).await.unwrap();
    sleep(Duration::from_millis(10)).await;

    // Verify no metrics counter incremented for untrusted validator
    assert_eq!(fixture.metrics.get_counter("genesis"), 0);

    // Cancel the background task
    protocol_handle.abort();
}

#[tokio::test]
async fn should_send_unapproved_block_message_to_peers_at_every_interval() {
    let secp256k1 = Secp256k1;
    let key_pair = secp256k1.new_key_pair();
    let key_pairs = vec![key_pair.clone()];

    let fixture = TestFixture::new(
        10, 
        Duration::from_millis(100), 
        Duration::from_millis(5), // 5ms interval for testing
        key_pairs
    );

    // Start the protocol in the background
    let protocol = Arc::new(fixture.protocol);
    let protocol_clone = protocol.clone();
    let protocol_handle = tokio::spawn(async move {
        let _ = protocol_clone.run().await;
    });

    // Wait for several intervals to pass
    sleep(Duration::from_millis(20)).await;

    let events = fixture.event_log.get_events();
    let message_count = fixture.transport.get_messages().len();

    // Verify that UnapprovedBlock messages are being sent at intervals
    // Should have at least 1 message, likely several due to the 5ms interval
    assert!(message_count >= 1, "Expected at least 1 message to be sent, got {}", message_count);
    
    // Verify that SentUnapprovedBlock events were published
    let unapproved_events_count = events.iter()
        .filter(|e| e.starts_with("SentUnapprovedBlock"))
        .count();
    assert!(unapproved_events_count >= 1, "Expected at least 1 SentUnapprovedBlock event, got {}", unapproved_events_count);

    // Add one approval to test the behavior continues
    let approval = create_approval(&fixture.candidate, &key_pair.0, &key_pair.1);
    protocol.add_approval(approval).await.unwrap();
    
    // Wait for another interval cycle  
    sleep(Duration::from_millis(15)).await;

    // Should continue broadcasting - verify more messages were sent
    let new_message_count = fixture.transport.get_messages().len();
    assert!(new_message_count > message_count, "Expected more messages after adding approval");
    
    // Cancel the background task
    protocol_handle.abort();
}

#[tokio::test]
async fn should_send_approved_block_message_to_peers_once_approved_block_is_created() {
    let secp256k1 = Secp256k1;
    let key_pair = secp256k1.new_key_pair();
    let key_pairs = vec![key_pair.clone()];

    let fixture = TestFixture::new(
        1, // Only need 1 signature
        Duration::from_millis(2), 
        Duration::from_millis(1),
        key_pairs
    );

    // Start the protocol in the background
    let protocol = Arc::new(fixture.protocol);
    let protocol_clone = protocol.clone();
    let protocol_handle = tokio::spawn(async move {
        let _ = protocol_clone.run().await;
    });

    // Add the required approval
    let approval = create_approval(&fixture.candidate, &key_pair.0, &key_pair.1);
    protocol.add_approval(approval).await.unwrap();

    // Wait for the duration to complete the ceremony
    sleep(Duration::from_millis(10)).await;

    // Verify that approved block was created
    let approved_block = {
        let last_approved = fixture.last_approved_block.lock().unwrap();
        last_approved.clone()
    };
    
    assert!(approved_block.is_some(), "ApprovedBlock should have been created");
    
    // Verify ApprovedBlock event was published
    assert!(fixture.event_log.events_contain("SentApprovedBlock", 1));
    
    // Cancel the background task
    protocol_handle.abort();
}
// See casper/src/main/scala/coop/rchain/casper/engine/BlockRetriever.scala

use std::{
    collections::{HashMap, HashSet},
    sync::{Arc, Mutex},
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use comm::rust::{
    peer_node::PeerNode,
    rp::{connect::ConnectionsCell, rp_conf::RPConf},
    transport::transport_layer::TransportLayer,
};
use log::{debug, info};
use models::rust::{block_hash::BlockHash, casper::pretty_printer::PrettyPrinter};

use crate::rust::errors::CasperError;

#[derive(Debug, Clone, PartialEq)]
pub enum AdmitHashReason {
    HasBlockMessageReceived,
    HashBroadcastReceived,
    MissingDependencyRequested,
    BlockReceived,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AdmitHashStatus {
    NewSourcePeerAddedToRequest,
    NewRequestAdded,
    Ignore,
}

#[derive(Debug, Clone)]
pub struct AdmitHashResult {
    pub status: AdmitHashStatus,
    pub broadcast_request: bool,
    pub request_block: bool,
}

#[derive(Debug, Clone)]
pub struct RequestState {
    pub timestamp: u64,
    pub peers: HashSet<PeerNode>,
    pub received: bool,
    pub in_casper_buffer: bool,
    pub waiting_list: Vec<PeerNode>,
}

#[derive(Debug, Clone, PartialEq)]
enum AckReceiveResult {
    AddedAsReceived,
    MarkedAsReceived,
}

/**
 * BlockRetriever makes sure block is received once Casper request it.
 * Block is in scope of BlockRetriever until it is added to CasperBuffer.
 */
pub struct BlockRetriever<T: TransportLayer + Send + Sync> {
    requested_blocks: Arc<Mutex<HashMap<BlockHash, RequestState>>>,
    transport: Arc<T>,
    connections_cell: Arc<ConnectionsCell>,
    conf: Arc<RPConf>,
}

impl<T: TransportLayer + Send + Sync> BlockRetriever<T> {
    pub fn new(
        transport: Arc<T>,
        connections_cell: Arc<ConnectionsCell>,
        conf: Arc<RPConf>,
    ) -> Self {
        Self {
            requested_blocks: Arc::new(Mutex::new(HashMap::new())),
            transport,
            connections_cell,
            conf,
        }
    }

    /// Helper method to add a source peer to an existing request
    fn add_source_peer_to_request(
        init_state: &mut HashMap<BlockHash, RequestState>,
        hash: &BlockHash,
        peer: &PeerNode,
    ) {
        if let Some(request_state) = init_state.get_mut(hash) {
            request_state.waiting_list.push(peer.clone());
        }
    }

    /// Helper method to add a new request
    fn add_new_request(
        init_state: &mut HashMap<BlockHash, RequestState>,
        hash: BlockHash,
        now: u64,
        mark_as_received: bool,
        source_peer: Option<&PeerNode>,
    ) -> bool {
        if init_state.contains_key(&hash) {
            false // Request already exists
        } else {
            let waiting_list = if let Some(peer) = source_peer {
                vec![peer.clone()]
            } else {
                Vec::new()
            };

            init_state.insert(
                hash,
                RequestState {
                    timestamp: now,
                    peers: HashSet::new(),
                    received: mark_as_received,
                    in_casper_buffer: false,
                    waiting_list,
                },
            );
            true
        }
    }

    /// Get current timestamp in milliseconds
    fn current_millis() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64
    }

    pub async fn admit_hash(
        &self,
        hash: BlockHash,
        peer: Option<PeerNode>,
        admit_hash_reason: AdmitHashReason,
    ) -> Result<AdmitHashResult, CasperError> {
        let now = Self::current_millis();

        // Lock the requested_blocks mutex and modify state atomically
        let result = {
            let mut state = self.requested_blocks.lock().map_err(|_| {
                CasperError::RuntimeError("Failed to acquire requested_blocks lock".to_string())
            })?;

            let unknown_hash = !state.contains_key(&hash);

            if unknown_hash {
                // Add new request
                Self::add_new_request(&mut state, hash.clone(), now, false, peer.as_ref());
                AdmitHashResult {
                    status: AdmitHashStatus::NewRequestAdded,
                    broadcast_request: peer.is_none(),
                    request_block: peer.is_some(),
                }
            } else if let Some(ref peer_node) = peer {
                // Hash exists, check if peer is already in waiting list
                let request_state = state.get(&hash).unwrap();

                if request_state.waiting_list.contains(peer_node) {
                    // Peer already in waiting list, ignore
                    AdmitHashResult {
                        status: AdmitHashStatus::Ignore,
                        broadcast_request: false,
                        request_block: false,
                    }
                } else {
                    // Add peer to waiting list
                    let was_empty = request_state.waiting_list.is_empty();
                    Self::add_source_peer_to_request(&mut state, &hash, peer_node);

                    AdmitHashResult {
                        status: AdmitHashStatus::NewSourcePeerAddedToRequest,
                        broadcast_request: false,
                        // Request block if this is the first peer in waiting list
                        request_block: was_empty,
                    }
                }
            } else {
                // Hash exists but no peer provided, ignore
                AdmitHashResult {
                    status: AdmitHashStatus::Ignore,
                    broadcast_request: false,
                    request_block: false,
                }
            }
        };

        // Log the result
        match result.status {
            AdmitHashStatus::NewSourcePeerAddedToRequest => {
                if let Some(ref peer_node) = peer {
                    debug!(
                        "Adding {} to waiting list of {} request. Reason: {:?}",
                        peer_node.endpoint.host,
                        PrettyPrinter::build_string_bytes(&hash),
                        admit_hash_reason
                    );
                }
            }
            AdmitHashStatus::NewRequestAdded => {
                info!(
                    "Adding {} hash to RequestedBlocks because of {:?}",
                    PrettyPrinter::build_string_bytes(&hash),
                    admit_hash_reason
                );
            }
            AdmitHashStatus::Ignore => {
                // No logging for ignore case
            }
        }

        // Handle broadcasting and requesting
        if result.broadcast_request {
            self.transport
                .broadcast_has_block_request(&self.connections_cell, &self.conf, &hash)
                .await?;
            debug!(
                "Broadcasted HasBlockRequest for {}",
                PrettyPrinter::build_string_bytes(&hash)
            );
        }

        if result.request_block {
            if let Some(ref peer_node) = peer {
                self.transport
                    .request_for_block(&self.conf, peer_node, hash.clone())
                    .await?;
                debug!(
                    "Requested block {} from {}",
                    PrettyPrinter::build_string_bytes(&hash),
                    peer_node.endpoint.host
                );
            }
        }

        Ok(result)
    }

    pub async fn request_all(&self, age_threshold: Duration) -> Result<(), CasperError> {
        let current_time = Self::current_millis();

        // Get all hashes that need processing
        let hashes_to_process: Vec<BlockHash> = {
            let state = self.requested_blocks.lock().map_err(|_| {
                CasperError::RuntimeError("Failed to acquire requested_blocks lock".to_string())
            })?;

            debug!(
                "Running BlockRetriever maintenance ({} items unexpired).",
                state.keys().len()
            );

            state.keys().cloned().collect()
        };

        // Process each hash
        for hash in hashes_to_process {
            // Get the current state for this hash
            let (expired, _received, sent_to_casper, should_rerequest) = {
                let state = self.requested_blocks.lock().map_err(|_| {
                    CasperError::RuntimeError("Failed to acquire requested_blocks lock".to_string())
                })?;

                if let Some(requested) = state.get(&hash) {
                    let expired =
                        current_time - requested.timestamp > age_threshold.as_millis() as u64;
                    let received = requested.received;
                    let sent_to_casper = requested.in_casper_buffer;

                    if !received {
                        debug!(
                            "Casper loop: checking if should re-request {}. Received: {}.",
                            PrettyPrinter::build_string_bytes(&hash),
                            received
                        );
                    }

                    (expired, received, sent_to_casper, !received && expired)
                } else {
                    continue; // Hash was removed, skip
                }
            };

            // Try to re-request if needed
            if should_rerequest {
                self.try_rerequest(&hash).await?;
            }

            // Remove expired entries that have been sent to Casper
            if sent_to_casper && expired {
                let mut state = self.requested_blocks.lock().map_err(|_| {
                    CasperError::RuntimeError("Failed to acquire requested_blocks lock".to_string())
                })?;
                state.remove(&hash);
            }
        }

        Ok(())
    }

    /// Helper method to try re-requesting a block from the next peer in waiting list
    async fn try_rerequest(&self, hash: &BlockHash) -> Result<(), CasperError> {
        // Get next peer from waiting list and update state
        let next_peer_opt = {
            let mut state = self.requested_blocks.lock().map_err(|_| {
                CasperError::RuntimeError("Failed to acquire requested_blocks lock".to_string())
            })?;

            if let Some(request_state) = state.get_mut(hash) {
                if !request_state.waiting_list.is_empty() {
                    let next_peer = request_state.waiting_list.remove(0);
                    let now = Self::current_millis();

                    // Update timestamp and add to peers set
                    request_state.timestamp = now;
                    request_state.peers.insert(next_peer.clone());

                    Some((next_peer, request_state.waiting_list.clone()))
                } else {
                    None
                }
            } else {
                None
            }
        };

        match next_peer_opt {
            Some((next_peer, remaining_waiting)) => {
                debug!(
                    "Trying {} to query for {} block. Remain waiting: {}.",
                    next_peer.endpoint.host,
                    PrettyPrinter::build_string_bytes(hash),
                    remaining_waiting
                        .iter()
                        .map(|p| p.endpoint.host.clone())
                        .collect::<Vec<_>>()
                        .join(", ")
                );

                // Request block from the peer
                self.transport
                    .request_for_block(&self.conf, &next_peer, hash.clone())
                    .await?;
            }
            None => {
                // No more peers in waiting list, get peers that were already tried
                let tried_peers = {
                    let state = self.requested_blocks.lock().map_err(|_| {
                        CasperError::RuntimeError(
                            "Failed to acquire requested_blocks lock".to_string(),
                        )
                    })?;

                    state
                        .get(hash)
                        .map(|rs| {
                            rs.peers
                                .iter()
                                .map(|p| p.endpoint.host.clone())
                                .collect::<Vec<_>>()
                        })
                        .unwrap_or_default()
                };

                log::warn!(
                    "Could not retrieve requested block {} from {}. Asking peers again.",
                    PrettyPrinter::build_string_bytes(hash),
                    tried_peers.join(", ")
                );

                // Remove from RequestedBlocks and broadcast HasBlockRequest
                {
                    let mut state = self.requested_blocks.lock().map_err(|_| {
                        CasperError::RuntimeError(
                            "Failed to acquire requested_blocks lock".to_string(),
                        )
                    })?;
                    state.remove(hash);
                }

                // Broadcast request for the block
                self.transport
                    .broadcast_has_block_request(&self.connections_cell, &self.conf, hash)
                    .await?;
            }
        }

        Ok(())
    }

    pub async fn ack_receive(&self, hash: BlockHash) -> Result<(), CasperError> {
        let now = Self::current_millis();

        // Lock the requested_blocks mutex and modify state atomically
        let result = {
            let mut state = self.requested_blocks.lock().map_err(|_| {
                CasperError::RuntimeError("Failed to acquire requested_blocks lock".to_string())
            })?;

            match state.get(&hash) {
                // There might be blocks that are not maintained by RequestedBlocks, e.g. fork-choice tips
                None => {
                    Self::add_new_request(&mut state, hash.clone(), now, true, None);
                    AckReceiveResult::AddedAsReceived
                }
                Some(requested) => {
                    // Make Casper loop aware that the block has been received
                    let mut updated_request = requested.clone();
                    updated_request.received = true;
                    state.insert(hash.clone(), updated_request);
                    AckReceiveResult::MarkedAsReceived
                }
            }
        };

        // Log based on the result
        match result {
            AckReceiveResult::AddedAsReceived => {
                info!(
                    "Block {} is not in RequestedBlocks. Adding and marking received.",
                    PrettyPrinter::build_string_bytes(&hash)
                );
            }
            AckReceiveResult::MarkedAsReceived => {
                info!(
                    "Block {} marked as received.",
                    PrettyPrinter::build_string_bytes(&hash)
                );
            }
        }

        Ok(())
    }

    pub async fn ack_in_casper(&self, hash: BlockHash) -> Result<(), CasperError> {
        // Check if block is already received
        let is_received = self.is_received(hash.clone()).await?;

        // If not received, acknowledge receipt first
        if !is_received {
            self.ack_receive(hash.clone()).await?;
        }

        // Mark as in Casper buffer
        let mut state = self.requested_blocks.lock().map_err(|_| {
            CasperError::RuntimeError("Failed to acquire requested_blocks lock".to_string())
        })?;

        if let Some(request_state) = state.get_mut(&hash) {
            request_state.in_casper_buffer = true;
        }

        Ok(())
    }

    pub async fn is_received(&self, hash: BlockHash) -> Result<bool, CasperError> {
        let state = self.requested_blocks.lock().map_err(|_| {
            CasperError::RuntimeError("Failed to acquire requested_blocks lock".to_string())
        })?;

        match state.get(&hash) {
            Some(request_state) => Ok(request_state.received),
            None => Ok(false),
        }
    }
}

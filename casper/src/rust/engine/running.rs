// See casper/src/main/scala/coop/rchain/casper/engine/Running.scala

use crate::{
    rust::{
        casper::MultiParentCasper,
        engine::{
            block_retriever::{self, BlockRetriever},
            engine::{self, Engine},
        },
        errors::CasperError,
        validator_identity::ValidatorIdentity,
    },
};
use async_trait::async_trait;
use comm::rust::{
    peer_node::PeerNode,
    rp::{connect::ConnectionsCell, rp_conf::RPConf}, transport::transport_layer::TransportLayer,
};
use models::rust::{
    block_hash::BlockHash,
    casper::{
        pretty_printer::PrettyPrinter,
        protocol::casper_message::{
            self, ApprovedBlock, ApprovedBlockCandidate, BlockHashMessage, BlockMessage,
            BlockRequest, CasperMessage, HasBlock, HasBlockRequest,
        },
    },
};

use rspace_plus_plus::rspace::{
    hashing::blake2b256_hash::Blake2b256Hash, 
    state::rspace_exporter::RSpaceExporterInstance,
};
use std::{
    collections::{HashSet, VecDeque},
    sync::{Arc, Mutex},
    time::{Duration, SystemTime, UNIX_EPOCH},
};

#[async_trait]
impl<'r, T: MultiParentCasper + Send + Sync + Clone> Engine for Running<'r, T> {
    async fn init(&self) -> Result<(), CasperError> {
        let mut init_called = self.init_called.lock().map_err(|_| {
            CasperError::RuntimeError("Failed to acquire init lock".to_string())
        })?;
        
        if *init_called {
            return Err(CasperError::RuntimeError("Init function already called".to_string()));
        }
        
        *init_called = true;
        log::info!("Running engine initialized");
        Ok(())
    }

    async fn handle(&mut self, peer: PeerNode, msg: CasperMessage) -> Result<(), CasperError> {
        match msg {
            CasperMessage::BlockHashMessage(h) => {
                self.handle_block_hash_message(peer, h, |hash| self.ignore_casper_message(hash))
                    .await
            }
            CasperMessage::BlockMessage(b) => {
                if let Some(id) = self.casper.get_validator() {
                    if b.sender == id.public_key.bytes {
                        log::warn!("There is another node {} proposing using the same private key as you. Or did you restart your node?", peer);
                    } else {
                        log::warn!("There is another node {} proposing using a different private key. Or did you restart your node?", peer);
                    }
                }
                if self.ignore_casper_message(b.block_hash.clone())? {
                    log::debug!(
                        "Ignoring BlockMessage {} from {}",
                        PrettyPrinter::build_string_block_message(&b, true),
                        peer.endpoint.host
                    );
                } else {
                    log::debug!(
                        "Incoming BlockMessage {} from {}",
                        PrettyPrinter::build_string_block_message(&b, true),
                        peer.endpoint.host
                    );
                    self.block_processing_queue.push_back((self.casper.clone(), b));
                }
                Ok(())
            }
            CasperMessage::BlockRequest(br) => self.handle_block_request(peer, br).await,

            
            // TODO should node say it has block only after it is in DAG, or CasperBuffer is enough? Or even just BlockStore?
            // https://github.com/rchain/rchain/pull/2943#discussion_r449887701 -- OLD
            CasperMessage::HasBlockRequest(hbr) => self
                .handle_has_block_request(peer, hbr, |hash| self.casper.dag_contains(&hash))
                .await,
            CasperMessage::HasBlock(hb) => {
                self.handle_has_block_message(peer, hb, |hash| self.ignore_casper_message(hash))
                    .await
            }
            CasperMessage::ForkChoiceTipRequest(_) => {
                self.handle_fork_choice_tip_request(peer).await
            }
            CasperMessage::ApprovedBlockRequest(abr) => {
                let last_finalized_block_hash =
                    self.casper.block_dag().await?.last_finalized_block();

                // Create approved block from last finalized block
                let last_finalized_block = self
                    .casper
                    .block_store()
                    .get(&last_finalized_block_hash)?
                    .unwrap();

                // Each approved block should be justified by validators signatures
                // ATM we have signatures only for genesis approved block - we also have to have a procedure
                // for gathering signatures for each approved block post genesis.
                // Now new node have to trust bootstrap if it wants to trim state when connecting to the network.
                // TODO We need signatures of Validators supporting this block -- OLD
                let last_approved_block = ApprovedBlock {
                    candidate: ApprovedBlockCandidate {
                        block: last_finalized_block,
                        required_sigs: 0,
                    },
                    sigs: vec![],
                };

                let approved_block = if abr.trim_state {
                    // If Last Finalized State is requested return Last Finalized block as Approved block
                    last_approved_block
                } else {
                    // Respond with approved block that this node is started from.
                    // The very first one is genesis, but this node still might start from later block,
                    // so it will not necessary be genesis.
                    self.approved_block.clone()
                };

                self.handle_approved_block_request(peer, approved_block)
                    .await
            }
            CasperMessage::NoApprovedBlockAvailable(na) => {
                engine::log_no_approved_block_available(&na.node_identifier);
                Ok(())
            }
            CasperMessage::StoreItemsMessageRequest(req) => {
                let start = req
                    .start_path
                    .iter()
                    .map(RSpaceExporterInstance::path_pretty)
                    .collect::<Vec<_>>()
                    .join(" ");

                log::info!(
                    "Received request for store items, startPath: [{}], chunk: {}, skip: {}, from: {}",
                    start,
                    req.take,
                    req.skip,
                    peer
                );

                if !self.disable_state_exporter {
                    self.handle_state_items_message_request(
                        peer,
                        req.start_path,
                        req.skip as u32,
                        req.take as u32,
                    )
                    .await
                } else {
                    log::info!(
                        "Received StoreItemsMessage request but the node is configured to not respond to StoreItemsMessage, from {}.",
                        peer
                    );
                    Ok(())
                }
            }
            _ => Ok(()),
        }
    }

    fn clone_box(&self) -> Box<dyn Engine> {
        // Note: This is simplified - full implementation would need proper cloning
        panic!("Running engine cannot be cloned - not implemented")
    }
}

pub struct Running<'r, M: MultiParentCasper, T: TransportLayer + Send + Sync> {
    block_processing_queue: VecDeque<(T, BlockMessage)>,
    blocks_in_processing: Arc<Mutex<HashSet<BlockHash>>>,
    casper: M,
    approved_block: ApprovedBlock,
    validator_id: Option<ValidatorIdentity>,
    init_called: Arc<Mutex<bool>>,
    disable_state_exporter: bool,
    connections_cell: ConnectionsCell,
    transport: Arc<T>,
    conf: RPConf,
    block_retriever: Arc<BlockRetriever<T>>,
    _phantom: std::marker::PhantomData<&'r M>,
}

impl<'r, M: MultiParentCasper + Clone, T: TransportLayer + Send + Sync> Running<'r, M, T> {
    pub fn new(
        block_processing_queue: VecDeque<(T, BlockMessage)>,
        blocks_in_processing: Arc<Mutex<HashSet<BlockHash>>>,
        casper: M,
        approved_block: ApprovedBlock,
        validator_id: Option<ValidatorIdentity>,
        disable_state_exporter: bool,
        connections_cell: ConnectionsCell,
        transport: Arc<T>,
        conf: RPConf,
        block_retriever: Arc<BlockRetriever<T>>   
    ) -> Self {
        Running {
            block_processing_queue,
            blocks_in_processing,
            casper,
            approved_block,
            validator_id,
            init_called: Arc::new(Mutex::new(false)),
            disable_state_exporter,
            connections_cell,
            transport,
            conf,
            block_retriever,
            _phantom: std::marker::PhantomData,
        }
    }

    async fn with_casper<'a, A>(
        &'a mut self,
        f: Box<dyn FnOnce(&'a mut T) -> Result<A, CasperError> + 'a>,
        _default: Result<A, CasperError>,
    ) -> Result<A, CasperError> {
        f(&mut self.casper)
    }

    fn ignore_casper_message(&self, hash: BlockHash) -> Result<bool, CasperError> {
        let blocks_in_processing = self.blocks_in_processing.lock().unwrap().contains(&hash);
        let buffer_contains = self.casper.buffer_contains(&hash);
        let dag_contains = self.casper.dag_contains(&hash);
        Ok(blocks_in_processing || buffer_contains || dag_contains)
    }

    pub async fn update_fork_choice_tips_if_stuck(
        &mut self,
        delay_threshold: Duration,
    ) -> Result<(), CasperError> {
        let latest_messages = self.casper.block_dag().await?.latest_message_hashes();
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as i64;
        
        let mut has_recent_latest_message = false;
        // Convert Arc<DashMap> to iterate over its contents
        for entry in latest_messages.iter() {
            let block_hash = entry.value();
            if let Ok(Some(block)) = self.casper.block_store().get(block_hash) {
                let block_timestamp = block.header.timestamp;
                if (now - block_timestamp) < delay_threshold.as_millis() as i64 {
                    has_recent_latest_message = true;
                    break;
                }
            }
        }

        let stuck = !has_recent_latest_message;
        if stuck {
            log::info!(
                "Requesting tips update as newest latest message is more then {:?} old. Might be network is faulty.",
                delay_threshold
            );
            self.transport.send_fork_choice_tip_request(&self.connections_cell, &self.conf).await?;
        }
        Ok(())
    }

    pub async fn handle_block_hash_message(
        &self,
        peer: PeerNode,
        bhm: BlockHashMessage,
        ignore_message_f: impl Fn(BlockHash) -> Result<bool, CasperError>,
    ) -> Result<(), CasperError> {
        let h = bhm.block_hash;
        if ignore_message_f(h.clone())? {
            log::debug!("Ignoring {} hash broadcast", PrettyPrinter::build_string_bytes(&h));
        } else {
            log::debug!(
                "Incoming BlockHashMessage {} from {}",
                PrettyPrinter::build_string_bytes(&h),
                peer.endpoint.host
            );
            self.block_retriever.admit_hash(
                    h,
                    Some(peer),
                    block_retriever::AdmitHashReason::HashBroadcastReceived,
                )
                .await?;
        }
        Ok(())
    }

    pub async fn handle_has_block_message(
        &self,
        peer: PeerNode,
        hb: HasBlock,
        ignore_message_f: impl Fn(BlockHash) -> Result<bool, CasperError>,
    ) -> Result<(), CasperError> {
        let h = hb.hash;
        if ignore_message_f(h.clone())? {
            log::debug!(
                "Ignoring {} HasBlockMessage",
                PrettyPrinter::build_string_bytes(&h)
            );
        } else {
            log::debug!(
                "Incoming HasBlockMessage {} from {}",
                PrettyPrinter::build_string_bytes(&h),
                peer.endpoint.host
            );
            self.block_retriever.admit_hash(
                    h,
                    Some(peer),
                    block_retriever::AdmitHashReason::HasBlockMessageReceived,
                )
                .await?;
        }
        Ok(())
    }

    pub async fn handle_block_request(
        &self,
        peer: PeerNode,
        br: BlockRequest,
    ) -> Result<(), CasperError> {
        let maybe_block = self.casper.block_store().get(&br.hash)?;
        if let Some(block) = maybe_block {
            log::info!(
                "Received request for block {} from {}. Response sent.",
                PrettyPrinter::build_string_bytes(&br.hash),
                peer
            );
            self.transport.stream_message_to_peer(&self.conf, &peer, &block.to_proto()).await?;
        } else {
            log::info!(
                "Received request for block {} from {}. No response given since block not found.",
                PrettyPrinter::build_string_bytes(&br.hash),
                peer
            );
        }
        Ok(())
    }

    pub async fn handle_has_block_request(
        &self,
        peer: PeerNode,
        hbr: HasBlockRequest,
        block_lookup: impl Fn(BlockHash) -> bool,
    ) -> Result<(), CasperError> {
        if block_lookup(hbr.hash.clone()) {
            let has_block = HasBlock { hash: hbr.hash };
            self.transport.send_message_to_peer(&self.conf, &peer, &has_block.to_proto()).await?;
        }
        Ok(())
    }

    /**
    * Peer asks for fork-choice tip
    */
    // TODO name for this message is misleading, as its a request for all tips, not just fork choice. -- OLD
    pub async fn handle_fork_choice_tip_request(&self, peer: PeerNode) -> Result<(), CasperError> {
        log::info!(
            "Received ForkChoiceTipRequest from {}",
            peer.endpoint.host
        );
        let latest_messages = self.casper.block_dag().await?.latest_message_hashes();
        let tips: Vec<BlockHash> = latest_messages.iter().map(|entry| entry.value().clone()).collect();
        log::info!(
            "Sending tips {} to {}",
            tips.iter().map(|tip| PrettyPrinter::build_string_bytes(tip)).collect::<Vec<_>>().join(", "),
            peer.endpoint.host
        );
        for tip in tips {
            let has_block = HasBlock { hash: tip };
            self.transport.send_message_to_peer(&self.conf, &peer, &has_block.to_proto()).await?;
        }
        Ok(())
    }

    pub async fn handle_approved_block_request(
        &self,
        peer: PeerNode,
        _approved_block: ApprovedBlock,
    ) -> Result<(), CasperError> {
        log::info!("Received ApprovedBlockRequest from {}", peer);
        self.transport.stream_message_to_peer(&self.conf, &peer, &approved_block.to_proto()).await?;
        log::info!("ApprovedBlock sent to {}", peer);
        Ok(())
    }

    async fn handle_state_items_message_request(
        &self,
        peer: PeerNode,
        start_path: Vec<(Blake2b256Hash, Option<u8>)>,
        _skip: u32,
        _take: u32,
    ) -> Result<(), CasperError> {
        let (history, data) = self
            .casper
            .rspace_state_manager()
            .exporter
            .get_history_and_data(start_path.clone(), skip as usize, take as usize, |bytes| {
                bytes.to_vec()
            })?;
        let resp = casper_message::StoreItemsMessage {
            start_path: start_path,
            last_path: history.last_path,
            history_items: history.items,
            data_items: data.items,
        };
        log::info!("Read {}", resp.pretty());
        self.transport.stream_message_to_peer(&self.conf, &peer, &resp.to_proto()).await?;

        log::info!("Store items sent to {}", peer);
        Ok(())
    }
}

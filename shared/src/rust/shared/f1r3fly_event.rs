// See shared/src/main/scala/coop/rchain/shared/RChainEvent.scala

#[derive(Debug, Clone)]
pub enum F1r3flyEvent {
    BlockCreated(BlockCreated),
    BlockAdded(BlockAdded),
    BlockFinalised(BlockFinalised),
}

#[derive(Debug, Clone)]
pub struct BlockCreated {
    pub block_hash: String,
    pub parent_hashes: Vec<String>,
    pub justification_hashes: Vec<(String, String)>,
    pub deploy_ids: Vec<String>,
    pub creator: String,
    pub seq_number: i32,
}

#[derive(Debug, Clone)]
pub struct BlockAdded {
    pub block_hash: String,
    pub parent_hashes: Vec<String>,
    pub justification_hashes: Vec<(String, String)>,
    pub deploy_ids: Vec<String>,
    pub creator: String,
    pub seq_number: i32,
}

#[derive(Debug, Clone)]
pub struct BlockFinalised {
    pub block_hash: String,
}

impl F1r3flyEvent {
    pub fn block_created(
        block_hash: String,
        parent_hashes: Vec<String>,
        justification_hashes: Vec<(String, String)>,
        deploy_ids: Vec<String>,
        creator: String,
        seq_number: i32,
    ) -> Self {
        Self::BlockCreated(BlockCreated {
            block_hash,
            parent_hashes,
            justification_hashes,
            deploy_ids,
            creator,
            seq_number,
        })
    }

    pub fn block_added(
        block_hash: String,
        parent_hashes: Vec<String>,
        justification_hashes: Vec<(String, String)>,
        deploy_ids: Vec<String>,
        creator: String,
        seq_number: i32,
    ) -> Self {
        Self::BlockAdded(BlockAdded {
            block_hash,
            parent_hashes,
            justification_hashes,
            deploy_ids,
            creator,
            seq_number,
        })
    }

    pub fn block_finalised(block_hash: String) -> Self {
        Self::BlockFinalised(BlockFinalised { block_hash })
    }
}

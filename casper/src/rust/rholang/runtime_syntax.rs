// See casper/src/main/scala/coop/rchain/casper/rholang/RuntimeSyntax.scala

use std::sync::{Arc, Mutex};

use models::rust::block::state_hash::StateHash;
use rholang::rust::interpreter::rho_runtime::{bootstrap_registry, RhoRuntime, RhoRuntimeImpl};
use rspace_plus_plus::rspace::history::instances::radix_history::RadixHistory;

use crate::rust::errors::CasperError;

// TODO: Remove Arc<Mutex<>>
pub struct RuntimeOps {
    pub runtime: Arc<Mutex<RhoRuntimeImpl>>,
}

impl RuntimeOps {
    pub fn new(runtime: Arc<Mutex<RhoRuntimeImpl>>) -> Self {
        Self { runtime }
    }

    /**
     * Because of the history legacy, the emptyStateHash does not really represent an empty trie.
     * The `emptyStateHash` is used as genesis block pre state which the state only contains registry
     * fixed channels in the state.
     */
    pub async fn empty_state_hash(&self) -> Result<StateHash, CasperError> {
        let mut runtime_lock = self.runtime.lock().unwrap();
        runtime_lock.reset(RadixHistory::empty_root_node_hash());
        drop(runtime_lock);

        bootstrap_registry(self.runtime.clone()).await;
        let mut runtime_lock = self.runtime.lock().unwrap();
        let checkpoint = runtime_lock.create_checkpoint();
        Ok(checkpoint.root.bytes().into())
    }
}

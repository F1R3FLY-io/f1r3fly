// Test adapter equivalent to Scala EngineWithCasper.scala
// Provides an Engine that always has a casper instance

use async_trait::async_trait;
use std::sync::Arc;

use comm::rust::peer_node::PeerNode;
use models::rust::casper::protocol::casper_message::CasperMessage;

use crate::rust::casper::MultiParentCasper;
use crate::rust::engine::engine::Engine;
use crate::rust::errors::CasperError;

pub struct EngineWithCasper<M: MultiParentCasper + Send + Sync> {
    casper: Arc<M>,
}

impl<M: MultiParentCasper + Send + Sync> EngineWithCasper<M> {
    pub fn new(casper: Arc<M>) -> Self {
        Self { casper }
    }
}

impl<M: MultiParentCasper + Send + Sync> Clone for EngineWithCasper<M> {
    fn clone(&self) -> Self {
        Self {
            casper: Arc::clone(&self.casper),
        }
    }
}

#[async_trait(?Send)]
impl<M: MultiParentCasper + Send + Sync + 'static> Engine for EngineWithCasper<M> {
    async fn init(&self) -> Result<(), CasperError> {
        Ok(())
    }

    async fn handle(&mut self, _peer: PeerNode, _msg: CasperMessage) -> Result<(), CasperError> {
        Ok(())
    }

    fn with_casper(&self) -> Option<&dyn MultiParentCasper> {
        Some(&*self.casper)
    }

    fn clone_box(&self) -> Box<dyn Engine> {
        Box::new((*self).clone())
    }
}

// See rspace/src/main/scala/coop/rchain/rspace/ReportingRspace.scala

use super::checkpoint::{Checkpoint, SoftCheckpoint};
use super::errors::RSpaceError;
use super::history::history_repository::HistoryRepository;
use super::hot_store::HotStore;
use super::internal::{ConsumeCandidate, WaitingContinuation};
use super::r#match::Match;
use super::replay_rspace::ReplayRSpace;
use super::rspace::RSpace;

use super::trace::event::{COMM, Consume, Produce};
use crate::rspace::rspace_interface::ISpace;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::fmt::Debug;
use std::hash::Hash;
use std::sync::{Arc, Mutex};

/// ReportingRspace works exactly like how ReplayRspace works. It can replay the deploy and try to find if the
/// deploy can be replayed well. But instead of just replaying the deploy, the ReportingRspace also save the comm
/// event data into the `report` field.
///
/// Currently only the unmatched comm event data are left in the tuplespace which means that the comm event data
/// happened in the processing of the deploy does not save anywhere in the software. It is believed that if we save
/// every comm event data during processing the deploy, the execution of Rholang would be much slower. But this(not
/// saving all comm event data) also leads to another problem that a developer can not get history data of deploy which
/// some of the comm event data are important to them. This ReportingRspace is trying to address this issue and let
/// people get the comm event data from replay.

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReportingEvent<C, P, A, K>
where
    C: Clone + Debug,
    P: Clone + Debug,
    A: Clone + Debug,
    K: Clone + Debug,
{
    ReportingProduce(ReportingProduce<C, A>),
    ReportingConsume(ReportingConsume<C, P, K>),
    ReportingComm(ReportingComm<C, P, A, K>),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReportingProduce<C, A>
where
    C: Clone + Debug,
    A: Clone + Debug,
{
    pub channel: C,
    pub data: A,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReportingConsume<C, P, K>
where
    C: Clone + Debug,
    P: Clone + Debug,
    K: Clone + Debug,
{
    pub channels: Vec<C>,
    pub patterns: Vec<P>,
    pub continuation: K,
    pub peeks: Vec<i32>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReportingComm<C, P, A, K>
where
    C: Clone + Debug,
    P: Clone + Debug,
    A: Clone + Debug,
    K: Clone + Debug,
{
    pub consume: ReportingConsume<C, P, K>,
    pub produces: Vec<ReportingProduce<C, A>>,
}

pub struct ReportingRspace<C, P, A, K>
where
    C: Clone + Debug + Default + Serialize + Hash + Ord + Eq + 'static + Sync + Send,
    P: Clone + Debug + Default + Serialize + 'static + Sync + Send,
    A: Clone + Debug + Default + Serialize + 'static + Sync + Send,
    K: Clone + Debug + Default + Serialize + 'static + Sync + Send,
{
    replay_rspace: ReplayRSpace<C, P, A, K>,
    /// in order to distinguish the system deploy(precharge and refund) in the a normal user deploy
    /// It might be more easily to analyse the report with data structure
    /// Vec<Vec[ReportingEvent]>(Precharge, userDeploy, Refund)
    /// It would be seperated by the softcheckpoint creation.
    report: Arc<Mutex<Vec<Vec<ReportingEvent<C, P, A, K>>>>>,
    soft_report: Arc<Mutex<Vec<ReportingEvent<C, P, A, K>>>>,
}

impl<C, P, A, K> ReportingRspace<C, P, A, K>
where
    C: Clone
        + Debug
        + Default
        + Send
        + Sync
        + Serialize
        + Ord
        + Hash
        + Eq
        + for<'a> Deserialize<'a>
        + 'static,
    P: Clone + Debug + Default + Send + Sync + Serialize + for<'a> Deserialize<'a> + 'static,
    A: Clone + Debug + Default + Send + Sync + Serialize + for<'a> Deserialize<'a> + 'static,
    K: Clone + Debug + Default + Send + Sync + Serialize + for<'a> Deserialize<'a> + 'static,
{
    /// Creates [[ReportingRspace]] from [[HistoryRepository]] and [[HotStore]].
    pub fn apply(
        history_repository: Arc<Box<dyn HistoryRepository<C, P, A, K>>>,
        store: Arc<Box<dyn HotStore<C, P, A, K>>>,
        matcher: Arc<Box<dyn Match<P, A>>>,
    ) -> ReportingRspace<C, P, A, K> {
        let replay_rspace = ReplayRSpace::apply(history_repository, store, matcher);

        ReportingRspace {
            replay_rspace,
            report: Arc::new(Mutex::new(Vec::new())),
            soft_report: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Creates [[ReportingRspace]] from [[KeyValueStore]]'s
    pub fn create(
        store: super::rspace::RSpaceStore,
        matcher: Arc<Box<dyn Match<P, A>>>,
    ) -> Result<ReportingRspace<C, P, A, K>, RSpaceError> {
        let history = RSpace::create_history_repo(store).map_err(|e| {
            RSpaceError::InterpreterError(format!("Failed to create history repo: {:?}", e))
        })?;
        let (history_repository, replay_store) = history;
        let reporting_rspace =
            Self::apply(Arc::new(history_repository), Arc::new(replay_store), matcher);
        Ok(reporting_rspace)
    }

    fn collect_report(&self) -> Result<(), RSpaceError> {
        let mut soft_report_guard = self.soft_report.lock().unwrap();

        if !soft_report_guard.is_empty() {
            let soft_report_content = std::mem::take(&mut *soft_report_guard);
            self.report.lock().unwrap().push(soft_report_content);
        }

        Ok(())
    }

    pub fn get_report(&self) -> Result<Vec<Vec<ReportingEvent<C, P, A, K>>>, RSpaceError> {
        self.collect_report()?;

        let mut report_guard = self.report.lock().unwrap();
        Ok(std::mem::take(&mut *report_guard))
    }

    fn get_soft_report(&self) -> Result<Vec<ReportingEvent<C, P, A, K>>, RSpaceError> {
        Ok(self.soft_report.lock().unwrap().clone())
    }

    /// Logs COMM events for reporting.
    ///
    /// **Note:** This method is correctly implemented but not called in the current architecture.
    /// In Scala, ReportingRspace extends ReplayRSpace and overrides logComm, so when ReplayRSpace
    /// internally calls self.logComm(), it would call this reporting version. In Rust, we use
    /// composition, so ReplayRSpace.log_comm() is called instead of ours.
    ///
    /// This method exists to maintain API compatibility and could be used if the logging architecture
    /// is modified in the future to support callback-based or trait-based logging.
    fn log_comm(
        &mut self,
        data_candidates: &[ConsumeCandidate<C, A>],
        channels: &[C],
        wk: WaitingContinuation<P, K>,
        comm: COMM,
        label: &str,
    ) -> COMM {
        let comm_ref = self.replay_rspace.log_comm(
            &data_candidates.to_vec(),
            &channels.to_vec(),
            wk.clone(),
            comm,
            label,
        );

        let reporting_consume = ReportingConsume {
            channels: channels.to_vec(),
            patterns: wk.patterns,
            continuation: wk.continuation,
            peeks: wk.peeks.into_iter().collect(),
        };

        let reporting_produces = data_candidates
            .iter()
            .map(|dc| ReportingProduce {
                channel: dc.channel.clone(),
                data: dc.datum.a.clone(),
            })
            .collect();

        let reporting_comm = ReportingEvent::ReportingComm(ReportingComm {
            consume: reporting_consume,
            produces: reporting_produces,
        });

        if let Ok(mut soft_report_guard) = self.soft_report.lock() {
            soft_report_guard.push(reporting_comm);
        }

        comm_ref
    }

    fn log_consume(
        &mut self,
        consume_ref: Consume,
        channels: &[C],
        patterns: &[P],
        continuation: &K,
        persist: bool,
        peeks: &BTreeSet<i32>,
    ) -> Consume {
        let result = self.replay_rspace.log_consume(
            consume_ref.clone(),
            &channels.to_vec(),
            &patterns.to_vec(),
            continuation,
            persist,
            peeks,
        );

        let reporting_consume = ReportingEvent::ReportingConsume(ReportingConsume {
            channels: channels.to_vec(),
            patterns: patterns.to_vec(),
            continuation: continuation.clone(),
            peeks: peeks.iter().copied().collect(),
        });

        if let Ok(mut soft_report_guard) = self.soft_report.lock() {
            soft_report_guard.push(reporting_consume);
        }

        result
    }

    fn log_produce(
        &mut self,
        produce_ref: Produce,
        channel: &C,
        data: &A,
        persist: bool,
    ) -> Produce {
        let result = self
            .replay_rspace
            .log_produce(produce_ref.clone(), channel, data, persist);

        let reporting_produce = ReportingEvent::ReportingProduce(ReportingProduce {
            channel: channel.clone(),
            data: data.clone(),
        });

        if let Ok(mut soft_report_guard) = self.soft_report.lock() {
            soft_report_guard.push(reporting_produce);
        }

        result
    }

    pub fn create_checkpoint(&mut self) -> Result<Checkpoint, RSpaceError> {
        let checkpoint = self.replay_rspace.create_checkpoint()?;

        self.soft_report.lock().unwrap().clear();
        self.report.lock().unwrap().clear();

        Ok(checkpoint)
    }

    pub fn create_soft_checkpoint(&mut self) -> Result<SoftCheckpoint<C, P, A, K>, RSpaceError> {
        self.collect_report()?;
        Ok(self.replay_rspace.create_soft_checkpoint())
    }
}

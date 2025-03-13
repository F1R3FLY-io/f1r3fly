// See casper/src/main/scala/coop/rchain/casper/util/rholang/ReplayFailure.scala

#[derive(Debug)]
pub enum ReplayFailure {
    InternalError {
        msg: String,
    },

    ReplayStatusMismatch {
        initial_failed: bool,
        replay_failed: bool,
    },

    UnusedCOMMEvent {
        msg: String,
    },

    ReplayCostMismatch {
        initial_cost: i64,
        replay_cost: i64,
    },

    SystemDeployErrorMismatch {
        play_error: String,
        replay_error: String,
    },
}

impl ReplayFailure {
    pub fn internal_error(msg: String) -> Self {
        ReplayFailure::InternalError { msg }
    }

    pub fn replay_status_mismatch(initial_failed: bool, replay_failed: bool) -> Self {
        ReplayFailure::ReplayStatusMismatch {
            initial_failed,
            replay_failed,
        }
    }

    pub fn unused_comm_event(msg: String) -> Self {
        ReplayFailure::UnusedCOMMEvent { msg }
    }

    pub fn replay_cost_mismatch(initial_cost: i64, replay_cost: i64) -> Self {
        ReplayFailure::ReplayCostMismatch {
            initial_cost,
            replay_cost,
        }
    }

    pub fn system_deploy_error_mismatch(play_error: String, replay_error: String) -> Self {
        ReplayFailure::SystemDeployErrorMismatch {
            play_error,
            replay_error,
        }
    }
}

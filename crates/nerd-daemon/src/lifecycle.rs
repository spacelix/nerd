//! Per-project lifecycle state machine. Memory-resident; the daemon owns one
//! state per started/starting project and broadcasts transitions for the UI.

use std::fmt;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LifecycleState {
    Stopped,
    Resolving,
    StartingServices,
    StartingApp,
    WaitingReady,
    Running,
    Stopping,
    Failed,
}

impl LifecycleState {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Stopped => "stopped",
            Self::Resolving => "resolving",
            Self::StartingServices => "starting-services",
            Self::StartingApp => "starting-app",
            Self::WaitingReady => "waiting-ready",
            Self::Running => "running",
            Self::Stopping => "stopping",
            Self::Failed => "failed",
        }
    }
}

impl fmt::Display for LifecycleState {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LifecycleFailure {
    pub stage: LifecycleState,
    pub cause: String,
    pub exit_code: Option<i32>,
}

/// The sequence of a healthy start.
pub const START_SEQUENCE: &[LifecycleState] = &[
    LifecycleState::Resolving,
    LifecycleState::StartingServices,
    LifecycleState::StartingApp,
    LifecycleState::WaitingReady,
    LifecycleState::Running,
];

impl LifecycleState {
    /// Whether a transition from `self` to `next` is valid.
    pub fn can_transition(&self, next: LifecycleState) -> bool {
        match self {
            Self::Stopped => matches!(next, Self::Resolving | Self::StartingApp | Self::Failed),
            Self::Resolving => matches!(
                next,
                Self::StartingServices | Self::StartingApp | Self::Failed
            ),
            Self::StartingServices => matches!(next, Self::StartingApp | Self::Failed),
            Self::StartingApp => matches!(next, Self::WaitingReady | Self::Failed),
            Self::WaitingReady => matches!(next, Self::Running | Self::Failed),
            Self::Running => matches!(next, Self::Stopping | Self::Failed),
            Self::Stopping => matches!(next, Self::Stopped | Self::Failed),
            Self::Failed => matches!(next, Self::Stopped | Self::Resolving),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{LifecycleState, START_SEQUENCE};

    #[test]
    fn start_sequence_is_valid() {
        assert!(LifecycleState::Stopped.can_transition(LifecycleState::Resolving));
        let mut previous = LifecycleState::Stopped;
        for state in START_SEQUENCE {
            assert!(previous.can_transition(*state));
            previous = *state;
        }
        assert!(previous.can_transition(LifecycleState::Stopping));
        assert!(LifecycleState::Running.can_transition(LifecycleState::Stopping));
    }

    #[test]
    fn any_stage_can_fail() {
        for state in START_SEQUENCE {
            assert!(state.can_transition(LifecycleState::Failed));
        }
    }
}

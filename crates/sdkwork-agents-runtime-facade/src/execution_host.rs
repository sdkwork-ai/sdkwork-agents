//! Execution-host vocabulary for agent session placement
//! (`agents.execution-placement` host and binding planes).
//!
//! A session executes on one of three targets (see [`crate::execution_route`]):
//! in this process, in the cloud, or on a dedicated execution host. Hosts are
//! registered infrastructure — docker daemons, micro VM pool members,
//! bare-metal boxes, or kernel cloud-sandbox pools — and host selection is
//! placement-owned, never client-writable
//! (`agent-execution-placement-orchestration.contract.json`
//! `forbiddenClientFields`). These types are the shared vocabulary used by the
//! durable host registry and session placement binding; the numeric state
//! codes mirror the `placement_state` smallints in the PostgreSQL contract.

use sdkwork_utils_rust::string::is_blank;

/// Host kind code for docker-daemon backed execution hosts.
pub const EXECUTION_HOST_KIND_DOCKER: &str = "docker";
/// Host kind code for micro VM (firecracker-class) execution hosts.
pub const EXECUTION_HOST_KIND_MICRO_VM: &str = "micro_vm";
/// Host kind code for bare-metal execution hosts.
pub const EXECUTION_HOST_KIND_BARE_METAL: &str = "bare_metal";
/// Host kind code for kernel-managed cloud sandbox pools.
pub const EXECUTION_HOST_KIND_CLOUD_SANDBOX: &str = "cloud_sandbox";

/// The kind of technology that backs a registered execution host.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AgentExecutionHostKind {
    Docker,
    MicroVm,
    BareMetal,
    CloudSandbox,
}

impl AgentExecutionHostKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Docker => EXECUTION_HOST_KIND_DOCKER,
            Self::MicroVm => EXECUTION_HOST_KIND_MICRO_VM,
            Self::BareMetal => EXECUTION_HOST_KIND_BARE_METAL,
            Self::CloudSandbox => EXECUTION_HOST_KIND_CLOUD_SANDBOX,
        }
    }

    /// Parses a host-kind code; unknown codes fail closed so a placement can
    /// never persist a host kind the durable registry does not constrain.
    pub fn parse(code: Option<&str>) -> Option<Self> {
        match code.map(str::trim).filter(|value| !is_blank(Some(value)))? {
            EXECUTION_HOST_KIND_DOCKER => Some(Self::Docker),
            EXECUTION_HOST_KIND_MICRO_VM => Some(Self::MicroVm),
            EXECUTION_HOST_KIND_BARE_METAL => Some(Self::BareMetal),
            EXECUTION_HOST_KIND_CLOUD_SANDBOX => Some(Self::CloudSandbox),
            _ => None,
        }
    }
}

/// A host kind code was supplied but is not a canonical [`AgentExecutionHostKind`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InvalidExecutionHostKindError {
    pub code: String,
}

impl std::fmt::Display for InvalidExecutionHostKindError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "invalid execution host kind \"{}\": must be one of {EXECUTION_HOST_KIND_DOCKER}, {EXECUTION_HOST_KIND_MICRO_VM}, {EXECUTION_HOST_KIND_BARE_METAL}, {EXECUTION_HOST_KIND_CLOUD_SANDBOX}",
            self.code
        )
    }
}

impl std::error::Error for InvalidExecutionHostKindError {}

/// Parses a host-kind code, failing closed with the offending code named.
pub fn parse_agent_execution_host_kind(
    code: Option<&str>,
) -> Result<AgentExecutionHostKind, InvalidExecutionHostKindError> {
    AgentExecutionHostKind::parse(code).ok_or_else(|| InvalidExecutionHostKindError {
        code: code.unwrap_or_default().trim().to_string(),
    })
}

/// Lifecycle of one session execution placement (`placementLifecycleCandidate`
/// in `agent-execution-placement-orchestration.contract.json`). The numeric
/// codes are the durable `placement_state` smallints.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AgentExecutionPlacementState {
    /// The placement was requested but not yet reserved.
    Requested,
    /// The kernel/placement owner is allocating capacity.
    Allocating,
    /// Capacity is reserved and the host is ready to serve the session.
    Ready,
    /// The session is actively executing on the placement.
    Active,
    /// The placement is being released.
    Releasing,
    /// The placement was released; terminal.
    Released,
    /// The placement failed; requires release or reconciliation.
    Failed,
    /// The placement lease expired; requires release or reconciliation.
    Expired,
}

impl AgentExecutionPlacementState {
    pub fn as_code(self) -> i16 {
        match self {
            Self::Requested => 0,
            Self::Allocating => 1,
            Self::Ready => 2,
            Self::Active => 3,
            Self::Releasing => 4,
            Self::Released => 5,
            Self::Failed => 6,
            Self::Expired => 7,
        }
    }

    /// Parses a durable `placement_state` smallint; unknown codes fail closed.
    pub fn from_code(code: i16) -> Option<Self> {
        match code {
            0 => Some(Self::Requested),
            1 => Some(Self::Allocating),
            2 => Some(Self::Ready),
            3 => Some(Self::Active),
            4 => Some(Self::Releasing),
            5 => Some(Self::Released),
            6 => Some(Self::Failed),
            7 => Some(Self::Expired),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Allocating => "allocating",
            Self::Ready => "ready",
            Self::Active => "active",
            Self::Releasing => "releasing",
            Self::Released => "released",
            Self::Failed => "failed",
            Self::Expired => "expired",
        }
    }

    /// `RELEASED` is the only terminal state; failed and expired placements
    /// require an explicit release or reconciliation transition.
    pub fn is_terminal(self) -> bool {
        matches!(self, Self::Released)
    }

    /// States a scheduler treats as occupying placement capacity.
    pub fn occupies_capacity(self) -> bool {
        matches!(
            self,
            Self::Requested | Self::Allocating | Self::Ready | Self::Active
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn host_kind_codes_round_trip() {
        for (code, kind) in [
            (EXECUTION_HOST_KIND_DOCKER, AgentExecutionHostKind::Docker),
            (
                EXECUTION_HOST_KIND_MICRO_VM,
                AgentExecutionHostKind::MicroVm,
            ),
            (
                EXECUTION_HOST_KIND_BARE_METAL,
                AgentExecutionHostKind::BareMetal,
            ),
            (
                EXECUTION_HOST_KIND_CLOUD_SANDBOX,
                AgentExecutionHostKind::CloudSandbox,
            ),
        ] {
            assert_eq!(AgentExecutionHostKind::parse(Some(code)), Some(kind));
            assert_eq!(kind.as_str(), code);
        }
    }

    #[test]
    fn host_kind_unknown_codes_fail_closed() {
        assert_eq!(AgentExecutionHostKind::parse(None), None);
        assert_eq!(AgentExecutionHostKind::parse(Some("")), None);
        assert_eq!(AgentExecutionHostKind::parse(Some("Docker")), None);
        assert_eq!(AgentExecutionHostKind::parse(Some("kvm")), None);

        let error = parse_agent_execution_host_kind(Some("podman"))
            .expect_err("unknown host kind names the code");
        assert_eq!(error.code, "podman");
        assert!(error.to_string().contains("invalid execution host kind"));
    }

    #[test]
    fn placement_state_codes_round_trip() {
        let states = [
            AgentExecutionPlacementState::Requested,
            AgentExecutionPlacementState::Allocating,
            AgentExecutionPlacementState::Ready,
            AgentExecutionPlacementState::Active,
            AgentExecutionPlacementState::Releasing,
            AgentExecutionPlacementState::Released,
            AgentExecutionPlacementState::Failed,
            AgentExecutionPlacementState::Expired,
        ];
        for (index, state) in states.iter().enumerate() {
            assert_eq!(state.as_code(), index as i16);
            assert_eq!(
                AgentExecutionPlacementState::from_code(index as i16),
                Some(*state)
            );
        }
        assert_eq!(AgentExecutionPlacementState::from_code(8), None);
        assert_eq!(AgentExecutionPlacementState::from_code(-1), None);
    }

    #[test]
    fn released_is_the_only_terminal_state() {
        assert!(AgentExecutionPlacementState::Released.is_terminal());
        assert!(!AgentExecutionPlacementState::Failed.is_terminal());
        assert!(!AgentExecutionPlacementState::Expired.is_terminal());
        assert!(AgentExecutionPlacementState::Active.occupies_capacity());
        assert!(!AgentExecutionPlacementState::Released.occupies_capacity());
        assert!(!AgentExecutionPlacementState::Failed.occupies_capacity());
    }
}

//! Durable idempotency and execution ledger for one Agents turn.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentTurnMode {
    Interactive,
    Background,
    Automation,
    Resume,
    Retry,
}

impl AgentTurnMode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Interactive => "interactive",
            Self::Background => "background",
            Self::Automation => "automation",
            Self::Resume => "resume",
            Self::Retry => "retry",
        }
    }

    pub fn as_db_code(self) -> i16 {
        match self {
            Self::Interactive => 0,
            Self::Background => 1,
            Self::Automation => 2,
            Self::Resume => 3,
            Self::Retry => 4,
        }
    }

    pub fn from_db_code(value: i16) -> Option<Self> {
        match value {
            0 => Some(Self::Interactive),
            1 => Some(Self::Background),
            2 => Some(Self::Automation),
            3 => Some(Self::Resume),
            4 => Some(Self::Retry),
            _ => None,
        }
    }

    pub fn from_code(value: &str) -> Option<Self> {
        match value {
            "interactive" => Some(Self::Interactive),
            "background" => Some(Self::Background),
            "automation" => Some(Self::Automation),
            "resume" => Some(Self::Resume),
            "retry" => Some(Self::Retry),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentTurnStatus {
    Requested,
    Running,
    Completed,
    Failed,
    Cancelled,
}

impl AgentTurnStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Running => "running",
            Self::Completed => "completed",
            Self::Failed => "failed",
            Self::Cancelled => "cancelled",
        }
    }

    pub fn as_db_code(self) -> i16 {
        match self {
            Self::Requested => 0,
            Self::Running => 1,
            Self::Completed => 2,
            Self::Failed => 3,
            Self::Cancelled => 4,
        }
    }

    pub fn from_db_code(value: i16) -> Option<Self> {
        match value {
            0 => Some(Self::Requested),
            1 => Some(Self::Running),
            2 => Some(Self::Completed),
            3 => Some(Self::Failed),
            4 => Some(Self::Cancelled),
            _ => None,
        }
    }

    pub fn from_code(value: &str) -> Option<Self> {
        match value {
            "requested" => Some(Self::Requested),
            "running" => Some(Self::Running),
            "completed" => Some(Self::Completed),
            "failed" => Some(Self::Failed),
            "cancelled" => Some(Self::Cancelled),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentTurnRecord {
    pub id: u64,
    pub turn_id: String,
    pub tenant_id: u64,
    pub organization_id: u64,
    pub session_id: String,
    pub agent_id: String,
    pub owner_user_id: u64,
    pub runtime_binding_id: Option<String>,
    pub client_request_id: Option<String>,
    pub idempotency_key: String,
    pub payload_hash: String,
    pub request_item_id: String,
    pub response_item_id: Option<String>,
    pub turn_mode: AgentTurnMode,
    pub status: AgentTurnStatus,
    pub requested_model_id: Option<String>,
    pub provider_binding_id: Option<String>,
    pub model_id: Option<String>,
    pub provider_id: Option<String>,
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub cached_tokens: u64,
    pub finish_reason: Option<String>,
    pub error_code: Option<String>,
    pub error_detail: Option<String>,
    pub trace_id: Option<String>,
    pub attempt_count: u32,
    pub max_attempts: u32,
    pub next_retry_at: Option<String>,
    pub available_at: String,
    pub lease_owner: Option<String>,
    pub lease_token: Option<String>,
    pub lease_expires_at: Option<String>,
    pub fencing_token: u64,
    pub version: u64,
    pub created_at: String,
    pub updated_at: String,
    pub started_at: Option<String>,
    pub completed_at: Option<String>,
    pub cancel_requested_at: Option<String>,
    pub cancelled_at: Option<String>,
    pub retention_until: Option<String>,
}

impl AgentTurnRecord {
    pub fn mark_running(&mut self, occurred_at: impl Into<String>) {
        let occurred_at = occurred_at.into();
        self.status = AgentTurnStatus::Running;
        self.attempt_count = self.attempt_count.saturating_add(1);
        self.started_at = Some(occurred_at.clone());
        self.updated_at = occurred_at;
        self.version = self.version.saturating_add(1);
    }

    /// Installs the execution lease of a running interactive turn.
    ///
    /// Interactive turns execute in-process on the replica that accepted the
    /// request, so they must carry a live lease like task runs do: without it
    /// the turn reconciler on any replica treats a silent stretch longer than
    /// the stale threshold (long thinking, long tool call) as a crashed worker
    /// and fails a turn that is still executing. The token doubles as the
    /// completion fencing input — `complete_turn` requires the executing
    /// replica to present the token it acquired.
    pub fn begin_lease(
        &mut self,
        owner: impl Into<String>,
        token: impl Into<String>,
        expires_at: impl Into<String>,
    ) {
        self.lease_owner = Some(owner.into());
        self.lease_token = Some(token.into());
        self.lease_expires_at = Some(expires_at.into());
    }

    pub fn mark_failed(
        &mut self,
        error_code: impl Into<String>,
        error_detail: impl Into<String>,
        occurred_at: impl Into<String>,
    ) {
        let occurred_at = occurred_at.into();
        self.status = AgentTurnStatus::Failed;
        self.error_code = Some(error_code.into());
        self.error_detail = Some(error_detail.into());
        self.completed_at = Some(occurred_at.clone());
        self.lease_owner = None;
        self.lease_token = None;
        self.lease_expires_at = None;
        self.updated_at = occurred_at;
        self.version = self.version.saturating_add(1);
    }

    pub fn mark_completed(&mut self, occurred_at: impl Into<String>) {
        let occurred_at = occurred_at.into();
        self.status = AgentTurnStatus::Completed;
        self.completed_at = Some(occurred_at.clone());
        self.lease_owner = None;
        self.lease_token = None;
        self.lease_expires_at = None;
        self.updated_at = occurred_at;
        self.version = self.version.saturating_add(1);
    }

    pub fn mark_cancelled(&mut self, occurred_at: impl Into<String>) {
        let occurred_at = occurred_at.into();
        self.status = AgentTurnStatus::Cancelled;
        self.cancel_requested_at = Some(occurred_at.clone());
        self.cancelled_at = Some(occurred_at.clone());
        self.completed_at = Some(occurred_at.clone());
        self.lease_owner = None;
        self.lease_token = None;
        self.lease_expires_at = None;
        self.updated_at = occurred_at;
        self.version = self.version.saturating_add(1);
    }
}

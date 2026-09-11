use crate::agent_turn::AgentTurnRecord;
use crate::agent_turn_input_queue::{
    AgentTurnInputQueueEntry, TurnInputQueueClaimOutcome, TurnInputQueueClaimRequest,
    TurnInputQueueFailureRequest, TurnInputQueueListQuery, TurnInputQueueReorderEntry,
};
use crate::domain::{
    AgentBusinessRecord, AgentCompositionSlotKind, AgentCompositionSlotRecord,
    AgentInteractionRecord, AgentItemDriveRefRecord, AgentItemFeedbackRecord,
    AgentProviderBindingRecord, AgentResourceType, AgentResourceUserStateRecord,
    AgentSessionCheckpointRecord, AgentSessionItemKind, AgentSessionItemRecord,
    AgentSessionItemStatus, AgentSessionRecord, AgentSessionRuntimeBindingRecord, AgentTaskRecord,
    AgentToolAssetRecord, AgentToolConfigurationRecord, AgentVisibility,
};
use crate::list_cursors::{
    audit_list_scope_fingerprint, interaction_list_scope_fingerprint,
    session_list_scope_fingerprint, turn_list_scope_fingerprint, AuditEventListCursor,
    CreatedAtListCursor, SessionListCursor,
};
use crate::project::{AgentProjectCompositionSlotRecord, AgentProjectRecord, AgentProjectStatus};
use crate::session_activity::{
    session_activity_scope_fingerprint, SessionActivityCursor, SessionActivitySummaryRecord,
};
use crate::session_item_cursor::{session_item_scope_fingerprint, SessionItemCursor};
use crate::task_execution_cursor::{task_scope_fingerprint, TaskCursor};
use crate::validation::optional_non_blank;
use crate::workspace::{AgentWorkspaceRecord, AgentWorkspaceStatus};
use sdkwork_agent_kernel::{KernelError, KernelEvent, KernelResult};
use std::collections::HashSet;

// ---------------------------------------------------------------------------
// Pagination types for list operations
// ---------------------------------------------------------------------------

/// Maximum allowed page size to prevent memory exhaustion and ensure
/// consistent API performance per DATABASE_SPEC §16 pagination requirements.
pub const MAX_PAGE_SIZE: usize = 200;

/// Default page size for list operations when not specified.
pub const DEFAULT_PAGE_SIZE: usize = 20;

/// Maximum number of prior session items loaded into one turn context.
pub const TURN_CONTEXT_ITEM_LIMIT: usize = 50;
/// Maximum user-input payload accepted on turn and task prompt surfaces.
pub const MAX_TURN_INPUT_CONTENT_BYTES: usize = 256 * 1024;

pub(crate) fn validate_completed_turn_items(
    turn: &AgentTurnRecord,
    expected_turn_version: u64,
    completed_items: &[AgentSessionItemRecord],
) -> KernelResult<()> {
    if turn.status != crate::agent_turn::AgentTurnStatus::Completed
        || turn.version != expected_turn_version.saturating_add(1)
        || completed_items.is_empty()
    {
        return Err(KernelError::validation(
            "completed Turn and item batch are inconsistent",
        ));
    }
    let response_item = completed_items.last().expect("non-empty item batch");
    if turn.response_item_id.as_deref() != Some(response_item.item_id.as_str())
        || response_item.kind != AgentSessionItemKind::AssistantOutput
        || response_item.status != AgentSessionItemStatus::Completed
        || response_item.parent_item_id.as_deref() != Some(turn.request_item_id.as_str())
    {
        return Err(KernelError::validation(
            "completed Turn response item is invalid",
        ));
    }

    let item_ids = completed_items
        .iter()
        .map(|item| item.item_id.as_str())
        .collect::<HashSet<_>>();
    if item_ids.len() != completed_items.len() {
        return Err(KernelError::conflict(
            "completed Turn contains duplicate item identities",
        ));
    }
    for (index, item) in completed_items.iter().enumerate() {
        if item.sequence != 0
            || item.tenant_id != turn.tenant_id
            || item.organization_id != turn.organization_id
            || item.session_id != turn.session_id
            || item.turn_id.as_deref() != Some(turn.turn_id.as_str())
            || item.created_by != turn.owner_user_id
            || item.status == AgentSessionItemStatus::Pending
            || item.completed_at.is_none()
        {
            return Err(KernelError::validation(
                "completed Turn item scope or lifecycle is invalid",
            ));
        }
        if index + 1 != completed_items.len()
            && matches!(
                item.kind,
                AgentSessionItemKind::UserInput
                    | AgentSessionItemKind::SystemInstruction
                    | AgentSessionItemKind::AssistantOutput
            )
        {
            return Err(KernelError::validation(
                "completed Turn provider item kind is invalid",
            ));
        }
        if item.parent_item_id.as_deref().is_none_or(|parent_item_id| {
            parent_item_id == item.item_id
                || (parent_item_id != turn.request_item_id && !item_ids.contains(parent_item_id))
        }) {
            return Err(KernelError::validation(
                "completed Turn item parent identity is invalid",
            ));
        }
        let content_required = matches!(
            item.kind,
            AgentSessionItemKind::AssistantOutput
                | AgentSessionItemKind::Reasoning
                | AgentSessionItemKind::StatusNotice
                | AgentSessionItemKind::ErrorNotice
        );
        if content_required && item.content.as_deref().is_none_or(str::is_empty) {
            return Err(KernelError::validation(
                "completed Turn text item has no content",
            ));
        }
        match item.kind {
            AgentSessionItemKind::ToolCall
                if item.tool_name.as_deref().is_none_or(str::is_empty)
                    || item.tool_call_id.as_deref().is_none_or(str::is_empty)
                    || item.tool_arguments_json.is_none()
                    || item.tool_result_json.is_some()
                    || item.content.is_some() =>
            {
                return Err(KernelError::validation(
                    "completed Turn tool-call payload is invalid",
                ));
            }
            AgentSessionItemKind::ToolResult
                if item.tool_call_id.as_deref().is_none_or(str::is_empty)
                    || item.tool_arguments_json.is_some()
                    || item.tool_result_json.is_none()
                    || item.content.is_some() =>
            {
                return Err(KernelError::validation(
                    "completed Turn tool-result payload is invalid",
                ));
            }
            AgentSessionItemKind::ToolCall | AgentSessionItemKind::ToolResult => {}
            _ if item.tool_name.is_some()
                || item.tool_call_id.is_some()
                || item.tool_arguments_json.is_some()
                || item.tool_result_json.is_some() =>
            {
                return Err(KernelError::validation(
                    "completed Turn non-tool item contains tool payload",
                ));
            }
            _ => {}
        }
        if item.status == AgentSessionItemStatus::Redacted {
            if item.redacted_at.is_none() || item.redacted_by.is_none() {
                return Err(KernelError::validation(
                    "redacted completed Turn item requires redaction attribution",
                ));
            }
        } else if item.redacted_at.is_some() || item.redacted_by.is_some() {
            return Err(KernelError::validation(
                "non-redacted completed Turn item contains redaction attribution",
            ));
        }
    }
    Ok(())
}

/// Build offset-mode pagination metadata from a repository page and total count.
pub fn offset_paginated_result<T>(
    items: Vec<T>,
    pagination: &PaginationParams,
    total_count: u64,
) -> PaginatedResult<T> {
    let has_more = (pagination.offset + items.len()) < total_count as usize;
    PaginatedResult {
        items,
        next_page_token: None,
        total_count: Some(total_count),
        has_more,
    }
}

/// Pagination parameters for list queries.
/// Implements DATABASE_SPEC §16 requirements for mandatory pagination.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PaginationParams {
    /// Maximum number of items to return per page (1-200)
    pub page_size: usize,
    /// Zero-based row offset for page navigation
    pub offset: usize,
    /// Cursor for fetching the next page (offset-free pagination)
    pub page_token: Option<String>,
}

impl Default for PaginationParams {
    fn default() -> Self {
        Self {
            page_size: DEFAULT_PAGE_SIZE,
            offset: 0,
            page_token: None,
        }
    }
}

impl PaginationParams {
    /// Create pagination params with a specific page size.
    /// Page size is clamped to [1, MAX_PAGE_SIZE].
    pub fn with_page_size(mut self, size: usize) -> Self {
        self.page_size = size.clamp(1, MAX_PAGE_SIZE);
        self
    }

    /// Apply one-based page number using the current page size.
    pub fn with_page(mut self, page: usize) -> Self {
        let page = page.max(1);
        self.offset = (page - 1) * self.page_size;
        self
    }

    /// Create pagination params with a cursor token.
    pub fn with_page_token(mut self, token: impl Into<String>) -> Self {
        self.page_token = Some(token.into());
        self
    }

    /// Create pagination from optional limit and offset.
    /// This provides backward compatibility with legacy limit/offset APIs.
    pub fn from_limit_offset(limit: Option<usize>, offset: Option<usize>) -> Self {
        let page_size = limit.unwrap_or(DEFAULT_PAGE_SIZE).clamp(1, MAX_PAGE_SIZE);
        let offset = offset.unwrap_or(0);
        let page = offset / page_size + 1;
        Self::default().with_page_size(page_size).with_page(page)
    }
}

/// Paginated result set for list operations.
/// Includes pagination metadata for API responses.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PaginatedResult<T> {
    /// Items in the current page
    pub items: Vec<T>,
    /// Token to fetch the next page, if more results exist
    pub next_page_token: Option<String>,
    /// Total count of items (optional, may be expensive to compute)
    pub total_count: Option<u64>,
    /// Whether there are more results available
    pub has_more: bool,
}

impl<T> PaginatedResult<T> {
    /// Create a paginated result from items, computing has_more from next_page_token.
    pub fn new(items: Vec<T>, next_page_token: Option<String>, total_count: Option<u64>) -> Self {
        let has_more = next_page_token.is_some();
        Self {
            items,
            next_page_token,
            total_count,
            has_more,
        }
    }

    /// Create an empty paginated result.
    pub fn empty() -> Self {
        Self {
            items: Vec::new(),
            next_page_token: None,
            total_count: Some(0),
            has_more: false,
        }
    }

    /// Map items to a different type.
    pub fn map<U, F: Fn(T) -> U>(self, f: F) -> PaginatedResult<U> {
        PaginatedResult {
            items: self.items.into_iter().map(f).collect(),
            next_page_token: self.next_page_token,
            total_count: self.total_count,
            has_more: self.has_more,
        }
    }
}

// ---------------------------------------------------------------------------
// Query types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentListQuery {
    pub tenant_id: u64,
    pub organization_id: Option<u64>,
    pub owner_user_id: Option<u64>,
    pub include_deleted: bool,
    pub search_query: Option<String>,
    /// When set, restricts results to a single visibility level (for example public marketplace).
    pub visibility: Option<AgentVisibility>,
    pub pagination: PaginationParams,
}

impl AgentListQuery {
    pub fn for_tenant(tenant_id: u64) -> Self {
        Self {
            tenant_id,
            organization_id: None,
            owner_user_id: None,
            include_deleted: false,
            search_query: None,
            visibility: None,
            pagination: PaginationParams::default(),
        }
    }

    pub fn for_organization(mut self, organization_id: u64) -> Self {
        self.organization_id = Some(organization_id);
        self
    }

    pub fn for_owner(mut self, owner_user_id: u64) -> Self {
        self.owner_user_id = Some(owner_user_id);
        self
    }

    pub fn with_deleted(mut self) -> Self {
        self.include_deleted = true;
        self
    }

    pub fn with_search(mut self, query: impl Into<String>) -> Self {
        let query = query.into();
        self.search_query = optional_non_blank(query);
        self
    }

    /// Set pagination parameters for the query.
    pub fn with_pagination(mut self, pagination: PaginationParams) -> Self {
        self.pagination = pagination;
        self
    }

    /// Set page size for the query.
    pub fn with_page_size(mut self, size: usize) -> Self {
        self.pagination = self.pagination.with_page_size(size);
        self
    }

    /// Set page token for cursor-based pagination.
    pub fn with_page_token(mut self, token: impl Into<String>) -> Self {
        self.pagination = self.pagination.with_page_token(token);
        self
    }

    /// Restrict list results to one visibility level.
    pub fn with_visibility(mut self, visibility: AgentVisibility) -> Self {
        self.visibility = Some(visibility);
        self
    }

    /// Restrict list results to publicly visible agents (marketplace scope).
    pub fn with_public_visibility_only(mut self) -> Self {
        self.visibility = Some(AgentVisibility::Public);
        self
    }
}

/// Query parameters for listing provider bindings under one agent.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProviderBindingListQuery {
    pub tenant_id: u64,
    pub agent_id: String,
    pub pagination: PaginationParams,
}

impl ProviderBindingListQuery {
    pub fn for_agent(tenant_id: u64, agent_id: impl Into<String>) -> Self {
        Self {
            tenant_id,
            agent_id: agent_id.into(),
            pagination: PaginationParams::default(),
        }
    }

    pub fn with_pagination(mut self, pagination: PaginationParams) -> Self {
        self.pagination = pagination;
        self
    }
}

/// Query parameters for listing composition slots under one agent.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompositionSlotListQuery {
    pub tenant_id: u64,
    pub agent_id: String,
    pub pagination: PaginationParams,
}

impl CompositionSlotListQuery {
    pub fn for_agent(tenant_id: u64, agent_id: impl Into<String>) -> Self {
        Self {
            tenant_id,
            agent_id: agent_id.into(),
            pagination: PaginationParams::default(),
        }
    }

    pub fn with_pagination(mut self, pagination: PaginationParams) -> Self {
        self.pagination = pagination;
        self
    }
}

/// Query parameters for listing persisted audit events for one agent.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuditEventListQuery {
    pub tenant_id: u64,
    pub agent_id: String,
    pub action: Option<String>,
    pub from: Option<String>,
    pub to: Option<String>,
    pub pagination: PaginationParams,
    pub cursor: Option<AuditEventListCursor>,
}

impl AuditEventListQuery {
    pub fn for_agent(tenant_id: u64, agent_id: impl Into<String>) -> Self {
        Self {
            tenant_id,
            agent_id: agent_id.into(),
            action: None,
            from: None,
            to: None,
            pagination: PaginationParams::default(),
            cursor: None,
        }
    }

    pub fn with_action(mut self, action: impl Into<String>) -> Self {
        self.action = optional_non_blank(action.into());
        self
    }

    pub fn with_from(mut self, from: impl Into<String>) -> Self {
        self.from = optional_non_blank(from.into());
        self
    }

    pub fn with_to(mut self, to: impl Into<String>) -> Self {
        self.to = optional_non_blank(to.into());
        self
    }

    pub fn with_pagination(mut self, pagination: PaginationParams) -> Self {
        self.pagination = pagination;
        self
    }

    pub fn scope_fingerprint(&self) -> String {
        audit_list_scope_fingerprint(
            self.tenant_id,
            &self.agent_id,
            self.action.as_deref(),
            self.from.as_deref(),
            self.to.as_deref(),
        )
    }

    /// Switch to keyset mode: fetch `page_size + 1` rows so the caller can
    /// derive `has_more` and the next opaque cursor.
    pub fn with_cursor_page(
        mut self,
        page_size: usize,
        cursor: Option<AuditEventListCursor>,
    ) -> Self {
        self.pagination = PaginationParams::default().with_page_size(page_size);
        self.cursor = cursor;
        self
    }
}

/// Query parameters for listing MCP marketplace catalog entries.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct McpMarketplaceListQuery {
    pub tenant_id: u64,
    pub q: Option<String>,
    pub pagination: PaginationParams,
}

impl McpMarketplaceListQuery {
    pub fn for_tenant(tenant_id: u64) -> Self {
        Self {
            tenant_id,
            q: None,
            pagination: PaginationParams::default(),
        }
    }

    pub fn with_q(mut self, q: impl Into<String>) -> Self {
        let value = q.into().trim().to_string();
        self.q = if value.is_empty() { None } else { Some(value) };
        self
    }

    pub fn with_pagination(mut self, pagination: PaginationParams) -> Self {
        self.pagination = pagination;
        self
    }
}

/// Message list sort order. Default API lists use ascending sequence.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SessionItemListSort {
    #[default]
    SequenceAsc,
    /// Public list order for newest items first, with normal offset pagination.
    SequenceDesc,
    /// Recent context window for turn execution (descending sequence, bounded limit).
    RecentContextDesc,
}

impl SessionItemListSort {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::SequenceAsc => "sequence",
            Self::SequenceDesc => "-sequence",
            Self::RecentContextDesc => "recent-context",
        }
    }
}

/// Query parameters for listing agent sessions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionListQuery {
    pub tenant_id: u64,
    pub organization_id: Option<u64>,
    pub agent_id: Option<String>,
    pub project_id: Option<String>,
    pub workspace_id: Option<String>,
    pub owner_user_id: Option<u64>,
    pub status: Option<String>,
    pub include_archived: bool,
    pub pagination: PaginationParams,
    pub cursor: Option<SessionListCursor>,
}

impl SessionListQuery {
    pub fn scope_fingerprint(&self) -> String {
        session_list_scope_fingerprint(
            self.tenant_id,
            self.organization_id,
            self.agent_id.as_deref(),
            self.project_id.as_deref(),
            self.workspace_id.as_deref(),
            self.owner_user_id,
            self.status.as_deref(),
            self.include_archived,
        )
    }

    /// Switch to keyset mode: fetch `page_size + 1` rows so the caller can
    /// derive `has_more` and the next opaque cursor.
    pub fn with_cursor_page(mut self, page_size: usize, cursor: Option<SessionListCursor>) -> Self {
        self.pagination = PaginationParams::default().with_page_size(page_size);
        self.cursor = cursor;
        self
    }
}

/// Owner-scoped cursor query for the canonical Session activity projection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionActivitySummaryListQuery {
    pub tenant_id: u64,
    pub organization_id: u64,
    pub owner_user_id: u64,
    pub agent_id: Option<String>,
    pub project_id: Option<String>,
    pub workspace_id: Option<String>,
    pub cursor: Option<SessionActivityCursor>,
    pub page_size: usize,
}

impl SessionActivitySummaryListQuery {
    pub fn for_owner(tenant_id: u64, organization_id: u64, owner_user_id: u64) -> Self {
        Self {
            tenant_id,
            organization_id,
            owner_user_id,
            agent_id: None,
            project_id: None,
            workspace_id: None,
            cursor: None,
            page_size: DEFAULT_PAGE_SIZE,
        }
    }

    pub fn for_agent(mut self, agent_id: impl Into<String>) -> Self {
        self.agent_id = optional_non_blank(agent_id.into());
        self
    }

    pub fn for_project(mut self, project_id: impl Into<String>) -> Self {
        self.project_id = optional_non_blank(project_id.into());
        self
    }

    pub fn for_workspace(mut self, workspace_id: impl Into<String>) -> Self {
        self.workspace_id = optional_non_blank(workspace_id.into());
        self
    }

    pub fn after(mut self, cursor: SessionActivityCursor) -> Self {
        self.cursor = Some(cursor);
        self
    }

    pub fn with_page_size(mut self, page_size: usize) -> Self {
        self.page_size = page_size;
        self
    }

    pub fn scope_fingerprint(&self) -> String {
        session_activity_scope_fingerprint(
            self.tenant_id,
            self.organization_id,
            self.owner_user_id,
            self.workspace_id.as_deref(),
            self.project_id.as_deref(),
            self.agent_id.as_deref(),
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectListQuery {
    pub tenant_id: u64,
    pub organization_id: u64,
    pub owner_user_id: Option<u64>,
    pub workspace_id: Option<String>,
    pub exact_name: Option<String>,
    pub status: Option<AgentProjectStatus>,
    pub search_query: Option<String>,
    pub include_deleted: bool,
    pub pagination: PaginationParams,
}

impl ProjectListQuery {
    pub fn for_organization(tenant_id: u64, organization_id: u64) -> Self {
        Self {
            tenant_id,
            organization_id,
            owner_user_id: None,
            workspace_id: None,
            exact_name: None,
            status: None,
            search_query: None,
            include_deleted: false,
            pagination: PaginationParams::default(),
        }
    }

    pub fn for_owner(mut self, owner_user_id: u64) -> Self {
        self.owner_user_id = Some(owner_user_id);
        self
    }

    pub fn for_workspace(mut self, workspace_id: impl Into<String>) -> Self {
        self.workspace_id = optional_non_blank(workspace_id.into());
        self
    }

    pub fn with_status(mut self, status: AgentProjectStatus) -> Self {
        self.status = Some(status);
        self
    }

    pub fn with_exact_name(mut self, exact_name: impl Into<String>) -> Self {
        self.exact_name = optional_non_blank(exact_name.into());
        self
    }

    pub fn with_search(mut self, search_query: impl Into<String>) -> Self {
        self.search_query = optional_non_blank(search_query.into());
        self
    }

    pub fn with_pagination(mut self, pagination: PaginationParams) -> Self {
        self.pagination = pagination;
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkspaceListQuery {
    pub tenant_id: u64,
    pub organization_id: u64,
    pub owner_user_id: u64,
    pub status: Option<AgentWorkspaceStatus>,
    pub include_deleted: bool,
    pub pagination: PaginationParams,
}

impl WorkspaceListQuery {
    pub fn for_owner(tenant_id: u64, organization_id: u64, owner_user_id: u64) -> Self {
        Self {
            tenant_id,
            organization_id,
            owner_user_id,
            status: None,
            include_deleted: false,
            pagination: PaginationParams::default(),
        }
    }

    pub fn with_pagination(mut self, pagination: PaginationParams) -> Self {
        self.pagination = pagination;
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectCompositionSlotListQuery {
    pub tenant_id: u64,
    pub organization_id: u64,
    pub project_id: String,
    pub slot_kind: Option<AgentCompositionSlotKind>,
    pub enabled: Option<bool>,
    pub pagination: PaginationParams,
}

impl ProjectCompositionSlotListQuery {
    pub fn for_project(
        tenant_id: u64,
        organization_id: u64,
        project_id: impl Into<String>,
    ) -> Self {
        Self {
            tenant_id,
            organization_id,
            project_id: project_id.into(),
            slot_kind: None,
            enabled: None,
            pagination: PaginationParams::default(),
        }
    }

    pub fn with_pagination(mut self, pagination: PaginationParams) -> Self {
        self.pagination = pagination;
        self
    }
}

impl SessionListQuery {
    pub fn for_tenant(tenant_id: u64) -> Self {
        Self {
            tenant_id,
            organization_id: None,
            agent_id: None,
            project_id: None,
            workspace_id: None,
            owner_user_id: None,
            status: None,
            include_archived: false,
            pagination: PaginationParams::default(),
            cursor: None,
        }
    }

    pub fn for_agent(mut self, agent_id: impl Into<String>) -> Self {
        self.agent_id = Some(agent_id.into());
        self
    }

    pub fn for_organization(mut self, organization_id: u64) -> Self {
        self.organization_id = Some(organization_id);
        self
    }

    pub fn for_project(mut self, project_id: impl Into<String>) -> Self {
        self.project_id = Some(project_id.into());
        self
    }

    pub fn for_workspace(mut self, workspace_id: impl Into<String>) -> Self {
        self.workspace_id = Some(workspace_id.into());
        self
    }

    pub fn for_owner(mut self, owner_user_id: u64) -> Self {
        self.owner_user_id = Some(owner_user_id);
        self
    }

    pub fn with_status(mut self, status: impl Into<String>) -> Self {
        self.status = Some(status.into());
        self
    }

    pub fn include_archived(mut self) -> Self {
        self.include_archived = true;
        self
    }

    pub fn with_pagination(mut self, pagination: PaginationParams) -> Self {
        self.pagination = pagination;
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourceUserStateListQuery {
    pub tenant_id: u64,
    pub organization_id: u64,
    pub user_id: u64,
    pub resource_type: AgentResourceType,
    pub agent_id: Option<String>,
    pub resource_ids: Vec<String>,
    pub pinned_only: bool,
    pub include_hidden: bool,
    pub pagination: PaginationParams,
}

impl ResourceUserStateListQuery {
    pub fn for_user_sessions(tenant_id: u64, organization_id: u64, user_id: u64) -> Self {
        Self {
            tenant_id,
            organization_id,
            user_id,
            resource_type: AgentResourceType::Session,
            agent_id: None,
            resource_ids: Vec::new(),
            pinned_only: false,
            include_hidden: false,
            pagination: PaginationParams::default(),
        }
    }

    pub fn pinned_only(mut self) -> Self {
        self.pinned_only = true;
        self
    }

    pub fn for_agent(mut self, agent_id: impl Into<String>) -> Self {
        self.agent_id = Some(agent_id.into());
        self
    }

    pub fn for_resource_ids(mut self, resource_ids: Vec<String>) -> Self {
        self.resource_ids = resource_ids;
        self
    }

    pub fn with_pagination(mut self, pagination: PaginationParams) -> Self {
        self.pagination = pagination;
        self
    }
}

/// Query parameters for listing agent items within a session.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionItemListQuery {
    pub tenant_id: u64,
    pub organization_id: u64,
    pub session_id: String,
    pub kind: Option<String>,
    pub status: Option<String>,
    pub sort: SessionItemListSort,
    pub(crate) cursor_mode: bool,
    pub(crate) cursor: Option<SessionItemCursor>,
    pub pagination: PaginationParams,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ItemFeedbackListQuery {
    pub tenant_id: u64,
    pub organization_id: u64,
    pub user_id: u64,
    pub session_id: String,
    pub pagination: PaginationParams,
}

impl ItemFeedbackListQuery {
    pub fn for_user_session(
        tenant_id: u64,
        organization_id: u64,
        user_id: u64,
        session_id: impl Into<String>,
    ) -> Self {
        Self {
            tenant_id,
            organization_id,
            user_id,
            session_id: session_id.into(),
            pagination: PaginationParams::default(),
        }
    }

    pub fn with_pagination(mut self, pagination: PaginationParams) -> Self {
        self.pagination = pagination;
        self
    }
}

impl SessionItemListQuery {
    pub fn for_session(
        tenant_id: u64,
        organization_id: u64,
        session_id: impl Into<String>,
    ) -> Self {
        Self {
            tenant_id,
            organization_id,
            session_id: session_id.into(),
            kind: None,
            status: None,
            sort: SessionItemListSort::default(),
            cursor_mode: false,
            cursor: None,
            pagination: PaginationParams::default(),
        }
    }

    pub fn with_kind(mut self, kind: impl Into<String>) -> Self {
        self.kind = Some(kind.into());
        self
    }

    pub fn with_status(mut self, status: impl Into<String>) -> Self {
        self.status = Some(status.into());
        self
    }

    pub fn with_pagination(mut self, pagination: PaginationParams) -> Self {
        self.pagination = pagination;
        self
    }

    pub fn with_sort(mut self, sort: SessionItemListSort) -> Self {
        self.sort = sort;
        self
    }

    pub(crate) fn with_cursor_page(
        mut self,
        page_size: usize,
        cursor: Option<SessionItemCursor>,
    ) -> Self {
        self.cursor_mode = true;
        self.cursor = cursor;
        self.pagination = PaginationParams::default().with_page_size(page_size);
        self
    }

    pub(crate) fn repository_page_size(&self) -> usize {
        self.pagination
            .page_size
            .saturating_add(usize::from(self.cursor_mode))
    }

    pub(crate) fn cursor_scope_fingerprint(&self) -> String {
        session_item_scope_fingerprint(
            self.tenant_id,
            self.organization_id,
            &self.session_id,
            self.kind.as_deref(),
            self.status.as_deref(),
            self.sort.as_str(),
        )
    }

    /// Load the most recent items for turn execution context.
    pub fn for_recent_turn_context(
        tenant_id: u64,
        organization_id: u64,
        session_id: impl Into<String>,
        limit: usize,
    ) -> Self {
        Self {
            tenant_id,
            organization_id,
            session_id: session_id.into(),
            kind: None,
            status: None,
            sort: SessionItemListSort::RecentContextDesc,
            cursor_mode: false,
            cursor: None,
            pagination: PaginationParams::default().with_page_size(limit),
        }
    }
}

/// Query parameters for listing live interactions within a session.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InteractionListQuery {
    pub tenant_id: u64,
    pub organization_id: u64,
    pub session_id: String,
    pub kind: Option<String>,
    pub status: Option<String>,
    pub pagination: PaginationParams,
    pub cursor: Option<CreatedAtListCursor>,
}

impl InteractionListQuery {
    pub fn scope_fingerprint(&self) -> String {
        interaction_list_scope_fingerprint(
            self.tenant_id,
            self.organization_id,
            &self.session_id,
            self.kind.as_deref(),
            self.status.as_deref(),
        )
    }

    /// Switch to keyset mode: fetch `page_size + 1` rows so the caller can
    /// derive `has_more` and the next opaque cursor.
    pub fn with_cursor_page(
        mut self,
        page_size: usize,
        cursor: Option<CreatedAtListCursor>,
    ) -> Self {
        self.pagination = PaginationParams::default().with_page_size(page_size);
        self.cursor = cursor;
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionRuntimeBindingListQuery {
    pub tenant_id: u64,
    pub organization_id: u64,
    pub session_id: String,
    pub status: Option<String>,
    pub current_only: bool,
    pub pagination: PaginationParams,
}

impl SessionRuntimeBindingListQuery {
    pub fn for_session(
        tenant_id: u64,
        organization_id: u64,
        session_id: impl Into<String>,
    ) -> Self {
        Self {
            tenant_id,
            organization_id,
            session_id: session_id.into(),
            status: None,
            current_only: false,
            pagination: PaginationParams::default(),
        }
    }

    pub fn with_status(mut self, status: impl Into<String>) -> Self {
        self.status = Some(status.into());
        self
    }

    pub fn current_only(mut self) -> Self {
        self.current_only = true;
        self
    }

    pub fn with_pagination(mut self, pagination: PaginationParams) -> Self {
        self.pagination = pagination;
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionCheckpointListQuery {
    pub tenant_id: u64,
    pub organization_id: u64,
    pub session_id: String,
    pub status: Option<String>,
    pub pagination: PaginationParams,
}

impl SessionCheckpointListQuery {
    pub fn for_session(
        tenant_id: u64,
        organization_id: u64,
        session_id: impl Into<String>,
    ) -> Self {
        Self {
            tenant_id,
            organization_id,
            session_id: session_id.into(),
            status: None,
            pagination: PaginationParams::default(),
        }
    }

    pub fn with_status(mut self, status: impl Into<String>) -> Self {
        self.status = Some(status.into());
        self
    }

    pub fn with_pagination(mut self, pagination: PaginationParams) -> Self {
        self.pagination = pagination;
        self
    }
}

impl InteractionListQuery {
    pub fn for_session(
        tenant_id: u64,
        organization_id: u64,
        session_id: impl Into<String>,
    ) -> Self {
        Self {
            tenant_id,
            organization_id,
            session_id: session_id.into(),
            kind: None,
            status: None,
            pagination: PaginationParams::default(),
            cursor: None,
        }
    }

    pub fn with_kind(mut self, kind: impl Into<String>) -> Self {
        self.kind = Some(kind.into());
        self
    }

    pub fn with_status(mut self, status: impl Into<String>) -> Self {
        self.status = Some(status.into());
        self
    }

    pub fn with_pagination(mut self, pagination: PaginationParams) -> Self {
        self.pagination = pagination;
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TurnListQuery {
    pub tenant_id: u64,
    pub organization_id: u64,
    pub session_id: String,
    pub status: Option<String>,
    pub pagination: PaginationParams,
    pub cursor: Option<CreatedAtListCursor>,
}

impl TurnListQuery {
    pub fn for_session(
        tenant_id: u64,
        organization_id: u64,
        session_id: impl Into<String>,
    ) -> Self {
        Self {
            tenant_id,
            organization_id,
            session_id: session_id.into(),
            status: None,
            pagination: PaginationParams::default(),
            cursor: None,
        }
    }

    pub fn with_status(mut self, status: impl Into<String>) -> Self {
        self.status = Some(status.into());
        self
    }

    pub fn with_pagination(mut self, pagination: PaginationParams) -> Self {
        self.pagination = pagination;
        self
    }

    pub fn scope_fingerprint(&self) -> String {
        turn_list_scope_fingerprint(
            self.tenant_id,
            self.organization_id,
            &self.session_id,
            self.status.as_deref(),
        )
    }

    /// Switch to keyset mode: fetch `page_size + 1` rows so the caller can
    /// derive `has_more` and the next opaque cursor.
    pub fn with_cursor_page(
        mut self,
        page_size: usize,
        cursor: Option<CreatedAtListCursor>,
    ) -> Self {
        self.pagination = PaginationParams::default().with_page_size(page_size);
        self.cursor = cursor;
        self
    }
}

/// Query parameters for listing agent tasks.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskListQuery {
    pub tenant_id: u64,
    pub organization_id: u64,
    pub agent_id: Option<String>,
    pub owner_user_id: Option<u64>,
    pub status: Option<String>,
    pub cursor: Option<TaskCursor>,
    pub page_size: usize,
}

impl TaskListQuery {
    pub fn for_organization(tenant_id: u64, organization_id: u64) -> Self {
        Self {
            tenant_id,
            organization_id,
            agent_id: None,
            owner_user_id: None,
            status: None,
            cursor: None,
            page_size: DEFAULT_PAGE_SIZE,
        }
    }

    pub fn for_agent(mut self, agent_id: impl Into<String>) -> Self {
        self.agent_id = Some(agent_id.into());
        self
    }

    pub fn for_owner(mut self, owner_user_id: u64) -> Self {
        self.owner_user_id = Some(owner_user_id);
        self
    }

    pub fn with_status(mut self, status: impl Into<String>) -> Self {
        self.status = Some(status.into());
        self
    }

    pub fn with_cursor_page(mut self, page_size: usize, cursor: Option<TaskCursor>) -> Self {
        self.page_size = page_size.clamp(1, MAX_PAGE_SIZE);
        self.cursor = cursor;
        self
    }

    pub fn scope_fingerprint(&self) -> String {
        task_scope_fingerprint(
            self.tenant_id,
            self.organization_id,
            self.agent_id.as_deref(),
            self.owner_user_id,
            self.status.as_deref(),
        )
    }

    pub fn store_limit(&self) -> usize {
        self.page_size.saturating_add(1)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TurnRequestWriteOutcome {
    Inserted {
        session: Box<AgentSessionRecord>,
        request_item: Box<AgentSessionItemRecord>,
    },
    Existing(Box<AgentTurnRecord>),
}

/// Thread-safe agent repository port.
///
/// All methods use `&self` — implementations MUST provide interior mutability
/// (e.g. `Mutex`, `RwLock`, or atomic database transactions). This eliminates
/// the need for a global `Mutex<AgentsService>` and enables true concurrent
/// request processing.
pub trait AgentRepository: Send + Sync {
    /// Verify that the repository's required backing store can serve requests.
    fn check_readiness(&self) -> KernelResult<()>;

    fn next_id(&self) -> KernelResult<u64>;

    fn insert(&self, record: AgentBusinessRecord) -> KernelResult<()>;

    fn update(&self, record: AgentBusinessRecord) -> KernelResult<()>;

    fn get(&self, tenant_id: u64, agent_id: &str) -> KernelResult<Option<AgentBusinessRecord>>;

    /// List one page of agents. Pagination is enforced at the repository layer.
    fn list(&self, query: &AgentListQuery) -> KernelResult<Vec<AgentBusinessRecord>>;

    /// Count agents matching list filters (excluding pagination bounds).
    fn count_agents(&self, query: &AgentListQuery) -> KernelResult<u64>;

    fn insert_workspace(&self, record: AgentWorkspaceRecord) -> KernelResult<()>;

    fn update_workspace(&self, record: AgentWorkspaceRecord) -> KernelResult<()>;

    fn get_workspace(
        &self,
        tenant_id: u64,
        organization_id: u64,
        workspace_id: &str,
    ) -> KernelResult<Option<AgentWorkspaceRecord>>;

    fn get_default_workspace(
        &self,
        tenant_id: u64,
        organization_id: u64,
        owner_user_id: u64,
    ) -> KernelResult<Option<AgentWorkspaceRecord>>;

    fn list_workspaces(
        &self,
        query: &WorkspaceListQuery,
    ) -> KernelResult<Vec<AgentWorkspaceRecord>>;

    fn count_workspaces(&self, query: &WorkspaceListQuery) -> KernelResult<u64>;

    fn insert_project(&self, record: AgentProjectRecord) -> KernelResult<()>;

    fn update_project(&self, record: AgentProjectRecord) -> KernelResult<()>;

    fn get_project(
        &self,
        tenant_id: u64,
        organization_id: u64,
        project_id: &str,
    ) -> KernelResult<Option<AgentProjectRecord>>;

    fn get_project_by_workspace_name(
        &self,
        tenant_id: u64,
        organization_id: u64,
        workspace_id: &str,
        name: &str,
    ) -> KernelResult<Option<AgentProjectRecord>>;

    fn get_project_by_import_source(
        &self,
        tenant_id: u64,
        organization_id: u64,
        owner_user_id: u64,
        source_kind: &str,
        source_ref: &str,
    ) -> KernelResult<Option<AgentProjectRecord>>;

    fn list_projects(&self, query: &ProjectListQuery) -> KernelResult<Vec<AgentProjectRecord>>;

    fn count_projects(&self, query: &ProjectListQuery) -> KernelResult<u64>;

    fn insert_project_composition_slot(
        &self,
        record: AgentProjectCompositionSlotRecord,
    ) -> KernelResult<()>;

    fn update_project_composition_slot(
        &self,
        record: AgentProjectCompositionSlotRecord,
    ) -> KernelResult<()>;

    fn get_project_composition_slot(
        &self,
        tenant_id: u64,
        organization_id: u64,
        project_id: &str,
        slot_id: &str,
    ) -> KernelResult<Option<AgentProjectCompositionSlotRecord>>;

    fn list_project_composition_slots(
        &self,
        query: &ProjectCompositionSlotListQuery,
    ) -> KernelResult<Vec<AgentProjectCompositionSlotRecord>>;

    fn count_project_composition_slots(
        &self,
        query: &ProjectCompositionSlotListQuery,
    ) -> KernelResult<u64>;

    /// List agents with pagination metadata.
    fn list_paginated(
        &self,
        query: &AgentListQuery,
    ) -> KernelResult<PaginatedResult<AgentBusinessRecord>> {
        let total_count = self.count_agents(query)?;
        let items = self.list(query)?;
        Ok(offset_paginated_result(
            items,
            &query.pagination,
            total_count,
        ))
    }

    fn insert_provider_binding(&self, record: AgentProviderBindingRecord) -> KernelResult<()>;

    fn update_provider_binding(&self, record: AgentProviderBindingRecord) -> KernelResult<()>;

    fn get_provider_binding(
        &self,
        tenant_id: u64,
        agent_id: &str,
        binding_id: &str,
    ) -> KernelResult<Option<AgentProviderBindingRecord>>;

    /// Load the single active provider binding for an agent.
    ///
    /// Implementations SHOULD override this with a dedicated indexed query
    /// (`WHERE active = TRUE LIMIT 1`) to avoid paginated full scans on hot
    /// paths such as turn execution and task execution. The default
    /// implementation falls back to a paginated scan for backward compatibility.
    fn get_active_provider_binding(
        &self,
        tenant_id: u64,
        agent_id: &str,
    ) -> KernelResult<Option<AgentProviderBindingRecord>>;

    fn list_provider_bindings(
        &self,
        query: &ProviderBindingListQuery,
    ) -> KernelResult<Vec<AgentProviderBindingRecord>>;

    fn count_provider_bindings(&self, query: &ProviderBindingListQuery) -> KernelResult<u64>;

    /// Atomically deactivate all active bindings for an agent and activate one target binding.
    fn activate_provider_binding_atomic(
        &self,
        tenant_id: u64,
        agent_id: &str,
        binding_id: &str,
        updated_at: String,
    ) -> KernelResult<AgentProviderBindingRecord> {
        let mut record = self
            .get_provider_binding(tenant_id, agent_id, binding_id)?
            .ok_or_else(|| KernelError::not_found("agent provider binding not found"))?;
        if record.active {
            return Ok(record);
        }
        let mut page = 1usize;
        loop {
            let batch = self.list_provider_bindings(
                &ProviderBindingListQuery::for_agent(tenant_id, agent_id).with_pagination(
                    PaginationParams::default()
                        .with_page_size(MAX_PAGE_SIZE)
                        .with_page(page),
                ),
            )?;
            if batch.is_empty() {
                break;
            }
            let batch_len = batch.len();
            for mut binding in batch {
                if binding.active {
                    binding.active = false;
                    binding.mark_updated(updated_at.clone());
                    self.update_provider_binding(binding)?;
                }
            }
            if batch_len < MAX_PAGE_SIZE {
                break;
            }
            page = page.saturating_add(1);
        }
        record = self
            .get_provider_binding(tenant_id, agent_id, binding_id)?
            .ok_or_else(|| KernelError::not_found("agent provider binding not found"))?;
        record.active = true;
        record.mark_updated(updated_at);
        self.update_provider_binding(record.clone())?;
        Ok(record)
    }

    fn insert_composition_slot(&self, record: AgentCompositionSlotRecord) -> KernelResult<()>;

    fn update_composition_slot(&self, record: AgentCompositionSlotRecord) -> KernelResult<()>;

    fn get_composition_slot(
        &self,
        tenant_id: u64,
        agent_id: &str,
        slot_id: &str,
    ) -> KernelResult<Option<AgentCompositionSlotRecord>>;

    fn list_composition_slots(
        &self,
        query: &CompositionSlotListQuery,
    ) -> KernelResult<Vec<AgentCompositionSlotRecord>>;

    fn count_composition_slots(&self, query: &CompositionSlotListQuery) -> KernelResult<u64>;

    fn list_mcp_marketplace_slots(
        &self,
        query: &McpMarketplaceListQuery,
    ) -> KernelResult<Vec<AgentCompositionSlotRecord>>;

    fn count_mcp_marketplace_slots(&self, query: &McpMarketplaceListQuery) -> KernelResult<u64>;

    // -----------------------------------------------------------------------
    // Session persistence
    // -----------------------------------------------------------------------

    fn insert_session(&self, record: AgentSessionRecord) -> KernelResult<()>;

    fn update_session(&self, record: AgentSessionRecord) -> KernelResult<()>;

    /// Atomically soft-deletes the session and purges its turn-input-queue
    /// entries. Durable backends run both writes in one transaction so a
    /// partial failure cannot leave a deleted session with queued inputs.
    fn delete_session_and_purge_queue(
        &self,
        deleted_session: AgentSessionRecord,
        tenant_id: u64,
        organization_id: u64,
        session_id: &str,
        owner_user_id: u64,
    ) -> KernelResult<()>;

    fn get_session(
        &self,
        tenant_id: u64,
        organization_id: u64,
        session_id: &str,
    ) -> KernelResult<Option<AgentSessionRecord>>;

    fn get_session_by_creation_idempotency(
        &self,
        tenant_id: u64,
        organization_id: u64,
        owner_user_id: u64,
        idempotency_key: &str,
    ) -> KernelResult<Option<AgentSessionRecord>>;

    fn list_sessions(&self, query: &SessionListQuery) -> KernelResult<Vec<AgentSessionRecord>>;

    fn count_sessions(&self, query: &SessionListQuery) -> KernelResult<u64>;

    fn list_session_activity_summaries(
        &self,
        query: &SessionActivitySummaryListQuery,
    ) -> KernelResult<PaginatedResult<SessionActivitySummaryRecord>>;

    fn insert_session_runtime_binding(
        &self,
        record: AgentSessionRuntimeBindingRecord,
    ) -> KernelResult<()>;

    fn update_session_runtime_binding(
        &self,
        record: AgentSessionRuntimeBindingRecord,
    ) -> KernelResult<()>;

    fn get_session_runtime_binding(
        &self,
        tenant_id: u64,
        organization_id: u64,
        session_id: &str,
        runtime_binding_id: &str,
    ) -> KernelResult<Option<AgentSessionRuntimeBindingRecord>>;

    /// Finds the runtime binding that currently claims a provider Session
    /// identity across Sessions, so legacy imports can be retired before the
    /// canonical provider-history Session takes the identity over.
    fn get_session_runtime_binding_by_provider_session(
        &self,
        tenant_id: u64,
        organization_id: u64,
        owner_user_id: u64,
        provider_binding_id: &str,
        provider_session_id: &str,
    ) -> KernelResult<Option<AgentSessionRuntimeBindingRecord>>;

    fn get_current_session_runtime_binding(
        &self,
        tenant_id: u64,
        organization_id: u64,
        session_id: &str,
    ) -> KernelResult<Option<AgentSessionRuntimeBindingRecord>>;

    fn list_session_runtime_bindings(
        &self,
        query: &SessionRuntimeBindingListQuery,
    ) -> KernelResult<Vec<AgentSessionRuntimeBindingRecord>>;

    fn count_session_runtime_bindings(
        &self,
        query: &SessionRuntimeBindingListQuery,
    ) -> KernelResult<u64>;

    /// Switches the current runtime binding under one repository transaction.
    fn activate_session_runtime_binding_atomic(
        &self,
        tenant_id: u64,
        organization_id: u64,
        session_id: &str,
        runtime_binding_id: &str,
        expected_version: u64,
        updated_at: String,
    ) -> KernelResult<AgentSessionRuntimeBindingRecord>;

    fn insert_session_checkpoint(&self, record: AgentSessionCheckpointRecord) -> KernelResult<()>;

    fn update_session_checkpoint(&self, record: AgentSessionCheckpointRecord) -> KernelResult<()>;

    fn get_session_checkpoint(
        &self,
        tenant_id: u64,
        organization_id: u64,
        session_id: &str,
        checkpoint_id: &str,
    ) -> KernelResult<Option<AgentSessionCheckpointRecord>>;

    fn list_session_checkpoints(
        &self,
        query: &SessionCheckpointListQuery,
    ) -> KernelResult<Vec<AgentSessionCheckpointRecord>>;

    fn count_session_checkpoints(&self, query: &SessionCheckpointListQuery) -> KernelResult<u64>;

    fn upsert_resource_user_state(
        &self,
        record: AgentResourceUserStateRecord,
        expected_version: Option<u64>,
    ) -> KernelResult<AgentResourceUserStateRecord>;

    fn get_resource_user_state(
        &self,
        tenant_id: u64,
        organization_id: u64,
        user_id: u64,
        resource_type: AgentResourceType,
        resource_id: &str,
    ) -> KernelResult<Option<AgentResourceUserStateRecord>>;

    fn list_resource_user_states(
        &self,
        query: &ResourceUserStateListQuery,
    ) -> KernelResult<Vec<AgentResourceUserStateRecord>>;

    fn count_resource_user_states(&self, query: &ResourceUserStateListQuery) -> KernelResult<u64>;
    // -----------------------------------------------------------------------
    // Media tool configuration persistence (admin-managed)
    // -----------------------------------------------------------------------

    /// Upserts one tenant-scoped media tool configuration. A row is created
    /// on first write; `expected_version` enforces optimistic concurrency for
    /// updates.
    fn upsert_tool_configuration(
        &self,
        record: AgentToolConfigurationRecord,
        expected_version: Option<u64>,
    ) -> KernelResult<AgentToolConfigurationRecord>;

    /// Reads one tenant-scoped tool configuration, or `None` when the tenant
    /// has never written one (registry defaults apply).
    fn get_tool_configuration(
        &self,
        tenant_id: u64,
        organization_id: u64,
        tool_id: &str,
    ) -> KernelResult<Option<AgentToolConfigurationRecord>>;

    /// Lists tenant tool configurations for admin surfaces, newest first.
    fn list_tool_configurations(
        &self,
        tenant_id: u64,
        organization_id: u64,
    ) -> KernelResult<Vec<AgentToolConfigurationRecord>>;

    // -----------------------------------------------------------------------
    // Generated-media asset persistence (saveToDrive registration)
    // -----------------------------------------------------------------------

    /// Inserts one generated-media asset record (idempotent per
    /// space+node scope).
    fn insert_tool_asset(&self, record: AgentToolAssetRecord) -> KernelResult<()>;

    /// Lists generated-media assets for one tenant/user, newest first.
    fn list_tool_assets(
        &self,
        tenant_id: u64,
        organization_id: u64,
        user_id: u64,
        limit: u64,
    ) -> KernelResult<Vec<AgentToolAssetRecord>>;

    // -----------------------------------------------------------------------
    // Session-item persistence
    // -----------------------------------------------------------------------

    /// Atomically append one item and update its owning session counters.
    fn append_session_item(
        &self,
        record: AgentSessionItemRecord,
    ) -> KernelResult<(AgentSessionRecord, AgentSessionItemRecord)>;

    fn update_session_item(&self, record: AgentSessionItemRecord) -> KernelResult<()>;

    fn get_session_item(
        &self,
        tenant_id: u64,
        organization_id: u64,
        session_id: &str,
        item_id: &str,
    ) -> KernelResult<Option<AgentSessionItemRecord>>;

    fn list_session_items(
        &self,
        query: &SessionItemListQuery,
    ) -> KernelResult<Vec<AgentSessionItemRecord>>;

    fn count_session_items(&self, query: &SessionItemListQuery) -> KernelResult<u64>;

    /// Load the bounded, ordered item set owned by one durable Turn.
    fn list_session_items_by_turn(
        &self,
        tenant_id: u64,
        organization_id: u64,
        session_id: &str,
        turn_id: &str,
        limit: usize,
    ) -> KernelResult<Vec<AgentSessionItemRecord>>;

    fn upsert_item_feedback(
        &self,
        record: AgentItemFeedbackRecord,
        expected_version: Option<u64>,
    ) -> KernelResult<AgentItemFeedbackRecord>;

    fn get_item_feedback(
        &self,
        tenant_id: u64,
        organization_id: u64,
        item_id: &str,
        user_id: u64,
        include_deleted: bool,
    ) -> KernelResult<Option<AgentItemFeedbackRecord>>;

    fn list_item_feedback(
        &self,
        query: &ItemFeedbackListQuery,
    ) -> KernelResult<Vec<AgentItemFeedbackRecord>>;

    fn count_item_feedback(&self, query: &ItemFeedbackListQuery) -> KernelResult<u64>;

    fn get_turn_by_idempotency(
        &self,
        tenant_id: u64,
        organization_id: u64,
        owner_user_id: u64,
        idempotency_key: &str,
    ) -> KernelResult<Option<AgentTurnRecord>>;
    fn get_turn(
        &self,
        tenant_id: u64,
        organization_id: u64,
        turn_id: &str,
    ) -> KernelResult<Option<AgentTurnRecord>>;

    fn list_turns(&self, query: &TurnListQuery) -> KernelResult<Vec<AgentTurnRecord>>;

    fn count_turns(&self, query: &TurnListQuery) -> KernelResult<u64>;

    fn get_turn_input_queue_entry(
        &self,
        tenant_id: u64,
        organization_id: u64,
        session_id: &str,
        owner_user_id: u64,
        queue_entry_id: &str,
    ) -> KernelResult<Option<AgentTurnInputQueueEntry>>;

    fn list_turn_input_queue_entries(
        &self,
        query: &TurnInputQueueListQuery,
        owner_user_id: u64,
    ) -> KernelResult<Vec<AgentTurnInputQueueEntry>>;

    fn count_turn_input_queue_entries(
        &self,
        query: &TurnInputQueueListQuery,
        owner_user_id: u64,
    ) -> KernelResult<u64>;

    fn insert_turn_input_queue_entry(
        &self,
        entry: AgentTurnInputQueueEntry,
    ) -> KernelResult<AgentTurnInputQueueEntry>;

    fn update_turn_input_queue_entry(
        &self,
        entry: AgentTurnInputQueueEntry,
        expected_version: u64,
    ) -> KernelResult<AgentTurnInputQueueEntry>;

    fn remove_turn_input_queue_entry(
        &self,
        tenant_id: u64,
        organization_id: u64,
        session_id: &str,
        owner_user_id: u64,
        queue_entry_id: &str,
        expected_version: u64,
    ) -> KernelResult<AgentTurnInputQueueEntry>;

    fn clear_turn_input_queue_entries(
        &self,
        tenant_id: u64,
        organization_id: u64,
        session_id: &str,
        owner_user_id: u64,
    ) -> KernelResult<u64>;

    fn purge_turn_input_queue_entries(
        &self,
        tenant_id: u64,
        organization_id: u64,
        session_id: &str,
        owner_user_id: u64,
    ) -> KernelResult<u64>;

    fn reorder_turn_input_queue_entries(
        &self,
        tenant_id: u64,
        organization_id: u64,
        session_id: &str,
        owner_user_id: u64,
        entries: &[TurnInputQueueReorderEntry],
        requested_at: &str,
    ) -> KernelResult<Vec<AgentTurnInputQueueEntry>>;

    fn claim_next_turn_input_queue_entry(
        &self,
        request: &TurnInputQueueClaimRequest,
    ) -> KernelResult<TurnInputQueueClaimOutcome>;

    fn fail_turn_input_queue_entry(
        &self,
        request: &TurnInputQueueFailureRequest,
    ) -> KernelResult<AgentTurnInputQueueEntry>;

    fn list_reconcilable_turns(
        &self,
        stale_before: &str,
        limit: usize,
    ) -> KernelResult<Vec<AgentTurnRecord>>;

    /// Checkpoints accumulated streaming deltas onto the running turn row so
    /// a crash during a long turn retains the partial reply (H4).
    fn append_turn_streaming_content(
        &self,
        tenant_id: u64,
        organization_id: u64,
        turn_id: &str,
        content: &str,
        updated_at: &str,
    ) -> KernelResult<()>;

    /// Clears the streaming checkpoint after the turn completes durably.
    fn clear_turn_streaming_content(
        &self,
        tenant_id: u64,
        organization_id: u64,
        turn_id: &str,
    ) -> KernelResult<()>;

    /// Atomically persist a requested turn, its user-input item, any Drive
    /// references, and the corresponding session counter update.
    fn insert_turn_request(
        &self,
        turn: AgentTurnRecord,
        request_item: AgentSessionItemRecord,
        drive_refs: Vec<AgentItemDriveRefRecord>,
    ) -> KernelResult<TurnRequestWriteOutcome>;

    fn update_turn_state(
        &self,
        turn: AgentTurnRecord,
        expected_version: u64,
    ) -> KernelResult<AgentTurnRecord>;

    /// Atomically persist terminal provider items plus the final assistant
    /// response, transition the Turn, and update Session counters.
    fn complete_turn(
        &self,
        turn: AgentTurnRecord,
        expected_turn_version: u64,
        expected_fencing_token: u64,
        expected_lease_token: Option<String>,
        completed_items: Vec<AgentSessionItemRecord>,
    ) -> KernelResult<(AgentSessionRecord, Vec<AgentSessionItemRecord>)>;

    fn list_item_drive_refs(
        &self,
        tenant_id: u64,
        organization_id: u64,
        item_id: &str,
    ) -> KernelResult<Vec<AgentItemDriveRefRecord>>;

    fn list_item_drive_refs_batch(
        &self,
        tenant_id: u64,
        organization_id: u64,
        item_ids: &[String],
    ) -> KernelResult<Vec<AgentItemDriveRefRecord>> {
        let mut records = Vec::new();
        for item_id in item_ids {
            records.extend(self.list_item_drive_refs(tenant_id, organization_id, item_id)?);
        }
        Ok(records)
    }

    // -----------------------------------------------------------------------
    // Interaction persistence
    // -----------------------------------------------------------------------

    fn insert_interaction(&self, record: AgentInteractionRecord) -> KernelResult<()>;

    fn update_interaction(&self, record: AgentInteractionRecord) -> KernelResult<()>;

    fn get_interaction(
        &self,
        tenant_id: u64,
        organization_id: u64,
        session_id: &str,
        interaction_id: &str,
    ) -> KernelResult<Option<AgentInteractionRecord>>;

    fn list_interactions(
        &self,
        query: &InteractionListQuery,
    ) -> KernelResult<Vec<AgentInteractionRecord>>;

    fn count_interactions(&self, query: &InteractionListQuery) -> KernelResult<u64>;

    // -----------------------------------------------------------------------
    // Task persistence
    // -----------------------------------------------------------------------

    fn insert_task(&self, record: AgentTaskRecord) -> KernelResult<()>;

    fn update_task(&self, record: AgentTaskRecord) -> KernelResult<()>;

    fn get_task(
        &self,
        tenant_id: u64,
        organization_id: u64,
        task_id: &str,
    ) -> KernelResult<Option<AgentTaskRecord>>;

    fn list_tasks(&self, query: &TaskListQuery) -> KernelResult<Vec<AgentTaskRecord>>;
}

/// Thread-safe audit event sink port.
///
/// All methods use `&self` for the same reason as `AgentRepository`.
pub trait AgentAuditSink: Send + Sync {
    fn record(&self, event: KernelEvent) -> KernelResult<()>;

    fn list_events(
        &self,
        query: &AuditEventListQuery,
    ) -> KernelResult<PaginatedResult<KernelEvent>>;
}

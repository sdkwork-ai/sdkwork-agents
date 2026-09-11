#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ApiOperation {
    pub method: &'static str,
    pub path: &'static str,
    pub tag: &'static str,
    pub operation_id: &'static str,
}

pub const AGENT_OPEN_API_PREFIX: &str = "/agent/v3/api";
pub const AGENT_APP_API_PREFIX: &str = "/app/v3/api";
pub const AGENT_BACKEND_API_PREFIX: &str = "/backend/v3/api";

pub const AGENT_OPEN_API_OPERATIONS: &[ApiOperation] = &[
    ApiOperation {
        method: "GET",
        path: "/agent/v3/api/ai/agents",
        tag: "ai",
        operation_id: "agents.list",
    },
    ApiOperation {
        method: "POST",
        path: "/agent/v3/api/ai/agents",
        tag: "ai",
        operation_id: "agents.create",
    },
    ApiOperation {
        method: "GET",
        path: "/agent/v3/api/ai/agents/{agentId}",
        tag: "ai",
        operation_id: "agents.retrieve",
    },
    ApiOperation {
        method: "PATCH",
        path: "/agent/v3/api/ai/agents/{agentId}",
        tag: "ai",
        operation_id: "agents.update",
    },
    ApiOperation {
        method: "DELETE",
        path: "/agent/v3/api/ai/agents/{agentId}",
        tag: "ai",
        operation_id: "agents.delete",
    },
    ApiOperation {
        method: "GET",
        path: "/agent/v3/api/ai/agents/{agentId}/provider_bindings",
        tag: "ai",
        operation_id: "agents.providerBindings.list",
    },
    ApiOperation {
        method: "POST",
        path: "/agent/v3/api/ai/agents/{agentId}/provider_bindings",
        tag: "ai",
        operation_id: "agents.providerBindings.create",
    },
    ApiOperation {
        method: "POST",
        path: "/agent/v3/api/ai/agents/{agentId}/provider_bindings/{bindingId}/activate",
        tag: "ai",
        operation_id: "agents.providerBindings.activate",
    },
    ApiOperation {
        method: "POST",
        path: "/agent/v3/api/ai/agents/{agentId}/preview_responses",
        tag: "ai",
        operation_id: "agents.previewResponses.create",
    },
    ApiOperation {
        method: "POST",
        path: "/agent/v3/api/ai/agents/{agentId}/prompt_optimizations",
        tag: "ai",
        operation_id: "agents.promptOptimizations.create",
    },
    ApiOperation {
        method: "GET",
        path: "/agent/v3/api/ai/agents/{agentId}/sessions",
        tag: "ai",
        operation_id: "agents.sessions.list",
    },
    ApiOperation {
        method: "POST",
        path: "/agent/v3/api/ai/agents/{agentId}/sessions",
        tag: "ai",
        operation_id: "agents.sessions.create",
    },
    ApiOperation {
        method: "GET",
        path: "/agent/v3/api/ai/agents/{agentId}/sessions/{sessionId}",
        tag: "ai",
        operation_id: "agents.sessions.retrieve",
    },
    ApiOperation {
        method: "POST",
        path: "/agent/v3/api/ai/agents/{agentId}/sessions/{sessionId}/close",
        tag: "ai",
        operation_id: "agents.sessions.close",
    },
    ApiOperation {
        method: "GET",
        path: "/agent/v3/api/ai/agents/{agentId}/sessions/{sessionId}/items",
        tag: "ai",
        operation_id: "agents.sessionItems.list",
    },
    ApiOperation {
        method: "GET",
        path: "/agent/v3/api/ai/agents/{agentId}/sessions/{sessionId}/items/{itemId}",
        tag: "ai",
        operation_id: "agents.sessionItems.retrieve",
    },
    ApiOperation {
        method: "GET",
        path: "/agent/v3/api/ai/agents/{agentId}/sessions/{sessionId}/turns",
        tag: "ai",
        operation_id: "agents.turns.list",
    },
    ApiOperation {
        method: "POST",
        path: "/agent/v3/api/ai/agents/{agentId}/sessions/{sessionId}/turns",
        tag: "ai",
        operation_id: "agents.turns.stream",
    },
    ApiOperation {
        method: "GET",
        path: "/agent/v3/api/ai/agents/{agentId}/sessions/{sessionId}/turns/{turnId}",
        tag: "ai",
        operation_id: "agents.turns.retrieve",
    },
    ApiOperation {
        method: "POST",
        path: "/agent/v3/api/ai/agents/{agentId}/sessions/{sessionId}/turns/{turnId}/cancel",
        tag: "ai",
        operation_id: "agents.turns.cancel",
    },
    ApiOperation {
        method: "GET",
        path: "/agent/v3/api/ai/agents/{agentId}/sessions/{sessionId}/interactions",
        tag: "ai",
        operation_id: "agents.interactions.list",
    },
    ApiOperation {
        method: "POST",
        path: "/agent/v3/api/ai/agents/{agentId}/sessions/{sessionId}/interactions",
        tag: "ai",
        operation_id: "agents.interactions.create",
    },
    ApiOperation {
        method: "GET",
        path: "/agent/v3/api/ai/agents/{agentId}/sessions/{sessionId}/interactions/{interactionId}",
        tag: "ai",
        operation_id: "agents.interactions.retrieve",
    },
    ApiOperation {
        method: "POST",
        path: "/agent/v3/api/ai/agents/{agentId}/sessions/{sessionId}/interactions/{interactionId}/claim",
        tag: "ai",
        operation_id: "agents.interactions.claim",
    },
    ApiOperation {
        method: "POST",
        path: "/agent/v3/api/ai/agents/{agentId}/sessions/{sessionId}/interactions/{interactionId}/approve",
        tag: "ai",
        operation_id: "agents.interactions.approve",
    },
    ApiOperation {
        method: "POST",
        path: "/agent/v3/api/ai/agents/{agentId}/sessions/{sessionId}/interactions/{interactionId}/answer",
        tag: "ai",
        operation_id: "agents.interactions.answer",
    },
    ApiOperation {
        method: "POST",
        path: "/agent/v3/api/ai/agents/{agentId}/sessions/{sessionId}/interactions/{interactionId}/resolve",
        tag: "ai",
        operation_id: "agents.interactions.resolve",
    },
    ApiOperation {
        method: "GET",
        path: "/agent/v3/api/ai/agents/{agentId}/sessions/{sessionId}/checkpoints",
        tag: "ai",
        operation_id: "agents.checkpoints.list",
    },
    ApiOperation {
        method: "POST",
        path: "/agent/v3/api/ai/agents/{agentId}/sessions/{sessionId}/checkpoints",
        tag: "ai",
        operation_id: "agents.checkpoints.create",
    },
    ApiOperation {
        method: "GET",
        path: "/agent/v3/api/ai/agents/{agentId}/sessions/{sessionId}/checkpoints/{checkpointId}",
        tag: "ai",
        operation_id: "agents.checkpoints.retrieve",
    },
    ApiOperation {
        method: "POST",
        path: "/agent/v3/api/ai/agents/{agentId}/sessions/{sessionId}/checkpoints/{checkpointId}/restore",
        tag: "ai",
        operation_id: "agents.checkpoints.restore",
    },
    ApiOperation {
        method: "POST",
        path: "/agent/v3/api/ai/agents/{agentId}/sessions/{sessionId}/checkpoints/{checkpointId}/invalidate",
        tag: "ai",
        operation_id: "agents.checkpoints.invalidate",
    },
    ApiOperation {
        method: "GET",
        path: "/agent/v3/api/ai/agents/{agentId}/sessions/{sessionId}/runtime_bindings",
        tag: "ai",
        operation_id: "agents.sessionRuntimeBindings.list",
    },
    ApiOperation {
        method: "POST",
        path: "/agent/v3/api/ai/agents/{agentId}/sessions/{sessionId}/runtime_bindings",
        tag: "ai",
        operation_id: "agents.sessionRuntimeBindings.create",
    },
    ApiOperation {
        method: "GET",
        path: "/agent/v3/api/ai/agents/{agentId}/sessions/{sessionId}/runtime_bindings/{runtimeBindingId}",
        tag: "ai",
        operation_id: "agents.sessionRuntimeBindings.retrieve",
    },
    ApiOperation {
        method: "PATCH",
        path: "/agent/v3/api/ai/agents/{agentId}/sessions/{sessionId}/runtime_bindings/{runtimeBindingId}",
        tag: "ai",
        operation_id: "agents.sessionRuntimeBindings.update",
    },
    ApiOperation {
        method: "POST",
        path: "/agent/v3/api/ai/agents/{agentId}/sessions/{sessionId}/runtime_bindings/{runtimeBindingId}/activate",
        tag: "ai",
        operation_id: "agents.sessionRuntimeBindings.activate",
    },
    ApiOperation {
        method: "POST",
        path: "/agent/v3/api/ai/agents/{agentId}/sessions/{sessionId}/runtime_bindings/{runtimeBindingId}/deactivate",
        tag: "ai",
        operation_id: "agents.sessionRuntimeBindings.deactivate",
    },
    ApiOperation {
        method: "GET",
        path: "/agent/v3/api/ai/agents/{agentId}/tasks",
        tag: "ai",
        operation_id: "agents.tasks.list",
    },
    ApiOperation {
        method: "POST",
        path: "/agent/v3/api/ai/agents/{agentId}/tasks",
        tag: "ai",
        operation_id: "agents.tasks.create",
    },
    ApiOperation {
        method: "GET",
        path: "/agent/v3/api/ai/agents/{agentId}/tasks/{taskId}",
        tag: "ai",
        operation_id: "agents.tasks.retrieve",
    },
    ApiOperation {
        method: "POST",
        path: "/agent/v3/api/ai/agents/{agentId}/tasks/{taskId}/cancel",
        tag: "ai",
        operation_id: "agents.tasks.cancel",
    },
    ApiOperation {
        method: "POST",
        path: "/agent/v3/api/ai/agents/{agentId}/tasks/{taskId}/execute",
        tag: "ai",
        operation_id: "agents.tasks.execute",
    },
    ApiOperation {
        method: "PUT",
        path: "/agent/v3/api/ai/agents/{agentId}/tasks/{taskId}",
        tag: "ai",
        operation_id: "agents.tasks.update",
    },
    ApiOperation {
        method: "POST",
        path: "/agent/v3/api/ai/agents/{agentId}/tasks/{taskId}/pause",
        tag: "ai",
        operation_id: "agents.tasks.pause",
    },
    ApiOperation {
        method: "POST",
        path: "/agent/v3/api/ai/agents/{agentId}/tasks/{taskId}/resume",
        tag: "ai",
        operation_id: "agents.tasks.resume",
    },
    ApiOperation {
        method: "GET",
        path: "/agent/v3/api/ai/agents/{agentId}/tasks/{taskId}/runs",
        tag: "ai",
        operation_id: "agents.taskRuns.list",
    },
    ApiOperation {
        method: "GET",
        path: "/agent/v3/api/ai/agents/{agentId}/tasks/{taskId}/runs/{runId}",
        tag: "ai",
        operation_id: "agents.taskRuns.retrieve",
    },
    ApiOperation {
        method: "POST",
        path: "/agent/v3/api/ai/agents/{agentId}/tasks/{taskId}/runs/{runId}/retry",
        tag: "ai",
        operation_id: "agents.taskRuns.retry",
    },
    ApiOperation {
        method: "POST",
        path: "/agent/v3/api/ai/agents/{agentId}/tasks/{taskId}/runs/{runId}/cancel",
        tag: "ai",
        operation_id: "agents.taskRuns.cancel",
    },
    ApiOperation {
        method: "GET",
        path: "/agent/v3/api/ai/agents/{agentId}/tasks/{taskId}/runs/{runId}/attempts",
        tag: "ai",
        operation_id: "agents.taskRunAttempts.list",
    },
    ApiOperation {
        method: "GET",
        path: "/agent/v3/api/ai/agents/{agentId}/composition_slots",
        tag: "ai",
        operation_id: "agents.compositionSlots.list",
    },
    ApiOperation {
        method: "POST",
        path: "/agent/v3/api/ai/agents/{agentId}/composition_slots",
        tag: "ai",
        operation_id: "agents.compositionSlots.create",
    },
    ApiOperation {
        method: "GET",
        path: "/agent/v3/api/ai/agents/{agentId}/composition_slots/{slotId}",
        tag: "ai",
        operation_id: "agents.compositionSlots.retrieve",
    },
    ApiOperation {
        method: "PATCH",
        path: "/agent/v3/api/ai/agents/{agentId}/composition_slots/{slotId}",
        tag: "ai",
        operation_id: "agents.compositionSlots.update",
    },
    ApiOperation {
        method: "DELETE",
        path: "/agent/v3/api/ai/agents/{agentId}/composition_slots/{slotId}",
        tag: "ai",
        operation_id: "agents.compositionSlots.delete",
    },
];

pub const AGENT_APP_API_OPERATIONS: &[ApiOperation] = &[
    ApiOperation {
        method: "GET",
        path: "/app/v3/api/ai/agents",
        tag: "ai",
        operation_id: "agents.list",
    },
    ApiOperation {
        method: "POST",
        path: "/app/v3/api/ai/agents",
        tag: "ai",
        operation_id: "agents.create",
    },
    ApiOperation {
        method: "GET",
        path: "/app/v3/api/ai/agents/{agentId}",
        tag: "ai",
        operation_id: "agents.retrieve",
    },
    ApiOperation {
        method: "PATCH",
        path: "/app/v3/api/ai/agents/{agentId}",
        tag: "ai",
        operation_id: "agents.update",
    },
    ApiOperation {
        method: "DELETE",
        path: "/app/v3/api/ai/agents/{agentId}",
        tag: "ai",
        operation_id: "agents.delete",
    },
    ApiOperation {
        method: "POST",
        path: "/app/v3/api/ai/agents/{agentId}/restore",
        tag: "ai",
        operation_id: "agents.restore",
    },
    ApiOperation {
        method: "GET",
        path: "/app/v3/api/ai/agents/{agentId}/provider_bindings",
        tag: "ai",
        operation_id: "agents.providerBindings.list",
    },
    ApiOperation {
        method: "POST",
        path: "/app/v3/api/ai/agents/{agentId}/provider_bindings",
        tag: "ai",
        operation_id: "agents.providerBindings.create",
    },
    ApiOperation {
        method: "POST",
        path: "/app/v3/api/ai/agents/{agentId}/provider_bindings/{bindingId}/activate",
        tag: "ai",
        operation_id: "agents.providerBindings.activate",
    },
    ApiOperation {
        method: "POST",
        path: "/app/v3/api/ai/agents/{agentId}/preview_responses",
        tag: "ai",
        operation_id: "agents.previewResponses.create",
    },
    ApiOperation {
        method: "POST",
        path: "/app/v3/api/ai/agents/{agentId}/prompt_optimizations",
        tag: "ai",
        operation_id: "agents.promptOptimizations.create",
    },
    ApiOperation {
        method: "POST",
        path: "/app/v3/api/ai/agents/{agentId}/calls",
        tag: "ai",
        operation_id: "agents.calls.create",
    },
    ApiOperation {
        method: "GET",
        path: "/app/v3/api/ai/agents/{agentId}/calls",
        tag: "ai",
        operation_id: "agents.calls.list",
    },
    ApiOperation {
        method: "GET",
        path: "/app/v3/api/ai/agents/{agentId}/calls/{executionId}",
        tag: "ai",
        operation_id: "agents.calls.retrieve",
    },
    ApiOperation {
        method: "POST",
        path: "/app/v3/api/ai/agents/{agentId}/versions",
        tag: "ai",
        operation_id: "agents.versions.create",
    },
    ApiOperation {
        method: "GET",
        path: "/app/v3/api/ai/agents/{agentId}/versions",
        tag: "ai",
        operation_id: "agents.versions.list",
    },
    ApiOperation {
        method: "GET",
        path: "/app/v3/api/ai/agents/{agentId}/versions/{versionId}",
        tag: "ai",
        operation_id: "agents.versions.retrieve",
    },
    ApiOperation {
        method: "POST",
        path: "/app/v3/api/ai/agents/{agentId}/versions/{versionId}/activate",
        tag: "ai",
        operation_id: "agents.versions.activate",
    },
    ApiOperation {
        method: "GET",
        path: "/app/v3/api/ai/usage/summary",
        tag: "ai",
        operation_id: "agents.usage.summary.retrieve",
    },
    ApiOperation {
        method: "GET",
        path: "/app/v3/api/ai/usage/records",
        tag: "ai",
        operation_id: "agents.usage.records.list",
    },
    ApiOperation {
        method: "POST",
        path: "/app/v3/api/ai/webhooks",
        tag: "ai",
        operation_id: "agents.webhooks.create",
    },
    ApiOperation {
        method: "GET",
        path: "/app/v3/api/ai/webhooks",
        tag: "ai",
        operation_id: "agents.webhooks.list",
    },
    ApiOperation {
        method: "GET",
        path: "/app/v3/api/ai/webhooks/{webhookId}",
        tag: "ai",
        operation_id: "agents.webhooks.retrieve",
    },
    ApiOperation {
        method: "DELETE",
        path: "/app/v3/api/ai/webhooks/{webhookId}",
        tag: "ai",
        operation_id: "agents.webhooks.delete",
    },
    ApiOperation {
        method: "POST",
        path: "/app/v3/api/ai/webhooks/{webhookId}/test",
        tag: "ai",
        operation_id: "agents.webhooks.test",
    },
    ApiOperation {
        method: "GET",
        path: "/app/v3/api/ai/workspaces",
        tag: "ai",
        operation_id: "agents.workspaces.list",
    },
    ApiOperation {
        method: "POST",
        path: "/app/v3/api/ai/workspaces",
        tag: "ai",
        operation_id: "agents.workspaces.create",
    },
    ApiOperation {
        method: "POST",
        path: "/app/v3/api/ai/workspaces/default",
        tag: "ai",
        operation_id: "agents.workspaces.default.create",
    },
    ApiOperation {
        method: "GET",
        path: "/app/v3/api/ai/workspaces/{workspaceId}",
        tag: "ai",
        operation_id: "agents.workspaces.retrieve",
    },
    ApiOperation {
        method: "PATCH",
        path: "/app/v3/api/ai/workspaces/{workspaceId}",
        tag: "ai",
        operation_id: "agents.workspaces.update",
    },
    ApiOperation {
        method: "DELETE",
        path: "/app/v3/api/ai/workspaces/{workspaceId}",
        tag: "ai",
        operation_id: "agents.workspaces.delete",
    },
    ApiOperation {
        method: "POST",
        path: "/app/v3/api/ai/workspaces/{workspaceId}/archive",
        tag: "ai",
        operation_id: "agents.workspaces.archive",
    },
    ApiOperation {
        method: "GET",
        path: "/app/v3/api/ai/workspaces/{workspaceId}/sessions",
        tag: "ai",
        operation_id: "agents.workspaceSessions.list",
    },
    ApiOperation {
        method: "GET",
        path: "/app/v3/api/ai/projects",
        tag: "ai",
        operation_id: "agents.projects.list",
    },
    ApiOperation {
        method: "POST",
        path: "/app/v3/api/ai/projects",
        tag: "ai",
        operation_id: "agents.projects.create",
    },
    ApiOperation {
        method: "POST",
        path: "/app/v3/api/ai/projects/import",
        tag: "ai",
        operation_id: "agents.projects.import",
    },
    ApiOperation {
        method: "GET",
        path: "/app/v3/api/ai/projects/{projectId}",
        tag: "ai",
        operation_id: "agents.projects.retrieve",
    },
    ApiOperation {
        method: "PATCH",
        path: "/app/v3/api/ai/projects/{projectId}",
        tag: "ai",
        operation_id: "agents.projects.update",
    },
    ApiOperation {
        method: "DELETE",
        path: "/app/v3/api/ai/projects/{projectId}",
        tag: "ai",
        operation_id: "agents.projects.delete",
    },
    ApiOperation {
        method: "POST",
        path: "/app/v3/api/ai/projects/{projectId}/archive",
        tag: "ai",
        operation_id: "agents.projects.archive",
    },
    ApiOperation {
        method: "GET",
        path: "/app/v3/api/ai/projects/{projectId}/composition_slots",
        tag: "ai",
        operation_id: "agents.projectCompositionSlots.list",
    },
    ApiOperation {
        method: "POST",
        path: "/app/v3/api/ai/projects/{projectId}/composition_slots",
        tag: "ai",
        operation_id: "agents.projectCompositionSlots.create",
    },
    ApiOperation {
        method: "GET",
        path: "/app/v3/api/ai/projects/{projectId}/composition_slots/{slotId}",
        tag: "ai",
        operation_id: "agents.projectCompositionSlots.retrieve",
    },
    ApiOperation {
        method: "PATCH",
        path: "/app/v3/api/ai/projects/{projectId}/composition_slots/{slotId}",
        tag: "ai",
        operation_id: "agents.projectCompositionSlots.update",
    },
    ApiOperation {
        method: "DELETE",
        path: "/app/v3/api/ai/projects/{projectId}/composition_slots/{slotId}",
        tag: "ai",
        operation_id: "agents.projectCompositionSlots.delete",
    },
    ApiOperation {
        method: "GET",
        path: "/app/v3/api/ai/projects/{projectId}/sessions",
        tag: "ai",
        operation_id: "agents.projectSessions.list",
    },
    ApiOperation {
        method: "POST",
        path: "/app/v3/api/ai/projects/{projectId}/sessions/synchronize",
        tag: "ai",
        operation_id: "agents.projectSessions.synchronize",
    },
    ApiOperation {
        method: "POST",
        path: "/app/v3/api/ai/projects/{projectId}/sessions",
        tag: "ai",
        operation_id: "agents.projectSessions.create",
    },
    ApiOperation {
        method: "GET",
        path: "/app/v3/api/ai/projects/{projectId}/sessions/{sessionId}",
        tag: "ai",
        operation_id: "agents.projectSessions.retrieve",
    },
    ApiOperation {
        method: "GET",
        path: "/app/v3/api/ai/session_activity_summaries",
        tag: "ai",
        operation_id: "agents.sessionActivitySummaries.list",
    },
    ApiOperation {
        method: "GET",
        path: "/app/v3/api/ai/agents/{agentId}/sessions",
        tag: "ai",
        operation_id: "agents.sessions.list",
    },
    ApiOperation {
        method: "POST",
        path: "/app/v3/api/ai/agents/{agentId}/sessions",
        tag: "ai",
        operation_id: "agents.sessions.create",
    },
    ApiOperation {
        method: "GET",
        path: "/app/v3/api/ai/agents/{agentId}/sessions/user_states",
        tag: "ai",
        operation_id: "agents.sessionUserStates.list",
    },
    ApiOperation {
        method: "GET",
        path: "/app/v3/api/ai/agents/{agentId}/sessions/{sessionId}",
        tag: "ai",
        operation_id: "agents.sessions.retrieve",
    },
    ApiOperation {
        method: "PATCH",
        path: "/app/v3/api/ai/agents/{agentId}/sessions/{sessionId}",
        tag: "ai",
        operation_id: "agents.sessions.update",
    },
    ApiOperation {
        method: "DELETE",
        path: "/app/v3/api/ai/agents/{agentId}/sessions/{sessionId}",
        tag: "ai",
        operation_id: "agents.sessions.delete",
    },
    ApiOperation {
        method: "POST",
        path: "/app/v3/api/ai/agents/{agentId}/sessions/{sessionId}/close",
        tag: "ai",
        operation_id: "agents.sessions.close",
    },
    ApiOperation {
        method: "GET",
        path: "/app/v3/api/ai/agents/{agentId}/sessions/{sessionId}/user_state",
        tag: "ai",
        operation_id: "agents.sessionUserStates.retrieve",
    },
    ApiOperation {
        method: "PATCH",
        path: "/app/v3/api/ai/agents/{agentId}/sessions/{sessionId}/user_state",
        tag: "ai",
        operation_id: "agents.sessionUserStates.update",
    },
    ApiOperation {
        method: "GET",
        path: "/app/v3/api/ai/agents/{agentId}/sessions/{sessionId}/item_feedback",
        tag: "ai",
        operation_id: "agents.itemFeedback.list",
    },
    ApiOperation {
        method: "GET",
        path: "/app/v3/api/ai/agents/{agentId}/sessions/{sessionId}/items",
        tag: "ai",
        operation_id: "agents.sessionItems.list",
    },
    ApiOperation {
        method: "POST",
        path: "/app/v3/api/ai/agents/{agentId}/sessions/{sessionId}/items/synchronize",
        tag: "ai",
        operation_id: "agents.sessionItems.synchronize",
    },
    ApiOperation {
        method: "GET",
        path: "/app/v3/api/ai/agents/{agentId}/sessions/{sessionId}/items/{itemId}",
        tag: "ai",
        operation_id: "agents.sessionItems.retrieve",
    },
    ApiOperation {
        method: "PATCH",
        path: "/app/v3/api/ai/agents/{agentId}/sessions/{sessionId}/items/{itemId}/feedback",
        tag: "ai",
        operation_id: "agents.itemFeedback.update",
    },
    ApiOperation {
        method: "GET",
        path: "/app/v3/api/ai/agents/{agentId}/sessions/{sessionId}/turns",
        tag: "ai",
        operation_id: "agents.turns.list",
    },
    ApiOperation {
        method: "POST",
        path: "/app/v3/api/ai/agents/{agentId}/sessions/{sessionId}/turns",
        tag: "ai",
        operation_id: "agents.turns.stream",
    },
    ApiOperation {
        method: "GET",
        path: "/app/v3/api/ai/agents/{agentId}/sessions/{sessionId}/turns/{turnId}",
        tag: "ai",
        operation_id: "agents.turns.retrieve",
    },
    ApiOperation {
        method: "POST",
        path: "/app/v3/api/ai/agents/{agentId}/sessions/{sessionId}/turns/{turnId}/cancel",
        tag: "ai",
        operation_id: "agents.turns.cancel",
    },
    ApiOperation {
        method: "GET",
        path: "/app/v3/api/ai/agents/{agentId}/sessions/{sessionId}/turn_input_queue",
        tag: "ai",
        operation_id: "agents.turnInputQueueEntries.list",
    },
    ApiOperation {
        method: "POST",
        path: "/app/v3/api/ai/agents/{agentId}/sessions/{sessionId}/turn_input_queue",
        tag: "ai",
        operation_id: "agents.turnInputQueueEntries.create",
    },
    ApiOperation {
        method: "PATCH",
        path: "/app/v3/api/ai/agents/{agentId}/sessions/{sessionId}/turn_input_queue/{queueEntryId}",
        tag: "ai",
        operation_id: "agents.turnInputQueueEntries.update",
    },
    ApiOperation {
        method: "DELETE",
        path: "/app/v3/api/ai/agents/{agentId}/sessions/{sessionId}/turn_input_queue/{queueEntryId}",
        tag: "ai",
        operation_id: "agents.turnInputQueueEntries.delete",
    },
    ApiOperation {
        method: "POST",
        path: "/app/v3/api/ai/agents/{agentId}/sessions/{sessionId}/turn_input_queue/clear",
        tag: "ai",
        operation_id: "agents.turnInputQueueEntries.clear",
    },
    ApiOperation {
        method: "POST",
        path: "/app/v3/api/ai/agents/{agentId}/sessions/{sessionId}/turn_input_queue/reorder",
        tag: "ai",
        operation_id: "agents.turnInputQueueEntries.reorder",
    },
    ApiOperation {
        method: "POST",
        path: "/app/v3/api/ai/agents/{agentId}/sessions/{sessionId}/turn_input_queue/claim_next",
        tag: "ai",
        operation_id: "agents.turnInputQueueEntries.claimNext",
    },
    ApiOperation {
        method: "POST",
        path: "/app/v3/api/ai/agents/{agentId}/sessions/{sessionId}/turn_input_queue/{queueEntryId}/fail",
        tag: "ai",
        operation_id: "agents.turnInputQueueEntries.fail",
    },
    ApiOperation {
        method: "POST",
        path: "/app/v3/api/ai/agents/{agentId}/sessions/{sessionId}/turn_input_queue/{queueEntryId}/retry",
        tag: "ai",
        operation_id: "agents.turnInputQueueEntries.retry",
    },
    ApiOperation {
        method: "GET",
        path: "/app/v3/api/ai/agents/{agentId}/sessions/{sessionId}/interactions",
        tag: "ai",
        operation_id: "agents.interactions.list",
    },
    ApiOperation {
        method: "POST",
        path: "/app/v3/api/ai/agents/{agentId}/sessions/{sessionId}/interactions",
        tag: "ai",
        operation_id: "agents.interactions.create",
    },
    ApiOperation {
        method: "GET",
        path: "/app/v3/api/ai/agents/{agentId}/sessions/{sessionId}/interactions/{interactionId}",
        tag: "ai",
        operation_id: "agents.interactions.retrieve",
    },
    ApiOperation {
        method: "POST",
        path: "/app/v3/api/ai/agents/{agentId}/sessions/{sessionId}/interactions/{interactionId}/claim",
        tag: "ai",
        operation_id: "agents.interactions.claim",
    },
    ApiOperation {
        method: "POST",
        path: "/app/v3/api/ai/agents/{agentId}/sessions/{sessionId}/interactions/{interactionId}/approve",
        tag: "ai",
        operation_id: "agents.interactions.approve",
    },
    ApiOperation {
        method: "POST",
        path: "/app/v3/api/ai/agents/{agentId}/sessions/{sessionId}/interactions/{interactionId}/answer",
        tag: "ai",
        operation_id: "agents.interactions.answer",
    },
    ApiOperation {
        method: "POST",
        path: "/app/v3/api/ai/agents/{agentId}/sessions/{sessionId}/interactions/{interactionId}/resolve",
        tag: "ai",
        operation_id: "agents.interactions.resolve",
    },
    ApiOperation {
        method: "GET",
        path: "/app/v3/api/ai/agents/{agentId}/sessions/{sessionId}/checkpoints",
        tag: "ai",
        operation_id: "agents.checkpoints.list",
    },
    ApiOperation {
        method: "POST",
        path: "/app/v3/api/ai/agents/{agentId}/sessions/{sessionId}/checkpoints",
        tag: "ai",
        operation_id: "agents.checkpoints.create",
    },
    ApiOperation {
        method: "GET",
        path: "/app/v3/api/ai/agents/{agentId}/sessions/{sessionId}/checkpoints/{checkpointId}",
        tag: "ai",
        operation_id: "agents.checkpoints.retrieve",
    },
    ApiOperation {
        method: "POST",
        path: "/app/v3/api/ai/agents/{agentId}/sessions/{sessionId}/checkpoints/{checkpointId}/restore",
        tag: "ai",
        operation_id: "agents.checkpoints.restore",
    },
    ApiOperation {
        method: "POST",
        path: "/app/v3/api/ai/agents/{agentId}/sessions/{sessionId}/checkpoints/{checkpointId}/invalidate",
        tag: "ai",
        operation_id: "agents.checkpoints.invalidate",
    },
    ApiOperation {
        method: "GET",
        path: "/app/v3/api/ai/agents/{agentId}/sessions/{sessionId}/runtime_bindings",
        tag: "ai",
        operation_id: "agents.sessionRuntimeBindings.list",
    },
    ApiOperation {
        method: "POST",
        path: "/app/v3/api/ai/agents/{agentId}/sessions/{sessionId}/runtime_bindings",
        tag: "ai",
        operation_id: "agents.sessionRuntimeBindings.create",
    },
    ApiOperation {
        method: "GET",
        path: "/app/v3/api/ai/agents/{agentId}/sessions/{sessionId}/runtime_bindings/{runtimeBindingId}",
        tag: "ai",
        operation_id: "agents.sessionRuntimeBindings.retrieve",
    },
    ApiOperation {
        method: "PATCH",
        path: "/app/v3/api/ai/agents/{agentId}/sessions/{sessionId}/runtime_bindings/{runtimeBindingId}",
        tag: "ai",
        operation_id: "agents.sessionRuntimeBindings.update",
    },
    ApiOperation {
        method: "POST",
        path: "/app/v3/api/ai/agents/{agentId}/sessions/{sessionId}/runtime_bindings/{runtimeBindingId}/activate",
        tag: "ai",
        operation_id: "agents.sessionRuntimeBindings.activate",
    },
    ApiOperation {
        method: "POST",
        path: "/app/v3/api/ai/agents/{agentId}/sessions/{sessionId}/runtime_bindings/{runtimeBindingId}/deactivate",
        tag: "ai",
        operation_id: "agents.sessionRuntimeBindings.deactivate",
    },
    ApiOperation {
        method: "GET",
        path: "/app/v3/api/ai/agents/{agentId}/tasks",
        tag: "ai",
        operation_id: "agents.tasks.list",
    },
    ApiOperation {
        method: "POST",
        path: "/app/v3/api/ai/agents/{agentId}/tasks",
        tag: "ai",
        operation_id: "agents.tasks.create",
    },
    ApiOperation {
        method: "GET",
        path: "/app/v3/api/ai/agents/{agentId}/tasks/{taskId}",
        tag: "ai",
        operation_id: "agents.tasks.retrieve",
    },
    ApiOperation {
        method: "POST",
        path: "/app/v3/api/ai/agents/{agentId}/tasks/{taskId}/cancel",
        tag: "ai",
        operation_id: "agents.tasks.cancel",
    },
    ApiOperation {
        method: "POST",
        path: "/app/v3/api/ai/agents/{agentId}/tasks/{taskId}/execute",
        tag: "ai",
        operation_id: "agents.tasks.execute",
    },
    ApiOperation {
        method: "PUT",
        path: "/app/v3/api/ai/agents/{agentId}/tasks/{taskId}",
        tag: "ai",
        operation_id: "agents.tasks.update",
    },
    ApiOperation {
        method: "POST",
        path: "/app/v3/api/ai/agents/{agentId}/tasks/{taskId}/pause",
        tag: "ai",
        operation_id: "agents.tasks.pause",
    },
    ApiOperation {
        method: "POST",
        path: "/app/v3/api/ai/agents/{agentId}/tasks/{taskId}/resume",
        tag: "ai",
        operation_id: "agents.tasks.resume",
    },
    ApiOperation {
        method: "GET",
        path: "/app/v3/api/ai/agents/{agentId}/tasks/{taskId}/runs",
        tag: "ai",
        operation_id: "agents.taskRuns.list",
    },
    ApiOperation {
        method: "GET",
        path: "/app/v3/api/ai/agents/{agentId}/tasks/{taskId}/runs/{runId}",
        tag: "ai",
        operation_id: "agents.taskRuns.retrieve",
    },
    ApiOperation {
        method: "POST",
        path: "/app/v3/api/ai/agents/{agentId}/tasks/{taskId}/runs/{runId}/retry",
        tag: "ai",
        operation_id: "agents.taskRuns.retry",
    },
    ApiOperation {
        method: "POST",
        path: "/app/v3/api/ai/agents/{agentId}/tasks/{taskId}/runs/{runId}/cancel",
        tag: "ai",
        operation_id: "agents.taskRuns.cancel",
    },
    ApiOperation {
        method: "GET",
        path: "/app/v3/api/ai/agents/{agentId}/tasks/{taskId}/runs/{runId}/attempts",
        tag: "ai",
        operation_id: "agents.taskRunAttempts.list",
    },
    ApiOperation {
        method: "GET",
        path: "/app/v3/api/ai/agents/{agentId}/composition_slots",
        tag: "ai",
        operation_id: "agents.compositionSlots.list",
    },
    ApiOperation {
        method: "POST",
        path: "/app/v3/api/ai/agents/{agentId}/composition_slots",
        tag: "ai",
        operation_id: "agents.compositionSlots.create",
    },
    ApiOperation {
        method: "GET",
        path: "/app/v3/api/ai/agents/{agentId}/composition_slots/{slotId}",
        tag: "ai",
        operation_id: "agents.compositionSlots.retrieve",
    },
    ApiOperation {
        method: "PATCH",
        path: "/app/v3/api/ai/agents/{agentId}/composition_slots/{slotId}",
        tag: "ai",
        operation_id: "agents.compositionSlots.update",
    },
    ApiOperation {
        method: "DELETE",
        path: "/app/v3/api/ai/agents/{agentId}/composition_slots/{slotId}",
        tag: "ai",
        operation_id: "agents.compositionSlots.delete",
    },
    ApiOperation {
        method: "GET",
        path: "/app/v3/api/ai/agent_engines",
        tag: "ai",
        operation_id: "agents.agentEngines.list",
    },
    ApiOperation {
        method: "POST",
        path: "/app/v3/api/ai/model_configurations/apply",
        tag: "ai",
        operation_id: "agents.modelConfigurations.apply",
    },
    ApiOperation {
        method: "GET",
        path: "/app/v3/api/ai/model_configurations",
        tag: "ai",
        operation_id: "agents.modelConfigurations.list",
    },
    ApiOperation {
        method: "GET",
        path: "/app/v3/api/ai/model_configurations/{engineId}/{profileId}",
        tag: "ai",
        operation_id: "agents.modelConfigurations.get",
    },
    ApiOperation {
        method: "GET",
        path: "/app/v3/api/ai/model_configurations/{engineId}/{profileId}/status",
        tag: "ai",
        operation_id: "agents.modelConfigurations.status",
    },
    ApiOperation {
        method: "GET",
        path: "/app/v3/api/ai/model_configurations/{engineId}/config_file",
        tag: "ai",
        operation_id: "agents.modelConfigurations.configFile",
    },
    ApiOperation {
        method: "POST",
        path: "/app/v3/api/ai/model_configurations/{engineId}/{profileId}/archive",
        tag: "ai",
        operation_id: "agents.modelConfigurations.archive",
    },
    ApiOperation {
        method: "POST",
        path: "/app/v3/api/ai/model_configurations/migrate",
        tag: "ai",
        operation_id: "agents.modelConfigurations.migrate",
    },
    ApiOperation {
        method: "POST",
        path: "/app/v3/api/ai/model_selections/apply",
        tag: "ai",
        operation_id: "agents.modelSelections.apply",
    },
    ApiOperation {
        method: "GET",
        path: "/app/v3/api/ai/mcp_servers",
        tag: "ai",
        operation_id: "agents.mcpServers.list",
    },
    ApiOperation {
        method: "GET",
        path: "/app/v3/api/ai/tools",
        tag: "ai",
        operation_id: "agents.tools.list",
    },
    ApiOperation {
        method: "POST",
        path: "/app/v3/api/ai/tools/{toolId}/invoke",
        tag: "ai",
        operation_id: "agents.tools.invoke",
    },
    ApiOperation {
        method: "GET",
        path: "/app/v3/api/ai/assets",
        tag: "ai",
        operation_id: "agents.assets.list",
    },
];

pub const AGENT_BACKEND_API_OPERATIONS: &[ApiOperation] = &[
    ApiOperation {
        method: "GET",
        path: "/backend/v3/api/ai/agents",
        tag: "ai",
        operation_id: "agents.list",
    },
    ApiOperation {
        method: "POST",
        path: "/backend/v3/api/ai/agents",
        tag: "ai",
        operation_id: "agents.create",
    },
    ApiOperation {
        method: "GET",
        path: "/backend/v3/api/ai/agents/{agentId}",
        tag: "ai",
        operation_id: "agents.retrieve",
    },
    ApiOperation {
        method: "PATCH",
        path: "/backend/v3/api/ai/agents/{agentId}",
        tag: "ai",
        operation_id: "agents.update",
    },
    ApiOperation {
        method: "POST",
        path: "/backend/v3/api/ai/agents/{agentId}/status",
        tag: "ai",
        operation_id: "agents.status.create",
    },
    ApiOperation {
        method: "POST",
        path: "/backend/v3/api/ai/agents/{agentId}/restore",
        tag: "ai",
        operation_id: "agents.restore",
    },
    ApiOperation {
        method: "GET",
        path: "/backend/v3/api/ai/agents/{agentId}/audit_events",
        tag: "ai",
        operation_id: "agents.auditEvents.list",
    },
    ApiOperation {
        method: "GET",
        path: "/backend/v3/api/ai/agents/{agentId}/provider_bindings",
        tag: "ai",
        operation_id: "agents.providerBindings.list",
    },
    ApiOperation {
        method: "POST",
        path: "/backend/v3/api/ai/agents/{agentId}/provider_bindings",
        tag: "ai",
        operation_id: "agents.providerBindings.create",
    },
    ApiOperation {
        method: "POST",
        path: "/backend/v3/api/ai/agents/{agentId}/provider_bindings/{bindingId}/activate",
        tag: "ai",
        operation_id: "agents.providerBindings.activate",
    },
    ApiOperation {
        method: "GET",
        path: "/backend/v3/api/ai/agents/{agentId}/sessions",
        tag: "ai",
        operation_id: "agents.sessions.list",
    },
    ApiOperation {
        method: "POST",
        path: "/backend/v3/api/ai/agents/{agentId}/sessions",
        tag: "ai",
        operation_id: "agents.sessions.create",
    },
    ApiOperation {
        method: "GET",
        path: "/backend/v3/api/ai/agents/{agentId}/sessions/{sessionId}",
        tag: "ai",
        operation_id: "agents.sessions.retrieve",
    },
    ApiOperation {
        method: "POST",
        path: "/backend/v3/api/ai/agents/{agentId}/sessions/{sessionId}/close",
        tag: "ai",
        operation_id: "agents.sessions.close",
    },
    ApiOperation {
        method: "GET",
        path: "/backend/v3/api/ai/agents/{agentId}/sessions/{sessionId}/items",
        tag: "ai",
        operation_id: "agents.sessionItems.list",
    },
    ApiOperation {
        method: "GET",
        path: "/backend/v3/api/ai/agents/{agentId}/sessions/{sessionId}/items/{itemId}",
        tag: "ai",
        operation_id: "agents.sessionItems.retrieve",
    },
    ApiOperation {
        method: "GET",
        path: "/backend/v3/api/ai/agents/{agentId}/sessions/{sessionId}/turns",
        tag: "ai",
        operation_id: "agents.turns.list",
    },
    ApiOperation {
        method: "POST",
        path: "/backend/v3/api/ai/agents/{agentId}/sessions/{sessionId}/turns",
        tag: "ai",
        operation_id: "agents.turns.stream",
    },
    ApiOperation {
        method: "GET",
        path: "/backend/v3/api/ai/agents/{agentId}/sessions/{sessionId}/turns/{turnId}",
        tag: "ai",
        operation_id: "agents.turns.retrieve",
    },
    ApiOperation {
        method: "POST",
        path: "/backend/v3/api/ai/agents/{agentId}/sessions/{sessionId}/turns/{turnId}/cancel",
        tag: "ai",
        operation_id: "agents.turns.cancel",
    },
    ApiOperation {
        method: "GET",
        path: "/backend/v3/api/ai/agents/{agentId}/sessions/{sessionId}/interactions",
        tag: "ai",
        operation_id: "agents.interactions.list",
    },
    ApiOperation {
        method: "POST",
        path: "/backend/v3/api/ai/agents/{agentId}/sessions/{sessionId}/interactions",
        tag: "ai",
        operation_id: "agents.interactions.create",
    },
    ApiOperation {
        method: "GET",
        path: "/backend/v3/api/ai/agents/{agentId}/sessions/{sessionId}/interactions/{interactionId}",
        tag: "ai",
        operation_id: "agents.interactions.retrieve",
    },
    ApiOperation {
        method: "POST",
        path: "/backend/v3/api/ai/agents/{agentId}/sessions/{sessionId}/interactions/{interactionId}/claim",
        tag: "ai",
        operation_id: "agents.interactions.claim",
    },
    ApiOperation {
        method: "POST",
        path: "/backend/v3/api/ai/agents/{agentId}/sessions/{sessionId}/interactions/{interactionId}/approve",
        tag: "ai",
        operation_id: "agents.interactions.approve",
    },
    ApiOperation {
        method: "POST",
        path: "/backend/v3/api/ai/agents/{agentId}/sessions/{sessionId}/interactions/{interactionId}/answer",
        tag: "ai",
        operation_id: "agents.interactions.answer",
    },
    ApiOperation {
        method: "POST",
        path: "/backend/v3/api/ai/agents/{agentId}/sessions/{sessionId}/interactions/{interactionId}/resolve",
        tag: "ai",
        operation_id: "agents.interactions.resolve",
    },
    ApiOperation {
        method: "GET",
        path: "/backend/v3/api/ai/agents/{agentId}/sessions/{sessionId}/checkpoints",
        tag: "ai",
        operation_id: "agents.checkpoints.list",
    },
    ApiOperation {
        method: "POST",
        path: "/backend/v3/api/ai/agents/{agentId}/sessions/{sessionId}/checkpoints",
        tag: "ai",
        operation_id: "agents.checkpoints.create",
    },
    ApiOperation {
        method: "GET",
        path: "/backend/v3/api/ai/agents/{agentId}/sessions/{sessionId}/checkpoints/{checkpointId}",
        tag: "ai",
        operation_id: "agents.checkpoints.retrieve",
    },
    ApiOperation {
        method: "POST",
        path: "/backend/v3/api/ai/agents/{agentId}/sessions/{sessionId}/checkpoints/{checkpointId}/restore",
        tag: "ai",
        operation_id: "agents.checkpoints.restore",
    },
    ApiOperation {
        method: "POST",
        path: "/backend/v3/api/ai/agents/{agentId}/sessions/{sessionId}/checkpoints/{checkpointId}/invalidate",
        tag: "ai",
        operation_id: "agents.checkpoints.invalidate",
    },
    ApiOperation {
        method: "GET",
        path: "/backend/v3/api/ai/agents/{agentId}/sessions/{sessionId}/runtime_bindings",
        tag: "ai",
        operation_id: "agents.sessionRuntimeBindings.list",
    },
    ApiOperation {
        method: "POST",
        path: "/backend/v3/api/ai/agents/{agentId}/sessions/{sessionId}/runtime_bindings",
        tag: "ai",
        operation_id: "agents.sessionRuntimeBindings.create",
    },
    ApiOperation {
        method: "GET",
        path: "/backend/v3/api/ai/agents/{agentId}/sessions/{sessionId}/runtime_bindings/{runtimeBindingId}",
        tag: "ai",
        operation_id: "agents.sessionRuntimeBindings.retrieve",
    },
    ApiOperation {
        method: "PATCH",
        path: "/backend/v3/api/ai/agents/{agentId}/sessions/{sessionId}/runtime_bindings/{runtimeBindingId}",
        tag: "ai",
        operation_id: "agents.sessionRuntimeBindings.update",
    },
    ApiOperation {
        method: "POST",
        path: "/backend/v3/api/ai/agents/{agentId}/sessions/{sessionId}/runtime_bindings/{runtimeBindingId}/activate",
        tag: "ai",
        operation_id: "agents.sessionRuntimeBindings.activate",
    },
    ApiOperation {
        method: "POST",
        path: "/backend/v3/api/ai/agents/{agentId}/sessions/{sessionId}/runtime_bindings/{runtimeBindingId}/deactivate",
        tag: "ai",
        operation_id: "agents.sessionRuntimeBindings.deactivate",
    },
    ApiOperation {
        method: "GET",
        path: "/backend/v3/api/ai/agents/{agentId}/tasks",
        tag: "ai",
        operation_id: "agents.tasks.list",
    },
    ApiOperation {
        method: "POST",
        path: "/backend/v3/api/ai/agents/{agentId}/tasks",
        tag: "ai",
        operation_id: "agents.tasks.create",
    },
    ApiOperation {
        method: "GET",
        path: "/backend/v3/api/ai/agents/{agentId}/tasks/{taskId}",
        tag: "ai",
        operation_id: "agents.tasks.retrieve",
    },
    ApiOperation {
        method: "POST",
        path: "/backend/v3/api/ai/agents/{agentId}/tasks/{taskId}/cancel",
        tag: "ai",
        operation_id: "agents.tasks.cancel",
    },
    ApiOperation {
        method: "POST",
        path: "/backend/v3/api/ai/agents/{agentId}/tasks/{taskId}/execute",
        tag: "ai",
        operation_id: "agents.tasks.execute",
    },
    ApiOperation {
        method: "PUT",
        path: "/backend/v3/api/ai/agents/{agentId}/tasks/{taskId}",
        tag: "ai",
        operation_id: "agents.tasks.update",
    },
    ApiOperation {
        method: "POST",
        path: "/backend/v3/api/ai/agents/{agentId}/tasks/{taskId}/pause",
        tag: "ai",
        operation_id: "agents.tasks.pause",
    },
    ApiOperation {
        method: "POST",
        path: "/backend/v3/api/ai/agents/{agentId}/tasks/{taskId}/resume",
        tag: "ai",
        operation_id: "agents.tasks.resume",
    },
    ApiOperation {
        method: "GET",
        path: "/backend/v3/api/ai/agents/{agentId}/tasks/{taskId}/runs",
        tag: "ai",
        operation_id: "agents.taskRuns.list",
    },
    ApiOperation {
        method: "GET",
        path: "/backend/v3/api/ai/agents/{agentId}/tasks/{taskId}/runs/{runId}",
        tag: "ai",
        operation_id: "agents.taskRuns.retrieve",
    },
    ApiOperation {
        method: "POST",
        path: "/backend/v3/api/ai/agents/{agentId}/tasks/{taskId}/runs/{runId}/retry",
        tag: "ai",
        operation_id: "agents.taskRuns.retry",
    },
    ApiOperation {
        method: "POST",
        path: "/backend/v3/api/ai/agents/{agentId}/tasks/{taskId}/runs/{runId}/cancel",
        tag: "ai",
        operation_id: "agents.taskRuns.cancel",
    },
    ApiOperation {
        method: "GET",
        path: "/backend/v3/api/ai/agents/{agentId}/tasks/{taskId}/runs/{runId}/attempts",
        tag: "ai",
        operation_id: "agents.taskRunAttempts.list",
    },
    ApiOperation {
        method: "POST",
        path: "/backend/v3/api/ai/agents/{agentId}/tasks/{taskId}/runs/{runId}/reconcile",
        tag: "ai",
        operation_id: "agents.taskRuns.reconcile",
    },
    ApiOperation {
        method: "POST",
        path: "/backend/v3/api/ai/agents/{agentId}/sessions/{sessionId}/archive",
        tag: "ai",
        operation_id: "agents.sessions.archive",
    },
    ApiOperation {
        method: "GET",
        path: "/backend/v3/api/ai/agents/{agentId}/composition_slots",
        tag: "ai",
        operation_id: "agents.compositionSlots.list",
    },
    ApiOperation {
        method: "POST",
        path: "/backend/v3/api/ai/agents/{agentId}/composition_slots",
        tag: "ai",
        operation_id: "agents.compositionSlots.create",
    },
    ApiOperation {
        method: "GET",
        path: "/backend/v3/api/ai/agents/{agentId}/composition_slots/{slotId}",
        tag: "ai",
        operation_id: "agents.compositionSlots.retrieve",
    },
    ApiOperation {
        method: "PATCH",
        path: "/backend/v3/api/ai/agents/{agentId}/composition_slots/{slotId}",
        tag: "ai",
        operation_id: "agents.compositionSlots.update",
    },
    ApiOperation {
        method: "DELETE",
        path: "/backend/v3/api/ai/agents/{agentId}/composition_slots/{slotId}",
        tag: "ai",
        operation_id: "agents.compositionSlots.delete",
    },
    ApiOperation {
        method: "GET",
        path: "/backend/v3/api/ai/tools",
        tag: "ai",
        operation_id: "agents.tools.adminList",
    },
    ApiOperation {
        method: "PUT",
        path: "/backend/v3/api/ai/tools/{toolId}/configuration",
        tag: "ai",
        operation_id: "agents.tools.updateConfiguration",
    },
];

#[cfg(test)]
mod tests {
    use super::*;

    const COMPOSITION_SLOT_ID_PATTERN: &str = r"^slot\.[a-z0-9_-]+(\.[a-z0-9_-]+)*$";

    fn assert_composition_slot_id_pattern(label: &str, openapi: &str) {
        let document: serde_yaml::Value = serde_yaml::from_str(openapi)
            .unwrap_or_else(|error| panic!("{label} OpenAPI must be valid YAML: {error}"));
        let pattern = document
            .get("components")
            .and_then(|value| value.get("parameters"))
            .and_then(|value| value.get("SlotIdPath"))
            .and_then(|value| value.get("schema"))
            .and_then(|value| value.get("pattern"))
            .and_then(serde_yaml::Value::as_str);

        assert_eq!(
            pattern,
            Some(COMPOSITION_SLOT_ID_PATTERN),
            "{label} OpenAPI SlotIdPath must enforce the canonical composition slot ID pattern"
        );
    }

    fn assert_openapi_permission(
        openapi: &str,
        method: &str,
        path: &str,
        expected_permission: &str,
    ) {
        let document: serde_yaml::Value =
            serde_yaml::from_str(openapi).expect("OpenAPI authority must be valid YAML");
        let permission = document
            .get("paths")
            .and_then(|value| value.get(path))
            .and_then(|value| value.get(method))
            .and_then(|value| value.get("x-sdkwork-permission"))
            .and_then(serde_yaml::Value::as_str);

        assert_eq!(permission, Some(expected_permission));
    }

    #[test]
    fn app_project_and_agent_engine_lists_require_agents_read_permission() {
        let app_openapi = include_str!("../specs/openapi/agents-app-api.openapi.yaml");

        assert_openapi_permission(
            app_openapi,
            "get",
            "/app/v3/api/ai/projects",
            crate::infrastructure::IAM_PERMISSION_AGENTS_READ,
        );
        assert_openapi_permission(
            app_openapi,
            "get",
            "/app/v3/api/ai/agent_engines",
            crate::infrastructure::IAM_PERMISSION_AGENTS_READ,
        );
    }

    #[test]
    fn provider_binding_operations_are_registered() {
        assert_operation(
            AGENT_OPEN_API_OPERATIONS,
            "GET",
            "/agent/v3/api/ai/agents/{agentId}/provider_bindings",
            "agents.providerBindings.list",
        );
        assert_operation(
            AGENT_OPEN_API_OPERATIONS,
            "POST",
            "/agent/v3/api/ai/agents/{agentId}/provider_bindings",
            "agents.providerBindings.create",
        );
        assert_operation(
            AGENT_OPEN_API_OPERATIONS,
            "POST",
            "/agent/v3/api/ai/agents/{agentId}/provider_bindings/{bindingId}/activate",
            "agents.providerBindings.activate",
        );
        assert_operation(
            AGENT_OPEN_API_OPERATIONS,
            "POST",
            "/agent/v3/api/ai/agents/{agentId}/preview_responses",
            "agents.previewResponses.create",
        );
        assert_operation(
            AGENT_OPEN_API_OPERATIONS,
            "POST",
            "/agent/v3/api/ai/agents/{agentId}/prompt_optimizations",
            "agents.promptOptimizations.create",
        );

        assert_operation(
            AGENT_APP_API_OPERATIONS,
            "GET",
            "/app/v3/api/ai/agents/{agentId}/provider_bindings",
            "agents.providerBindings.list",
        );
        assert_operation(
            AGENT_APP_API_OPERATIONS,
            "POST",
            "/app/v3/api/ai/agents/{agentId}/provider_bindings",
            "agents.providerBindings.create",
        );
        assert_operation(
            AGENT_APP_API_OPERATIONS,
            "POST",
            "/app/v3/api/ai/agents/{agentId}/provider_bindings/{bindingId}/activate",
            "agents.providerBindings.activate",
        );
        assert_operation(
            AGENT_APP_API_OPERATIONS,
            "POST",
            "/app/v3/api/ai/agents/{agentId}/preview_responses",
            "agents.previewResponses.create",
        );
        assert_operation(
            AGENT_APP_API_OPERATIONS,
            "POST",
            "/app/v3/api/ai/agents/{agentId}/prompt_optimizations",
            "agents.promptOptimizations.create",
        );

        assert_operation(
            AGENT_BACKEND_API_OPERATIONS,
            "GET",
            "/backend/v3/api/ai/agents/{agentId}/provider_bindings",
            "agents.providerBindings.list",
        );
        assert_operation(
            AGENT_BACKEND_API_OPERATIONS,
            "POST",
            "/backend/v3/api/ai/agents/{agentId}/provider_bindings",
            "agents.providerBindings.create",
        );
        assert_operation(
            AGENT_BACKEND_API_OPERATIONS,
            "POST",
            "/backend/v3/api/ai/agents/{agentId}/provider_bindings/{bindingId}/activate",
            "agents.providerBindings.activate",
        );
    }

    #[test]
    fn composition_slot_operations_are_registered_for_all_api_boundaries() {
        for (operations, prefix) in [
            (AGENT_OPEN_API_OPERATIONS, AGENT_OPEN_API_PREFIX),
            (AGENT_APP_API_OPERATIONS, AGENT_APP_API_PREFIX),
            (AGENT_BACKEND_API_OPERATIONS, AGENT_BACKEND_API_PREFIX),
        ] {
            for (method, path_suffix, operation_id) in [
                (
                    "GET",
                    "/ai/agents/{agentId}/composition_slots",
                    "agents.compositionSlots.list",
                ),
                (
                    "POST",
                    "/ai/agents/{agentId}/composition_slots",
                    "agents.compositionSlots.create",
                ),
                (
                    "GET",
                    "/ai/agents/{agentId}/composition_slots/{slotId}",
                    "agents.compositionSlots.retrieve",
                ),
                (
                    "PATCH",
                    "/ai/agents/{agentId}/composition_slots/{slotId}",
                    "agents.compositionSlots.update",
                ),
                (
                    "DELETE",
                    "/ai/agents/{agentId}/composition_slots/{slotId}",
                    "agents.compositionSlots.delete",
                ),
            ] {
                assert_operation(
                    operations,
                    method,
                    format!("{prefix}{path_suffix}").as_str(),
                    operation_id,
                );
            }
        }
    }

    #[test]
    fn openapi_specs_expose_provider_binding_and_composition_contracts() {
        let open_openapi = include_str!("../specs/openapi/agents-open-api.openapi.yaml");
        let app_openapi = include_str!("../specs/openapi/agents-app-api.openapi.yaml");
        let backend_openapi = include_str!("../specs/openapi/agents-backend-api.openapi.yaml");

        for (label, openapi, prefix) in [
            ("open", open_openapi, "/agent/v3/api"),
            ("app", app_openapi, "/app/v3/api"),
            ("backend", backend_openapi, "/backend/v3/api"),
        ] {
            assert_composition_slot_id_pattern(label, openapi);

            for required in [
                format!("{prefix}/ai/agents/{{agentId}}/provider_bindings:"),
                format!("{prefix}/ai/agents/{{agentId}}/composition_slots:"),
                format!("{prefix}/ai/agents/{{agentId}}/composition_slots/{{slotId}}:"),
                "operationId: agents.providerBindings.list".to_string(),
                "operationId: agents.compositionSlots.list".to_string(),
                "operationId: agents.compositionSlots.create".to_string(),
                "operationId: agents.compositionSlots.retrieve".to_string(),
                "operationId: agents.compositionSlots.update".to_string(),
                "operationId: agents.compositionSlots.delete".to_string(),
                "AgentCompositionSlotKind:".to_string(),
                "AgentCompositionSlotRecord:".to_string(),
                "AgentCompositionTargetModule:".to_string(),
                "CreateAgentCompositionSlotRequest:".to_string(),
                "UpdateAgentCompositionSlotRequest:".to_string(),
                "AgentCompositionSlotResponse:".to_string(),
                "AgentCompositionSlotListResponse:".to_string(),
            ] {
                assert!(
                    openapi.contains(required.as_str()),
                    "{label} OpenAPI must contain {required}"
                );
            }

            let composition_permissions = if label == "app" {
                [
                    "x-sdkwork-permission: ai.agents.read",
                    "x-sdkwork-permission: ai.agents.manage",
                ]
            } else {
                [
                    "x-sdkwork-permission: agent.business.composition_slot.list",
                    "x-sdkwork-permission: agent.business.composition_slot.create",
                ]
            };
            for required in composition_permissions {
                assert!(
                    openapi.contains(required),
                    "{label} OpenAPI must contain {required}"
                );
            }

            for required in [
                format!("{prefix}/ai/agents/{{agentId}}/sessions:"),
                format!("{prefix}/ai/agents/{{agentId}}/sessions/{{sessionId}}:"),
                format!("{prefix}/ai/agents/{{agentId}}/sessions/{{sessionId}}/items:"),
                format!("{prefix}/ai/agents/{{agentId}}/sessions/{{sessionId}}/items/{{itemId}}:"),
                format!("{prefix}/ai/agents/{{agentId}}/sessions/{{sessionId}}/turns:"),
                format!("{prefix}/ai/agents/{{agentId}}/sessions/{{sessionId}}/interactions:"),
                format!("{prefix}/ai/agents/{{agentId}}/sessions/{{sessionId}}/checkpoints:"),
                format!("{prefix}/ai/agents/{{agentId}}/sessions/{{sessionId}}/runtime_bindings:"),
                format!("{prefix}/ai/agents/{{agentId}}/sessions/{{sessionId}}/close:"),
                "operationId: agents.sessions.list".to_string(),
                "operationId: agents.sessions.create".to_string(),
                "operationId: agents.sessionItems.list".to_string(),
                "operationId: agents.turns.stream".to_string(),
                "operationId: agents.interactions.claim".to_string(),
                "operationId: agents.checkpoints.restore".to_string(),
                "operationId: agents.sessionRuntimeBindings.activate".to_string(),
                "AgentSessionItemRecord:".to_string(),
                "AgentTurnRecord:".to_string(),
                "AgentInteractionRecord:".to_string(),
                "AgentSessionCheckpointRecord:".to_string(),
                "AgentSessionRuntimeBindingRecord:".to_string(),
                "AgentTurnExecutionResponse:".to_string(),
                "CreateAgentTurnRequest:".to_string(),
                "CloseAgentSessionRequest:".to_string(),
            ] {
                assert!(
                    openapi.contains(required.as_str()),
                    "{label} OpenAPI must contain {required}"
                );
            }

            if label == "app" {
                for required in [
                    "CreateAgentSessionRequest:".to_string(),
                    format!("{prefix}/ai/agent_engines:"),
                    format!("{prefix}/ai/mcp_servers:"),
                    "operationId: agents.agentEngines.list".to_string(),
                    "operationId: agents.mcpServers.list".to_string(),
                    "AgentEngineCatalogListResponse:".to_string(),
                    "McpServerMarketplaceListResponse:".to_string(),
                    format!("{prefix}/ai/agents/{{agentId}}/sessions/user_states:"),
                    format!("{prefix}/ai/agents/{{agentId}}/sessions/{{sessionId}}/user_state:"),
                    "operationId: agents.sessionUserStates.list".to_string(),
                    "operationId: agents.sessionUserStates.retrieve".to_string(),
                    "operationId: agents.sessionUserStates.update".to_string(),
                    "AgentResourceUserStateRecord:".to_string(),
                    "UpdateAgentSessionUserStateRequest:".to_string(),
                    format!(
                        "{prefix}/ai/agents/{{agentId}}/sessions/{{sessionId}}/item_feedback:"
                    ),
                    format!(
                        "{prefix}/ai/agents/{{agentId}}/sessions/{{sessionId}}/items/{{itemId}}/feedback:"
                    ),
                    "operationId: agents.itemFeedback.list".to_string(),
                    "operationId: agents.itemFeedback.update".to_string(),
                    "AgentItemFeedbackRecord:".to_string(),
                    "UpdateAgentItemFeedbackRequest:".to_string(),
                ] {
                    assert!(
                        openapi.contains(required.as_str()),
                        "{label} OpenAPI must contain {required}"
                    );
                }
            }

            if label == "open" || label == "backend" {
                for required in [
                    "CreateAgentTurnRequest:".to_string(),
                    "CreateAgentSessionRequest:".to_string(),
                    "CloseAgentSessionRequest:".to_string(),
                ] {
                    assert!(
                        openapi.contains(required.as_str()),
                        "{label} OpenAPI must contain {required}"
                    );
                }
            }

            if label == "backend" {
                for required in [
                    format!("{prefix}/ai/agents/{{agentId}}/sessions/{{sessionId}}/archive:"),
                    "operationId: agents.sessions.archive".to_string(),
                    "ArchiveAgentSessionRequest:".to_string(),
                ] {
                    assert!(
                        openapi.contains(required.as_str()),
                        "{label} OpenAPI must contain {required}"
                    );
                }
            }

            for forbidden in [
                "/ai/knowledge_bases",
                "/ai/memory_stores",
                "operationId: knowledgeBases.",
                "operationId: memoryStores.",
                "/messages",
                "{messageId}",
                "operationId: agents.messages.",
                "operationId: agents.chatTurns.",
                "AgentMessage",
                "AgentChatTurn",
                "SendAgentChatMessage",
            ] {
                assert!(
                    !openapi.contains(forbidden),
                    "{label} OpenAPI must not contain legacy inline knowledge/memory surface {forbidden}"
                );
            }

            if label == "open" {
                assert!(
                    !openapi.contains("/agent/v3/api/ai/agents/{agentId}/restore:"),
                    "open OpenAPI must not expose agents.restore (app/backend only)"
                );
            }

            if label != "backend" {
                for required in [
                    format!("{prefix}/ai/agents/{{agentId}}/preview_responses:"),
                    format!("{prefix}/ai/agents/{{agentId}}/prompt_optimizations:"),
                    "operationId: agents.previewResponses.create".to_string(),
                    "operationId: agents.promptOptimizations.create".to_string(),
                ] {
                    assert!(
                        openapi.contains(required.as_str()),
                        "{label} OpenAPI must contain {required}"
                    );
                }
            }
        }

        for required in [
            "composition_slot_created",
            "composition_slot_updated",
            "composition_slot_deleted",
        ] {
            assert!(
                backend_openapi.contains(required),
                "backend OpenAPI audit action enum must contain {required}"
            );
        }
    }

    fn assert_operation(operations: &[ApiOperation], method: &str, path: &str, operation_id: &str) {
        assert!(
            operations.iter().any(|operation| {
                operation.method == method
                    && operation.path == path
                    && operation.operation_id == operation_id
            }),
            "{method} {path} must be registered as {operation_id}"
        );
    }

    fn count_openapi_operation_ids(openapi: &str) -> usize {
        // Count only real OpenAPI operations (`operationId: <value>` on one line).
        // Schema properties such as `SdkWorkAsyncData.operationId` use
        // `operationId:` with the type on the next line, so they must be excluded
        // to avoid false positives from the standard envelope schemas.
        openapi
            .lines()
            .filter(|line| {
                let trimmed = line.trim_start();
                let rest = trimmed.strip_prefix("operationId:").unwrap_or("");
                !rest.trim().is_empty()
            })
            .count()
    }

    #[test]
    fn surface_operation_counts_match_canonical_inventory() {
        let open_openapi = include_str!("../specs/openapi/agents-open-api.openapi.yaml");
        let app_openapi = include_str!("../specs/openapi/agents-app-api.openapi.yaml");
        let backend_openapi = include_str!("../specs/openapi/agents-backend-api.openapi.yaml");

        assert_eq!(AGENT_OPEN_API_OPERATIONS.len(), 56);
        assert_eq!(AGENT_APP_API_OPERATIONS.len(), 126);
        assert_eq!(AGENT_BACKEND_API_OPERATIONS.len(), 60);

        assert_eq!(
            AGENT_OPEN_API_OPERATIONS.len(),
            count_openapi_operation_ids(open_openapi),
            "open operation registry must match OpenAPI authority"
        );
        assert_eq!(
            AGENT_APP_API_OPERATIONS.len(),
            count_openapi_operation_ids(app_openapi),
            "app operation registry must match OpenAPI authority"
        );
        assert_eq!(
            AGENT_BACKEND_API_OPERATIONS.len(),
            count_openapi_operation_ids(backend_openapi),
            "backend operation registry must match OpenAPI authority"
        );
    }

    #[test]
    fn open_operation_registry_must_not_include_unimplemented_surface_drift() {
        for forbidden in [
            "agents.providerBindings.retrieve",
            "agents.providerBindings.deactivate",
            "agents.providerBindings.delete",
            "agents.agentEngines.health",
            "agents.mcpServers.list",
        ] {
            assert!(
                !AGENT_OPEN_API_OPERATIONS
                    .iter()
                    .any(|operation| operation.operation_id == forbidden),
                "open registry must not include unimplemented {forbidden}"
            );
        }
    }

    #[test]
    fn operation_registry_paths_must_not_include_non_ga_scope_routes() {
        let forbidden_path_markers = [
            "/provider_bindings/{bindingId}/deactivate",
            "/agent_engines/{engineKey}/health",
        ];
        for operations in [AGENT_OPEN_API_OPERATIONS] {
            for operation in operations {
                for marker in forbidden_path_markers {
                    assert!(
                        !operation.path.contains(marker),
                        "{} path must not include {marker}",
                        operation.operation_id
                    );
                }
            }
        }

        for forbidden in [
            "agents.providerBindings.retrieve",
            "agents.providerBindings.deactivate",
            "agents.providerBindings.delete",
            "agents.agentEngines.health",
        ] {
            assert!(
                !AGENT_APP_API_OPERATIONS
                    .iter()
                    .any(|operation| operation.operation_id == forbidden),
                "app registry must not include non-GA scope operation {forbidden}"
            );
        }

        for forbidden in [
            "agents.providerBindings.retrieve",
            "agents.providerBindings.deactivate",
            "agents.providerBindings.delete",
            "agents.sessions.update",
            "agents.agentEngines.list",
            "agents.agentEngines.health",
            "agents.mcpServers.list",
        ] {
            assert!(
                !AGENT_BACKEND_API_OPERATIONS
                    .iter()
                    .any(|operation| operation.operation_id == forbidden),
                "backend registry must not include non-GA scope operation {forbidden}"
            );
        }
    }
}

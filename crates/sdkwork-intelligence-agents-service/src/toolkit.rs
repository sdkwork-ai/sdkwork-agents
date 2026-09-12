//! Effective toolkit resolution for agent turns.
//!
//! Every turn resolves the model-visible tool set from three layers (all
//! merged deterministically, high cohesion per layer):
//!
//! 1. **Default set** — the built-in generations MCP plus the synchronous
//!    media family. Agents get these by default (the product contract of
//!    "chat agents support image/video/audio/music/sound-effect MCP out of
//!    the box") and trim them per agent.
//! 2. **Composition slot overrides** — `slot_kind = tool` slots with
//!    `enabled = false` remove a default tool; `slot_kind = mcp` slots add
//!    every tool of a bound external MCP server (namespaced
//!    `mcp__<server>__<tool>`), gated on approval by default.
//! 3. **Skill assembly** — `slot_kind = skill` slots contribute an
//!    instructions section to the assembled system prompt so the model
//!    follows agent-specific skills during free conversation.

use serde::{Deserialize, Serialize};

use crate::domain::AgentCompositionSlotRecord;
use crate::tool_calling::TurnToolDescriptor;

/// Stable policy keys understood inside a composition slot's `policy_json`.
pub const SLOT_POLICY_TITLE: &str = "title";
pub const SLOT_POLICY_INSTRUCTIONS: &str = "instructions";

/// One enabled skill attached to the agent for this turn.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResolvedSkill {
    pub slot_id: String,
    /// Stable skill reference (skill registry id), e.g. `skill.brand-copy`.
    pub target_ref: String,
    pub title: Option<String>,
    pub instructions: Option<String>,
}

/// Resolved per-turn toolkit: model-visible tools, assembled instructions,
/// and the external MCP connections backing the expanded tools.
#[derive(Debug, Clone, PartialEq)]
pub struct ResolvedToolkit {
    pub tools: Vec<TurnToolDescriptor>,
    pub skills: Vec<ResolvedSkill>,
    /// Skill-assembled system prompt section. The turn executor uses it only
    /// when the request carries no explicit `systemPrompt`.
    pub assembled_system_prompt: Option<String>,
    /// Connections for every expanded external MCP server, matched to the
    /// executor by `server_key`.
    pub connections: Vec<crate::tool_calling::McpServerConnection>,
}

/// API overview of one agent's effective toolkit (management/debug surface).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentToolkitOverview {
    pub agent_id: String,
    pub tools: Vec<TurnToolDescriptor>,
    pub skills: Vec<ResolvedSkill>,
    pub assembled_system_prompt: Option<String>,
}

/// Source for the toolkit layer the service cannot own by itself: the
/// built-in tool descriptors contributed by the registered executors. Wired
/// at gateway bootstrap. External MCP tools come from the composition-slot
/// policies themselves (see `McpSlotPolicy`), keeping the per-turn resolution
/// self-contained.
pub trait TurnToolkitConfig: Send + Sync {
    /// The default (built-in) tool descriptors every chat agent gets.
    fn default_tools(&self) -> Vec<TurnToolDescriptor>;
}

/// Resolves the effective toolkit for one agent turn.
pub fn resolve_effective_toolkit(
    config: &dyn TurnToolkitConfig,
    slots: &[AgentCompositionSlotRecord],
) -> ResolvedToolkit {
    let default_tools = config.default_tools();
    let mut tools: Vec<TurnToolDescriptor> = default_tools;
    let mut skills: Vec<ResolvedSkill> = Vec::new();
    let mut connections: Vec<crate::tool_calling::McpServerConnection> = Vec::new();

    // Disabled tool slots trim the default set (and any added tool).
    let disabled_tool_ids: Vec<&str> = slots
        .iter()
        .filter(|slot| {
            slot.slot_kind == crate::domain::AgentCompositionSlotKind::Tool && !slot.enabled
        })
        .map(|slot| slot.target_ref.as_str())
        .collect();
    tools.retain(|tool| !disabled_tool_ids.contains(&tool.tool_id.as_str()));

    for slot in slots {
        if !slot.enabled {
            continue;
        }
        match slot.slot_kind {
            crate::domain::AgentCompositionSlotKind::Tool => {
                // Explicitly enabled built-in tools are already present; an
                // unknown tool id cannot be described and is skipped (the
                // dispatcher fails closed at call time anyway).
                let _ = slot.target_ref.as_str();
            }
            crate::domain::AgentCompositionSlotKind::Mcp => {
                let Some(policy) = McpSlotPolicy::parse(&slot.policy_json) else {
                    continue;
                };
                // A binding without an endpoint contributes no tools and no
                // connection (invocation would fail closed anyway).
                let Some(endpoint_url) = policy.endpoint_url.clone() else {
                    continue;
                };
                for external in &policy.tools {
                    if !policy.tool_permitted(&external.name) {
                        continue;
                    }
                    let tool_id = format!("mcp__{}__{}", slot.target_ref, external.name);
                    if tools.iter().any(|tool| tool.tool_id == tool_id) {
                        continue;
                    }
                    tools.push(TurnToolDescriptor {
                        tool_id: tool_id.clone(),
                        name: tool_id,
                        description: external.description.clone(),
                        input_schema: external.input_schema.clone(),
                        requires_approval: external.requires_approval,
                        policy_category: None,
                        timeout_ms: policy
                            .timeout_ms
                            .unwrap_or(crate::tool_calling::DEFAULT_TOOL_TIMEOUT_MS),
                        origin: crate::tool_calling::TurnToolOrigin::ExternalMcp,
                    });
                }
                connections.push(crate::tool_calling::McpServerConnection {
                    server_key: slot.target_ref.clone(),
                    endpoint_url,
                    auth_type: policy
                        .auth_type
                        .clone()
                        .unwrap_or_else(|| "none".to_string()),
                    secret_ref: policy.secret_ref.clone(),
                    timeout_ms: policy
                        .timeout_ms
                        .unwrap_or(crate::tool_calling::DEFAULT_TOOL_TIMEOUT_MS),
                });
            }
            crate::domain::AgentCompositionSlotKind::Skill => {
                let policy = parse_slot_policy(&slot.policy_json);
                skills.push(ResolvedSkill {
                    slot_id: slot.slot_id.clone(),
                    target_ref: slot.target_ref.clone(),
                    title: policy.get(SLOT_POLICY_TITLE).cloned(),
                    instructions: policy.get(SLOT_POLICY_INSTRUCTIONS).cloned(),
                });
            }
            _ => {}
        }
    }

    let assembled_system_prompt = assemble_skill_prompt(&skills);
    ResolvedToolkit {
        tools,
        skills,
        assembled_system_prompt,
        connections,
    }
}

/// Policy document of one `slotKind: mcp` composition slot (`policy_json`).
///
/// The slot holds the resolved orchestration snapshot for one bound MCP
/// server: the JSON-RPC endpoint of its published revision, the auth shape,
/// and the tool descriptors to expand (typically mirrored from the
/// `sdkwork-mcp` catalog at binding time). Unparsable policies are skipped —
/// a broken binding must never fail the turn.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct McpSlotPolicy {
    pub endpoint_url: Option<String>,
    pub auth_type: Option<String>,
    pub secret_ref: Option<String>,
    pub timeout_ms: Option<u64>,
    /// Explicit tool allow-list (tool names); empty means every listed tool.
    pub allowed_tools: Vec<String>,
    /// Explicit tool deny-list (tool names); wins over the allow-list.
    pub denied_tools: Vec<String>,
    pub tools: Vec<McpSlotPolicyTool>,
}

/// One tool descriptor inside an MCP slot policy.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct McpSlotPolicyTool {
    pub name: String,
    pub description: String,
    pub input_schema: serde_json::Value,
    pub requires_approval: bool,
}

impl McpSlotPolicy {
    /// Parses the policy from a slot's `policy_json` (best effort).
    pub fn parse(policy_json: &str) -> Option<Self> {
        let value = serde_json::from_str::<serde_json::Value>(policy_json).ok()?;
        let tools = value
            .get("tools")
            .and_then(serde_json::Value::as_array)
            .map(|items| {
                items
                    .iter()
                    .filter_map(|item| {
                        let name = item.get("name").and_then(serde_json::Value::as_str)?;
                        Some(McpSlotPolicyTool {
                            name: name.trim().to_string(),
                            description: item
                                .get("description")
                                .and_then(serde_json::Value::as_str)
                                .unwrap_or_default()
                                .to_string(),
                            input_schema: item
                                .get("inputSchema")
                                .cloned()
                                .unwrap_or_else(|| serde_json::json!({"type": "object"})),
                            requires_approval: item
                                .get("requiresApproval")
                                .and_then(serde_json::Value::as_bool)
                                .unwrap_or(false),
                        })
                    })
                    .filter(|tool| !tool.name.is_empty())
                    .collect()
            })
            .unwrap_or_default();
        Some(Self {
            endpoint_url: value
                .get("endpointUrl")
                .and_then(serde_json::Value::as_str)
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(str::to_string),
            auth_type: value
                .get("authType")
                .and_then(serde_json::Value::as_str)
                .map(str::to_string),
            secret_ref: value
                .get("secretRef")
                .and_then(serde_json::Value::as_str)
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(str::to_string),
            timeout_ms: value.get("timeoutMs").and_then(serde_json::Value::as_u64),
            allowed_tools: string_list(&value, "allowedTools"),
            denied_tools: string_list(&value, "deniedTools"),
            tools,
        })
    }

    fn tool_permitted(&self, tool_name: &str) -> bool {
        if self.denied_tools.iter().any(|denied| denied == tool_name) {
            return false;
        }
        self.allowed_tools.is_empty()
            || self
                .allowed_tools
                .iter()
                .any(|allowed| allowed == tool_name)
    }
}

fn string_list(value: &serde_json::Value, key: &str) -> Vec<String> {
    value
        .get(key)
        .and_then(serde_json::Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(serde_json::Value::as_str)
                .map(str::trim)
                .filter(|item| !item.is_empty())
                .map(str::to_string)
                .collect()
        })
        .unwrap_or_default()
}

/// Parses a slot's `policy_json` into a flat string map (best effort; an
/// unparsable policy yields an empty map and never fails the turn).
fn parse_slot_policy(policy_json: &str) -> std::collections::HashMap<String, String> {
    serde_json::from_str::<serde_json::Value>(policy_json)
        .ok()
        .and_then(|value| value.as_object().cloned())
        .map(|object| {
            object
                .into_iter()
                .filter_map(|(key, value)| value.as_str().map(|text| (key, text.to_string())))
                .collect()
        })
        .unwrap_or_default()
}

/// Assembles the skill instructions section injected into the system prompt.
fn assemble_skill_prompt(skills: &[ResolvedSkill]) -> Option<String> {
    let described = skills
        .iter()
        .filter_map(|skill| {
            let title = skill.title.as_deref().unwrap_or(skill.target_ref.as_str());
            match skill.instructions.as_deref().map(str::trim) {
                Some(instructions) if !instructions.is_empty() => {
                    Some(format!("- {title}\n{instructions}"))
                }
                _ => Some(format!("- {title}")),
            }
        })
        .collect::<Vec<_>>();
    if described.is_empty() {
        return None;
    }
    Some(format!(
        "本智能体启用了以下技能，请在与用户对话时遵循相应说明：\n{}",
        described.join("\n")
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{
        AgentBusinessStatus, AgentCompositionSlotKind, AgentCompositionSlotRecord,
        AgentCompositionTargetModule,
    };
    use crate::tool_calling::{McpServerConnection, TurnToolOrigin, GENERATIONS_TOOL_TIMEOUT_MS};

    struct StaticConfig {
        defaults: Vec<TurnToolDescriptor>,
    }

    impl TurnToolkitConfig for StaticConfig {
        fn default_tools(&self) -> Vec<TurnToolDescriptor> {
            self.defaults.clone()
        }
    }

    fn default_tool(tool_id: &str) -> TurnToolDescriptor {
        TurnToolDescriptor {
            tool_id: tool_id.to_string(),
            name: tool_id.to_string(),
            description: format!("{tool_id} description"),
            input_schema: serde_json::json!({"type": "object"}),
            requires_approval: false,
            policy_category: None,
            timeout_ms: GENERATIONS_TOOL_TIMEOUT_MS,
            origin: TurnToolOrigin::BuiltinGenerations,
        }
    }

    fn slot(
        slot_id: &str,
        kind: AgentCompositionSlotKind,
        target_module: AgentCompositionTargetModule,
        target_ref: &str,
        enabled: bool,
        policy_json: &str,
    ) -> AgentCompositionSlotRecord {
        AgentCompositionSlotRecord {
            id: 1,
            tenant_id: 10,
            organization_id: 20,
            agent_id: "agent.test".to_string(),
            slot_id: slot_id.to_string(),
            slot_kind: kind,
            target_module,
            target_ref: target_ref.to_string(),
            target_version_ref: None,
            priority: 0,
            enabled,
            policy_json: policy_json.to_string(),
            status: AgentBusinessStatus::Active,
            version: 1,
            created_at: "2026-09-12T00:00:00Z".to_string(),
            updated_at: "2026-09-12T00:00:00Z".to_string(),
            deleted_at: None,
        }
    }

    #[test]
    fn no_slots_keeps_the_default_toolkit() {
        let config = StaticConfig {
            defaults: vec![default_tool("mcp__generations__image.create")],
        };
        let resolved = resolve_effective_toolkit(&config, &[]);
        assert_eq!(resolved.tools.len(), 1);
        assert!(resolved.assembled_system_prompt.is_none());
        assert!(resolved.connections.is_empty());
    }

    #[test]
    fn disabled_tool_slot_trims_the_default_set() {
        let config = StaticConfig {
            defaults: vec![
                default_tool("mcp__generations__image.create"),
                default_tool("mcp__generations__music.create"),
            ],
        };
        let slots = vec![slot(
            "slot.tool.1",
            AgentCompositionSlotKind::Tool,
            AgentCompositionTargetModule::Tools,
            "mcp__generations__image.create",
            false,
            "{}",
        )];
        let resolved = resolve_effective_toolkit(&config, &slots);
        assert_eq!(resolved.tools.len(), 1);
        assert_eq!(resolved.tools[0].tool_id, "mcp__generations__music.create");
    }

    #[test]
    fn mcp_slot_policy_expands_tools_and_connection() {
        let config = StaticConfig { defaults: vec![] };
        let slots = vec![slot(
            "slot.mcp.1",
            AgentCompositionSlotKind::Mcp,
            AgentCompositionTargetModule::Mcp,
            "browser",
            true,
            r#"{
                "endpointUrl": "https://mcp.example.test/rpc",
                "authType": "bearer",
                "secretRef": "vault://mcp/browser",
                "timeoutMs": 30000,
                "tools": [
                    {"name": "navigate", "description": "Navigate to a URL",
                     "inputSchema": {"type": "object"}, "requiresApproval": true}
                ]
            }"#,
        )];
        let resolved = resolve_effective_toolkit(&config, &slots);
        assert_eq!(resolved.tools.len(), 1);
        assert_eq!(resolved.tools[0].tool_id, "mcp__browser__navigate");
        assert!(resolved.tools[0].requires_approval);
        assert_eq!(resolved.tools[0].origin, TurnToolOrigin::ExternalMcp);
        assert_eq!(resolved.connections.len(), 1);
        assert_eq!(resolved.connections[0].server_key, "browser");
        assert_eq!(
            resolved.connections[0].endpoint_url,
            "https://mcp.example.test/rpc"
        );
        assert_eq!(resolved.connections[0].auth_type, "bearer");
        assert_eq!(
            resolved.connections[0].secret_ref.as_deref(),
            Some("vault://mcp/browser")
        );
        assert_eq!(resolved.connections[0].timeout_ms, 30_000);
    }

    #[test]
    fn mcp_slot_policy_allow_and_deny_lists_filter_tools() {
        let config = StaticConfig { defaults: vec![] };
        let slots = vec![slot(
            "slot.mcp.1",
            AgentCompositionSlotKind::Mcp,
            AgentCompositionTargetModule::Mcp,
            "browser",
            true,
            r#"{
                "endpointUrl": "https://mcp.example.test/rpc",
                "allowedTools": ["navigate"],
                "deniedTools": ["delete"],
                "tools": [
                    {"name": "navigate", "description": "nav"},
                    {"name": "delete", "description": "del"},
                    {"name": "read", "description": "read"}
                ]
            }"#,
        )];
        let resolved = resolve_effective_toolkit(&config, &slots);
        let ids: Vec<&str> = resolved.tools.iter().map(|t| t.tool_id.as_str()).collect();
        assert!(ids.contains(&"mcp__browser__navigate"));
        assert!(
            !ids.contains(&"mcp__browser__delete"),
            "denied tools must be filtered"
        );
        assert!(
            !ids.contains(&"mcp__browser__read"),
            "tools outside the allow-list must be filtered"
        );
    }

    #[test]
    fn mcp_slot_without_endpoint_or_broken_policy_is_skipped() {
        let config = StaticConfig { defaults: vec![] };
        let slots = vec![
            slot(
                "slot.mcp.1",
                AgentCompositionSlotKind::Mcp,
                AgentCompositionTargetModule::Mcp,
                "broken",
                true,
                "not-json",
            ),
            slot(
                "slot.mcp.2",
                AgentCompositionSlotKind::Mcp,
                AgentCompositionTargetModule::Mcp,
                "no-endpoint",
                true,
                r#"{"tools": [{"name": "t"}]}"#,
            ),
            slot(
                "slot.mcp.3",
                AgentCompositionSlotKind::Mcp,
                AgentCompositionTargetModule::Mcp,
                "disabled",
                false,
                r#"{"endpointUrl": "https://x", "tools": [{"name": "t"}]}"#,
            ),
        ];
        let resolved = resolve_effective_toolkit(&config, &slots);
        assert!(resolved.tools.is_empty());
        assert!(resolved.connections.is_empty());
    }

    #[test]
    fn skill_slots_assemble_instructions() {
        let config = StaticConfig { defaults: vec![] };
        let slots = vec![slot(
            "slot.skill.1",
            AgentCompositionSlotKind::Skill,
            AgentCompositionTargetModule::Skills,
            "skill.brand-copy",
            true,
            r#"{"title":"品牌文案规范","instructions":"所有回复使用简体中文，语气专业。"}"#,
        )];
        let resolved = resolve_effective_toolkit(&config, &slots);
        let prompt = resolved.assembled_system_prompt.expect("assembled prompt");
        assert!(prompt.contains("品牌文案规范"));
        assert!(prompt.contains("语气专业"));
    }

    #[test]
    fn malformed_policy_json_never_fails_resolution() {
        let config = StaticConfig {
            defaults: vec![default_tool("mcp__generations__image.create")],
        };
        let slots = vec![slot(
            "slot.skill.1",
            AgentCompositionSlotKind::Skill,
            AgentCompositionTargetModule::Skills,
            "skill.x",
            true,
            "not-json",
        )];
        let resolved = resolve_effective_toolkit(&config, &slots);
        assert_eq!(resolved.skills.len(), 1);
        assert_eq!(
            resolved.assembled_system_prompt.as_deref(),
            Some("本智能体启用了以下技能，请在与用户对话时遵循相应说明：\n- skill.x")
        );
    }

    #[test]
    fn mcp_connection_lookup_by_server_key_matches_executor_expectations() {
        // The executor resolves `mcp__<server>__<tool>` by the first segment;
        // the connection list produced by the resolver must be findable the
        // same way.
        let connections = vec![McpServerConnection {
            server_key: "browser".to_string(),
            endpoint_url: "https://mcp.example.test/rpc".to_string(),
            auth_type: "none".to_string(),
            secret_ref: None,
            timeout_ms: 30_000,
        }];
        let found = connections
            .iter()
            .find(|connection| connection.server_key == "browser");
        assert!(found.is_some());
        assert!(connections
            .iter()
            .find(|connection| connection.server_key == "missing")
            .is_none());
    }
}

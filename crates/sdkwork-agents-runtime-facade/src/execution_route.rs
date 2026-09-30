//! Execution-route model for agent conversations (`REQ-2026-0730` intent plane).
//!
//! A conversation turn runs either **in this process** (the agent-engine
//! facade slots execute locally) or **inside an SDKWork Sandbox session**
//! (lifecycle-coordinated through the kernel `SandboxedExecutionCoordinator`).
//! The route is an *intent*: it names where the conversation runtime should
//! execute, never which sandbox instance, node, lease, or fencing token serves
//! it — those stay kernel/sandbox-owned (`agent-execution-placement-orchestration.contract.json`
//! `forbiddenClientFields`).
//!
//! Resolution is a three-level chain, first match wins:
//!
//! 1. per-request override (the caller-supplied `executionRoute` parameter),
//! 2. deployment default (composition-time configuration),
//! 3. built-in default ([`AgentConversationExecutionRoute::InProcess`]).

use sdkwork_utils_rust::string::is_blank;

/// Route code accepted by request parameters and deployment configuration.
pub const EXECUTION_ROUTE_IN_PROCESS: &str = "in_process";
/// Route code selecting sandboxed conversation execution.
pub const EXECUTION_ROUTE_SANDBOX: &str = "sandbox";

/// Where an agent conversation turn executes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AgentConversationExecutionRoute {
    /// Execute in the current process through the agent-engine facade.
    InProcess,
    /// Execute inside an SDKWork Sandbox session lifecycle.
    Sandbox,
}

impl AgentConversationExecutionRoute {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::InProcess => EXECUTION_ROUTE_IN_PROCESS,
            Self::Sandbox => EXECUTION_ROUTE_SANDBOX,
        }
    }

    /// Parses the canonical route code; blank input yields `None` so callers
    /// can fall through to the next resolution level.
    pub fn parse(code: Option<&str>) -> Option<Self> {
        match code.map(str::trim).filter(|value| !is_blank(Some(value)))? {
            EXECUTION_ROUTE_IN_PROCESS => Some(Self::InProcess),
            EXECUTION_ROUTE_SANDBOX => Some(Self::Sandbox),
            _ => None,
        }
    }
}

/// Which resolution level produced the effective route.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AgentConversationExecutionRouteSource {
    /// The per-request `executionRoute` parameter decided the route.
    RequestOverride,
    /// The deployment default decided the route.
    DeploymentDefault,
    /// No override and no deployment default: the built-in default applies.
    BuiltInDefault,
}

impl AgentConversationExecutionRouteSource {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::RequestOverride => "request_override",
            Self::DeploymentDefault => "deployment_default",
            Self::BuiltInDefault => "built_in_default",
        }
    }
}

/// The effective execution route plus the resolution level that produced it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AgentConversationExecutionDecision {
    pub route: AgentConversationExecutionRoute,
    pub source: AgentConversationExecutionRouteSource,
}

impl AgentConversationExecutionDecision {
    pub fn built_in_default(route: AgentConversationExecutionRoute) -> Self {
        Self {
            route,
            source: AgentConversationExecutionRouteSource::BuiltInDefault,
        }
    }
}

/// A route code was supplied but is not a canonical [`AgentConversationExecutionRoute`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InvalidExecutionRouteError {
    pub code: String,
}

impl std::fmt::Display for InvalidExecutionRouteError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "invalid execution route \"{}\": must be one of {EXECUTION_ROUTE_IN_PROCESS}, {EXECUTION_ROUTE_SANDBOX}",
            self.code
        )
    }
}

impl std::error::Error for InvalidExecutionRouteError {}

/// Resolves the effective execution route for one conversation turn.
///
/// A blank override or deployment default falls through to the next level; a
/// non-blank value that is not a canonical route code fails closed so a typo
/// can never silently downgrade to a different execution placement. Both
/// levels are validated even when the override wins, so a broken deployment
/// default surfaces instead of hiding behind per-request overrides.
pub fn resolve_agent_conversation_execution_route(
    request_override: Option<&str>,
    deployment_default: Option<&str>,
) -> Result<AgentConversationExecutionDecision, InvalidExecutionRouteError> {
    let parsed_override = parse_level(request_override)?;
    let parsed_default = parse_level(deployment_default)?;
    if let Some(route) = parsed_override {
        return Ok(AgentConversationExecutionDecision {
            route,
            source: AgentConversationExecutionRouteSource::RequestOverride,
        });
    }
    if let Some(route) = parsed_default {
        return Ok(AgentConversationExecutionDecision {
            route,
            source: AgentConversationExecutionRouteSource::DeploymentDefault,
        });
    }
    Ok(AgentConversationExecutionDecision::built_in_default(
        AgentConversationExecutionRoute::InProcess,
    ))
}

fn parse_level(
    code: Option<&str>,
) -> Result<Option<AgentConversationExecutionRoute>, InvalidExecutionRouteError> {
    if is_blank(code) {
        return Ok(None);
    }
    AgentConversationExecutionRoute::parse(code)
        .map(Some)
        .ok_or_else(|| InvalidExecutionRouteError {
            code: code.unwrap_or_default().trim().to_string(),
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn route_codes_round_trip() {
        assert_eq!(
            AgentConversationExecutionRoute::parse(Some("in_process")),
            Some(AgentConversationExecutionRoute::InProcess)
        );
        assert_eq!(
            AgentConversationExecutionRoute::parse(Some("sandbox")),
            Some(AgentConversationExecutionRoute::Sandbox)
        );
        assert_eq!(
            AgentConversationExecutionRoute::InProcess.as_str(),
            "in_process"
        );
        assert_eq!(AgentConversationExecutionRoute::Sandbox.as_str(), "sandbox");
    }

    #[test]
    fn blank_and_none_codes_parse_to_none() {
        assert_eq!(AgentConversationExecutionRoute::parse(None), None);
        assert_eq!(AgentConversationExecutionRoute::parse(Some("")), None);
        assert_eq!(AgentConversationExecutionRoute::parse(Some("   ")), None);
    }

    #[test]
    fn unknown_codes_never_parse() {
        assert_eq!(AgentConversationExecutionRoute::parse(Some("local")), None);
        assert_eq!(AgentConversationExecutionRoute::parse(Some("cloud")), None);
        assert_eq!(
            AgentConversationExecutionRoute::parse(Some("In_Process")),
            None
        );
    }

    #[test]
    fn request_override_wins_over_deployment_default() {
        let decision =
            resolve_agent_conversation_execution_route(Some("sandbox"), Some("in_process"))
                .expect("override resolves");
        assert_eq!(decision.route, AgentConversationExecutionRoute::Sandbox);
        assert_eq!(
            decision.source,
            AgentConversationExecutionRouteSource::RequestOverride
        );
    }

    #[test]
    fn deployment_default_applies_without_override() {
        let decision = resolve_agent_conversation_execution_route(None, Some("sandbox"))
            .expect("default resolves");
        assert_eq!(decision.route, AgentConversationExecutionRoute::Sandbox);
        assert_eq!(
            decision.source,
            AgentConversationExecutionRouteSource::DeploymentDefault
        );
    }

    #[test]
    fn blank_override_falls_through_to_deployment_default() {
        let decision = resolve_agent_conversation_execution_route(Some("  "), Some("sandbox"))
            .expect("blank override falls through");
        assert_eq!(decision.route, AgentConversationExecutionRoute::Sandbox);
        assert_eq!(
            decision.source,
            AgentConversationExecutionRouteSource::DeploymentDefault
        );
    }

    #[test]
    fn built_in_default_is_in_process() {
        let decision = resolve_agent_conversation_execution_route(None, None)
            .expect("built-in default resolves");
        assert_eq!(decision.route, AgentConversationExecutionRoute::InProcess);
        assert_eq!(
            decision.source,
            AgentConversationExecutionRouteSource::BuiltInDefault
        );
    }

    #[test]
    fn invalid_override_fails_closed_naming_the_code() {
        let error = resolve_agent_conversation_execution_route(Some("cluster"), Some("sandbox"))
            .expect_err("unknown code fails closed");
        assert_eq!(error.code, "cluster");
        assert!(error.to_string().contains("invalid execution route"));
    }

    #[test]
    fn invalid_deployment_default_fails_closed_even_with_valid_override() {
        // Fail-closed: a broken deployment default must surface, not hide
        // behind the override, so operators notice the misconfiguration.
        let error = resolve_agent_conversation_execution_route(Some("in_process"), Some("集群"))
            .expect_err("unknown default fails closed");
        assert_eq!(error.code, "集群");
    }
}

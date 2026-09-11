use crate::response::ApiProblem;
use crate::validation::{parse_owner_user_id, parse_tenant_id};
use sdkwork_agent_kernel::PolicySubject;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentRequestContext {
    pub tenant_id: String,
    pub organization_id: Option<String>,
    pub owner_user_id: String,
    pub subject_id: String,
    pub roles: Vec<String>,
    /// W3C trace id resolved from `traceparent` or `x-sdkwork-trace-id` at the gateway boundary.
    pub trace_id: Option<String>,
    /// Server request id used as the Problem+json fallback correlation when no trace id is present.
    pub request_id: Option<String>,
}

impl AgentRequestContext {
    pub fn new(tenant_id: impl Into<String>, owner_user_id: impl Into<String>) -> Self {
        let owner_user_id = owner_user_id.into();
        Self {
            tenant_id: tenant_id.into(),
            organization_id: None,
            subject_id: owner_user_id.clone(),
            owner_user_id,
            roles: Vec::new(),
            trace_id: None,
            request_id: None,
        }
    }

    pub fn with_organization_id(mut self, organization_id: impl Into<String>) -> Self {
        self.organization_id = Some(organization_id.into());
        self
    }

    pub fn with_subject_id(mut self, subject_id: impl Into<String>) -> Self {
        self.subject_id = subject_id.into();
        self
    }

    pub fn with_roles(mut self, roles: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.roles = roles.into_iter().map(Into::into).collect();
        self
    }

    pub fn with_trace_id(mut self, trace_id: impl Into<String>) -> Self {
        self.trace_id = Some(trace_id.into());
        self
    }

    pub fn with_request_id(mut self, request_id: impl Into<String>) -> Self {
        self.request_id = Some(request_id.into());
        self
    }

    fn subject(&self) -> PolicySubject {
        let mut subject = PolicySubject::new(self.subject_id.clone(), self.tenant_id.clone());
        for role in &self.roles {
            subject = subject.with_role(role.clone());
        }
        subject
    }
}

#[derive(Debug, Clone)]
pub(crate) struct RequestScope {
    pub(crate) tenant_id: String,
    pub(crate) organization_id: String,
    pub(crate) owner_user_id: String,
    pub(crate) subject: PolicySubject,
}

impl RequestScope {
    pub(crate) fn from_context(context: AgentRequestContext) -> Self {
        let subject = context.subject();
        Self {
            tenant_id: context.tenant_id.clone(),
            organization_id: context.organization_id.unwrap_or_else(|| "0".to_string()),
            owner_user_id: context.owner_user_id.clone(),
            subject,
        }
    }

    pub(crate) fn owner_scope(&self) -> Result<Option<u64>, ApiProblem> {
        parse_owner_user_id(&self.owner_user_id)
            .map(Some)
            .map_err(ApiProblem::from_kernel_error)
    }

    pub(crate) fn subject(&self) -> &PolicySubject {
        &self.subject
    }

    pub(crate) fn tenant_id_u64(&self) -> Result<u64, ApiProblem> {
        parse_tenant_id(self.tenant_id.as_str()).map_err(ApiProblem::from_kernel_error)
    }

    /// Parsed organization scope; contexts without an explicit organization
    /// resolve to the platform sentinel `0`.
    pub(crate) fn organization_id_u64(&self) -> Result<u64, ApiProblem> {
        parse_tenant_id(self.organization_id.as_str()).map_err(ApiProblem::from_kernel_error)
    }
}

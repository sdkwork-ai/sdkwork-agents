//! HTTP response envelope helpers aligned with `API_SPEC.md` §15.
//!
//! 这里的助手把成功响应统一包装成 `SdkWorkApiResponse`（`{ code: 0, data, traceId }`），
//! 错误响应统一通过 `sdkwork_web_core::problem_response` 生成 `application/problem+json`
//! （含 numeric `code` 与 `traceId`）。handlers 不再手写信封。

use axum::{
    http::{header::CACHE_CONTROL, HeaderName, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use sdkwork_utils_rust::{SdkWorkApiResponse, SdkWorkProblemDetail, SdkWorkResultCode};
use sdkwork_web_core::{
    problem_response, WebFrameworkError, WebFrameworkErrorKind, WebRequestContext,
};
use serde::Serialize;

/// Handler 内部 Result 别名：成功载荷 `T` 或 `ApiProblem`。
pub type ApiResult<T> = Result<T, ApiProblem>;

/// 单资源载荷形状：`{ item: T }`。
pub type ResourceData<T> = sdkwork_utils_rust::SdkWorkResourceData<T>;
/// 列表载荷形状：`{ items: Vec<T>, pageInfo }`。
pub type PageData<T> = sdkwork_utils_rust::SdkWorkPageData<T>;
/// 分页信息。
pub use sdkwork_utils_rust::{PageInfo, PageMode};

/// 直接透传成功载荷（仍由 `finish_api_json` 包信封）。
pub fn ok_json<T>(data: T) -> ApiResult<T> {
    Ok(data)
}

/// 构造 200 + `SdkWorkApiResponse` 信封 + `x-sdkwork-trace-id` 响应头。
pub fn success_json<T: Serialize>(
    ctx: &WebRequestContext,
    data: T,
) -> Result<Response, ApiProblem> {
    success_response(ctx, StatusCode::OK, data)
}

/// 构造 201 + `SdkWorkApiResponse` 信封 + `x-sdkwork-trace-id` 响应头。
pub fn created_json<T: Serialize>(
    ctx: &WebRequestContext,
    data: T,
) -> Result<Response, ApiProblem> {
    success_response(ctx, StatusCode::CREATED, data)
}

/// 构造 202 + `SdkWorkApiResponse` 信封 + `x-sdkwork-trace-id` 响应头。
///
/// Used by async creation endpoints that return a queued resource
/// (`agents.calls.create` with `executionMode: "async"`).
pub fn accepted_json<T: Serialize>(
    ctx: &WebRequestContext,
    data: T,
) -> Result<Response, ApiProblem> {
    success_response(ctx, StatusCode::ACCEPTED, data)
}

/// 构造 204 无 body + `x-sdkwork-trace-id` 响应头。
pub fn no_content(ctx: &WebRequestContext) -> Result<Response, ApiProblem> {
    Ok(finalize_response(
        ctx,
        StatusCode::NO_CONTENT.into_response(),
    ))
}

pub(crate) fn success_response<T: Serialize>(
    ctx: &WebRequestContext,
    status: StatusCode,
    data: T,
) -> Result<Response, ApiProblem> {
    let trace_id = ctx.resolved_trace_id();
    let envelope = SdkWorkApiResponse::success(data, trace_id.clone());
    Ok(finalize_response(
        ctx,
        (status, Json(envelope)).into_response(),
    ))
}

fn finalize_response(ctx: &WebRequestContext, mut response: Response) -> Response {
    attach_trace_header(&mut response, &ctx.resolved_trace_id());
    if !ctx.is_public() {
        attach_private_no_store(&mut response);
    }
    response
}

fn attach_private_no_store(response: &mut Response) {
    response
        .headers_mut()
        .insert(CACHE_CONTROL, HeaderValue::from_static("private, no-store"));
}

fn attach_trace_header(response: &mut Response, trace_id: &str) {
    if let Ok(value) = HeaderValue::from_str(trace_id) {
        response
            .headers_mut()
            .insert(HeaderName::from_static("x-sdkwork-trace-id"), value);
    }
}

/// 业务错误类型，最终通过 `sdkwork_web_core::problem_response` 序列化为 `application/problem+json`。
#[derive(Debug)]
pub struct ApiProblem {
    pub message: String,
    status: StatusCode,
    result_code: Option<SdkWorkResultCode>,
}

impl ApiProblem {
    pub fn bad_request(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            status: StatusCode::BAD_REQUEST,
            result_code: None,
        }
    }

    pub fn invalid_parameter(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            status: StatusCode::BAD_REQUEST,
            result_code: Some(SdkWorkResultCode::InvalidParameter),
        }
    }

    pub fn forbidden(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            status: StatusCode::FORBIDDEN,
            result_code: None,
        }
    }

    pub fn not_found(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            status: StatusCode::NOT_FOUND,
            result_code: None,
        }
    }

    pub fn conflict(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            status: StatusCode::CONFLICT,
            result_code: None,
        }
    }

    pub fn unprocessable(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            status: StatusCode::UNPROCESSABLE_ENTITY,
            result_code: None,
        }
    }

    pub fn payload_too_large(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            status: StatusCode::PAYLOAD_TOO_LARGE,
            result_code: None,
        }
    }

    pub fn too_many_requests(message: impl Into<String>, _retry_after: Option<u64>) -> Self {
        Self {
            message: message.into(),
            status: StatusCode::TOO_MANY_REQUESTS,
            result_code: None,
        }
    }

    pub fn dependency_unavailable(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            status: StatusCode::SERVICE_UNAVAILABLE,
            result_code: None,
        }
    }

    /// Carries a standard SDKWork result code (e.g. `50301`) so the problem
    /// body includes `code`/`i18nKey` while keeping the business-safe message
    /// in `detail` instead of the generic dependency-unavailable text.
    pub fn with_result_code(mut self, result_code: SdkWorkResultCode) -> Self {
        self.result_code = Some(result_code);
        self
    }

    pub fn gateway_timeout(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            status: StatusCode::GATEWAY_TIMEOUT,
            result_code: None,
        }
    }

    pub fn internal_server_error(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            status: StatusCode::INTERNAL_SERVER_ERROR,
            result_code: None,
        }
    }

    /// Alias kept for legacy call sites that referred to validation errors as `validation`.
    pub fn validation(message: impl Into<String>) -> Self {
        Self::bad_request(message)
    }

    /// Alias kept for legacy call sites that referred to permission errors as `permission`.
    pub fn permission(message: impl Into<String>) -> Self {
        Self::forbidden(message)
    }

    /// Alias kept for legacy call sites that referred to optimistic-concurrency conflicts as `version_conflict`.
    pub fn version_conflict(message: impl Into<String>) -> Self {
        Self::conflict(message)
    }

    /// Alias kept for legacy call sites that referred to internal errors as `internal`.
    pub fn internal(message: impl Into<String>) -> Self {
        Self::internal_server_error(message)
    }

    pub fn from_web_framework(error: sdkwork_web_core::WebFrameworkError) -> Self {
        let status = error.status();
        Self {
            message: error.message,
            status,
            result_code: None,
        }
    }

    fn framework_error(&self) -> WebFrameworkError {
        let kind = match self.status {
            StatusCode::BAD_REQUEST => WebFrameworkErrorKind::BadRequest,
            StatusCode::FORBIDDEN => WebFrameworkErrorKind::Forbidden,
            StatusCode::NOT_FOUND => WebFrameworkErrorKind::NotFound,
            StatusCode::CONFLICT => WebFrameworkErrorKind::Conflict,
            StatusCode::PAYLOAD_TOO_LARGE => WebFrameworkErrorKind::PayloadTooLarge,
            StatusCode::TOO_MANY_REQUESTS => WebFrameworkErrorKind::RateLimitExceeded,
            StatusCode::SERVICE_UNAVAILABLE => WebFrameworkErrorKind::DependencyUnavailable,
            StatusCode::REQUEST_TIMEOUT => WebFrameworkErrorKind::RequestTimeout,
            StatusCode::METHOD_NOT_ALLOWED => WebFrameworkErrorKind::MethodNotAllowed,
            StatusCode::UNAUTHORIZED => WebFrameworkErrorKind::MissingCredentials,
            StatusCode::UNPROCESSABLE_ENTITY => WebFrameworkErrorKind::BadRequest,
            StatusCode::INTERNAL_SERVER_ERROR => WebFrameworkErrorKind::InternalServerError,
            _ => WebFrameworkErrorKind::InternalServerError,
        };
        let mut error = WebFrameworkError::internal_server_error(self.message.clone());
        error.kind = kind;
        error
    }

    pub fn into_response_for(&self, ctx: &WebRequestContext) -> Response {
        if let Some(result_code) = self.result_code {
            let trace_id = ctx.resolved_trace_id();
            let mut problem = SdkWorkProblemDetail::platform_enriched(
                result_code,
                self.message.clone(),
                trace_id.clone(),
                ctx.problem_correlation().routing(),
            );
            // `platform_enriched` redacts dependency-unavailable details to a
            // generic text. ApiProblem messages are business-safe (they come
            // from kernel `safe_message` and the executor's actionable hints),
            // so keep the real reason in `detail` — otherwise callers can only
            // see "A required dependency is temporarily unavailable" and
            // cannot tell account-pool, auth-token, or provider failures apart.
            if !self.message.trim().is_empty() {
                problem.detail = Some(self.message.clone());
            }
            let response = (
                self.status,
                [(axum::http::header::CONTENT_TYPE, "application/problem+json")],
                Json(problem),
            )
                .into_response();
            return finalize_response(ctx, response);
        }
        finalize_response(
            ctx,
            problem_response(&self.framework_error(), ctx.problem_correlation()),
        )
    }

    /// Render the problem without a request context.
    ///
    /// Used only by middleware that runs before `WebRequestContext` is established
    /// (e.g. header extraction failures at the gateway edge). A fresh server-side
    /// trace id is generated so the response still carries a `traceId` for support
    /// correlation. Handlers MUST prefer `into_response_for(ctx)` / `finish_api_json`.
    pub fn into_response_fallback(self) -> Response {
        let correlation = sdkwork_web_core::ProblemCorrelation::new(None, None);
        let mut response = problem_response(&self.framework_error(), correlation);
        attach_private_no_store(&mut response);
        response
    }

    pub fn status(&self) -> StatusCode {
        self.status
    }
}

/// 收尾 `ApiResult<T>`：成功走 `success_json`，失败走 `problem_response`。
pub fn finish_api_json<T: Serialize>(ctx: &WebRequestContext, result: ApiResult<T>) -> Response {
    match result {
        Ok(data) => {
            success_json(ctx, data).unwrap_or_else(|problem| problem.into_response_for(ctx))
        }
        Err(problem) => problem.into_response_for(ctx),
    }
}

/// Finish a create operation with the canonical HTTP 201 success status.
pub fn finish_created_api_json<T: Serialize>(
    ctx: &WebRequestContext,
    result: ApiResult<T>,
) -> Response {
    match result {
        Ok(data) => {
            created_json(ctx, data).unwrap_or_else(|problem| problem.into_response_for(ctx))
        }
        Err(problem) => problem.into_response_for(ctx),
    }
}

/// 收尾 `Result<Response, ApiProblem>`：成功透传，失败走 `problem_response`。
pub fn finish_api_response(
    ctx: &WebRequestContext,
    result: Result<Response, ApiProblem>,
) -> Response {
    match result {
        Ok(response) => finalize_response(ctx, response),
        Err(problem) => problem.into_response_for(ctx),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::to_bytes;
    use sdkwork_web_core::{ServerRequestId, WebApiSurface, WebAuthMode, WebTransportFacts};

    fn test_context() -> WebRequestContext {
        WebRequestContext {
            request_id: ServerRequestId("test-req".to_owned()),
            api_surface: WebApiSurface::BackendApi,
            auth_mode: WebAuthMode::DualToken,
            principal: None,
            transport: WebTransportFacts {
                path: "/backend/v3/api/ai/agents".to_owned(),
                method: "GET".to_owned(),
                auth_token_present: true,
                access_token_present: true,
                api_key_present: false,
                ingress_token_present: false,
                oauth_bearer_present: false,
                agent_token_present: false,
            },
            locale: None,
            client_kind: None,
            operation: None,
            trace_id: Some("trace-from-context-abc".to_owned()),
            idempotency_key: None,
        }
    }

    #[tokio::test]
    async fn success_json_uses_sdkwork_api_response_envelope() {
        let response =
            success_json(&test_context(), serde_json::json!({ "item": 1 })).expect("response");
        assert_eq!(
            Some("private, no-store"),
            response
                .headers()
                .get(CACHE_CONTROL)
                .and_then(|value| value.to_str().ok())
        );
        let body = to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("body");
        let payload: serde_json::Value = serde_json::from_slice(&body).expect("json");
        assert_eq!(0, payload["code"].as_i64().unwrap());
        assert_eq!(
            "trace-from-context-abc",
            payload["traceId"].as_str().unwrap()
        );
        assert_eq!(1, payload["data"]["item"].as_i64().unwrap());
    }

    #[tokio::test]
    async fn api_problem_uses_problem_json_content_type() {
        let response =
            ApiProblem::forbidden("missing permission").into_response_for(&test_context());
        assert_eq!(
            Some("private, no-store"),
            response
                .headers()
                .get(CACHE_CONTROL)
                .and_then(|value| value.to_str().ok())
        );
        assert_eq!(
            "application/problem+json",
            response
                .headers()
                .get(axum::http::header::CONTENT_TYPE)
                .and_then(|value| value.to_str().ok())
                .unwrap_or_default()
        );
        let body = to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("body");
        let payload: serde_json::Value = serde_json::from_slice(&body).expect("json");
        assert_eq!(403, payload["status"].as_u64().unwrap());
        assert_eq!(40301, payload["code"].as_i64().unwrap());
        assert!(payload["detail"]
            .as_str()
            .unwrap()
            .contains("missing permission"));
        assert!(!payload.to_string().contains("backtrace"));
    }

    #[tokio::test]
    async fn api_problem_into_response_for_includes_request_correlation() {
        let ctx = test_context();
        let response = ApiProblem::forbidden("missing permission").into_response_for(&ctx);
        let body = to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("body");
        let payload: serde_json::Value = serde_json::from_slice(&body).expect("json");
        assert_eq!(
            "trace-from-context-abc",
            payload["traceId"].as_str().unwrap()
        );
        assert!(payload.get("requestId").is_none());
    }

    #[tokio::test]
    async fn api_problem_not_found_returns_404_problem_json() {
        let response = ApiProblem::not_found("agent missing").into_response_for(&test_context());
        let body = to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("body");
        let payload: serde_json::Value = serde_json::from_slice(&body).expect("json");
        assert_eq!(404, payload["status"].as_u64().unwrap());
        assert_eq!(40401, payload["code"].as_i64().unwrap());
        assert_eq!(
            "https://docs.sdkwork.com/problems/40401",
            payload["type"].as_str().unwrap()
        );
    }

    #[tokio::test]
    async fn no_content_response_has_no_body() {
        let response = no_content(&test_context()).expect("response");
        assert_eq!(StatusCode::NO_CONTENT, response.status());
        assert!(response.headers().get("x-sdkwork-trace-id").is_some());
        assert_eq!(
            Some("private, no-store"),
            response
                .headers()
                .get(CACHE_CONTROL)
                .and_then(|value| value.to_str().ok())
        );
        let body = to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("body");
        assert!(body.is_empty());
    }

    #[tokio::test]
    async fn api_problem_conflict_returns_409_problem_json() {
        let response =
            ApiProblem::conflict("agent already exists").into_response_for(&test_context());
        let body = to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("body");
        let payload: serde_json::Value = serde_json::from_slice(&body).expect("json");
        assert_eq!(409, payload["status"].as_u64().unwrap());
        assert_eq!(40901, payload["code"].as_i64().unwrap());
    }

    #[tokio::test]
    async fn api_problem_dependency_unavailable_returns_503_problem_json() {
        let response = ApiProblem::dependency_unavailable("database operation failed")
            .into_response_for(&test_context());
        let body = to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("body");
        let payload: serde_json::Value = serde_json::from_slice(&body).expect("json");
        assert_eq!(503, payload["status"].as_u64().unwrap());
        assert_eq!(50301, payload["code"].as_i64().unwrap());
    }

    #[tokio::test]
    async fn api_problem_service_unavailable_with_result_code_keeps_business_detail() {
        let response = ApiProblem::dependency_unavailable(
            "cloud router chat completion failed: 当前租户在账号池中无可用账号",
        )
        .with_result_code(SdkWorkResultCode::ServiceUnavailable)
        .into_response_for(&test_context());
        let body = to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("body");
        let payload: serde_json::Value = serde_json::from_slice(&body).expect("json");
        assert_eq!(503, payload["status"].as_u64().unwrap());
        assert_eq!(50301, payload["code"].as_i64().unwrap());
        assert_eq!(
            "errors.result.50301",
            payload["i18nKey"].as_str().expect("i18nKey")
        );
        // The business-safe message survives instead of the generic
        // "A required dependency is temporarily unavailable" text.
        assert!(
            payload["detail"]
                .as_str()
                .expect("detail")
                .contains("账号池中无可用账号"),
            "detail must carry the business reason"
        );
    }

    #[test]
    fn finish_api_response_applies_authenticated_cache_policy() {
        let response =
            finish_api_response(&test_context(), Ok(StatusCode::ACCEPTED.into_response()));

        assert_eq!(StatusCode::ACCEPTED, response.status());
        assert_eq!(
            Some("private, no-store"),
            response
                .headers()
                .get(CACHE_CONTROL)
                .and_then(|value| value.to_str().ok())
        );
        assert!(response.headers().get("x-sdkwork-trace-id").is_some());
    }

    #[test]
    fn public_response_does_not_force_private_cache_policy() {
        let mut context = test_context();
        context.auth_mode = WebAuthMode::Public;

        let response =
            success_json(&context, serde_json::json!({ "item": 1 })).expect("public response");

        assert!(response.headers().get(CACHE_CONTROL).is_none());
    }
}

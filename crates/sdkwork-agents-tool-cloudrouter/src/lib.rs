//! Cloud Router open-api adapter for the SDKWork Agents media tool family
//! and the RIG agent engine model backend.

mod chat_stream;
mod client;
mod credentials;
mod rig_executor;
mod turn_transport;
mod vendor;
mod wire_protocol;

pub use chat_stream::{
    create_llm_completion_blocking, stream_chat_completion_blocking,
    stream_chat_completion_with_tools_blocking, stream_llm_completion_blocking,
    CloudRouterChatStreamResult, CloudRouterCompletionResult, CloudRouterStreamDelta,
    StreamedToolCall,
};
pub use client::{
    cloudrouter_base_url, cloudrouter_base_url_optional, cloudrouter_http_error_hint,
    is_cloudrouter_insufficient_balance, map_cloudrouter_error, resolve_cloudrouter_base_url,
    run_sync, CloudRouterMediaClient, CLOUDROUTER_INSUFFICIENT_BALANCE_CODE,
    DEFAULT_CLOUDROUTER_BASE_URL, ENV_CLOUDROUTER_BASE_URL, ENV_CLOUDROUTER_INGRESS_BIND,
    ENV_CLOUDROUTER_PUBLIC_HTTP_URL, FUNDING_SHORTFALL_DETAIL_KEY,
};
pub use credentials::{embedded_open_api_key, HopCredentials};
pub use rig_executor::{
    map_cloudrouter_kernel_error, RigCloudRouterExecutor, RigCloudRouterModelProvider,
};
pub use turn_transport::{
    embedded_cloudrouter_surface, install_embedded_cloudrouter_surface, turn_service_mode,
    CloudRouterTransportError, CloudRouterTurnRequest, CloudRouterTurnTransport,
    EmbeddedSurfaceDispatcher, SurfaceOutcome,
};
pub use vendor::{
    model_arg, normalize_vendor_status, normalized_vendor_media, optional_i64_arg, string_array_arg,
};
pub use wire_protocol::WireProtocol;

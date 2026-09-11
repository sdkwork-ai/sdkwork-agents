//! Cloud Router open-api adapter for the SDKWork Agents media tool family
//! and the RIG agent engine model backend.

mod chat_stream;
mod client;
mod rig_executor;
mod vendor;
mod wire_protocol;

pub use chat_stream::{
    CloudRouterChatStreamResult, CloudRouterCompletionResult, CloudRouterStreamDelta,
    create_llm_completion_blocking, stream_chat_completion_blocking,
    stream_llm_completion_blocking,
};
pub use client::{cloudrouter_http_error_hint,
    cloudrouter_base_url, map_cloudrouter_error, run_sync, CloudRouterMediaClient,
    DEFAULT_CLOUDROUTER_BASE_URL, ENV_CLOUDROUTER_BASE_URL, ENV_CLOUDROUTER_INGRESS_BIND,
};
pub use rig_executor::{
    map_cloudrouter_kernel_error, RigCloudRouterExecutor, RigCloudRouterModelProvider,
};
pub use vendor::{
    model_arg, normalize_vendor_status, normalized_vendor_media, optional_i64_arg, string_array_arg,
};
pub use wire_protocol::WireProtocol;

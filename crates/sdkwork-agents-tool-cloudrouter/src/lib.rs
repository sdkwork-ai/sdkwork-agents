//! Cloud Router open-api adapter for the SDKWork Agents media tool family
//! and the RIG agent engine model backend.

mod client;
mod rig_executor;
mod vendor;

pub use client::{
    cloudrouter_base_url, map_cloudrouter_error, run_sync, CloudRouterMediaClient,
    DEFAULT_CLOUDROUTER_BASE_URL, ENV_CLOUDROUTER_BASE_URL, ENV_CLOUDROUTER_INGRESS_BIND,
};
pub use rig_executor::{map_cloudrouter_kernel_error, RigCloudRouterExecutor};
pub use vendor::{
    model_arg, normalize_vendor_status, normalized_vendor_media, optional_i64_arg, string_array_arg,
};

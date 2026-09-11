use axum::Router;
use sdkwork_routes_drive_app_api::wrap_router_with_iam_web_framework;

/// Wire the embedded Assets App API (`/app/v3/api/assets*`) into the Agents
/// standalone gateway.
///
/// Assets catalog authority lives in `sdkwork-assets`; the assembly reuses the
/// Drive database host because asset metadata is stored in `dr_drive_node`.
pub(super) async fn wire_assets_app_router() -> Result<Router, String> {
    let contribution = sdkwork_api_assets_assembly::assemble_app_api_contribution()
        .await
        .map_err(|error| format!("failed to assemble embedded Assets app API: {error}"))?;
    let resolver = sdkwork_iam_web_adapter::iam_web_request_context_resolver_from_env().await;
    Ok(wrap_router_with_iam_web_framework(
        resolver,
        contribution.router,
    ))
}

#[cfg(test)]
mod tests {
    #[test]
    fn assets_router_wire_contract_is_present() {
        // Assets routes enter through the assets assembly contribution, never
        // through a direct dependency on sdkwork-routes-assets-app-api
        // (API_ASSEMBLY_SPEC §3 dependency composition rules). The wire
        // function is exercised by the standalone bootstrap integration test;
        // this unit test only pins the assembly-based wiring path.
        let _ = super::wire_assets_app_router;
    }
}

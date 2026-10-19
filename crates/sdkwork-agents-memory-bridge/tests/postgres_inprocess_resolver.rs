//! Optional PostgreSQL proof that the in-process Agents resolver returns a written memory.
//!
//! Runs only when `SDKWORK_DATABASE_URL` is set.

use sdkwork_agents_memory_bridge::InProcessMemoryGroundingResolver;
use sdkwork_intelligence_agents_service::{
    GroundingRequest, MemoryGroundingBinding, MemoryGroundingResolver,
};
use sdkwork_memory_inprocess::InProcessMemoryRuntime;

#[tokio::test]
async fn postgres_inprocess_resolver_returns_written_memory() {
    let Some(database_url) = std::env::var("SDKWORK_DATABASE_URL")
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
    else {
        eprintln!("skipping in-process agents resolver test: set SDKWORK_DATABASE_URL");
        return;
    };
    std::env::set_var("SDKWORK_DATABASE_URL", &database_url);

    let runtime = InProcessMemoryRuntime::from_env()
        .await
        .expect("bootstrap memory runtime")
        .expect("memory runtime configured");
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let actor_id = 720_000_000_u64 + (nonce % 100_000_000) as u64;
    let space = runtime
        .create_space(100_001, Some(actor_id), format!("agent provider {nonce}"))
        .await
        .expect("create memory space");
    runtime
        .create_memory(
            100_001,
            Some(actor_id),
            space.space_id,
            format!("Refund preference {nonce}: refunds are accepted within 30 days."),
        )
        .await
        .expect("write memory");

    let resolver = InProcessMemoryGroundingResolver::new(std::sync::Arc::new(runtime));
    let bindings = [MemoryGroundingBinding {
        slot_id: "slot.memory.test".to_string(),
        memory_policy_ref: space.space_id.to_string(),
        top_k: Some(3),
    }];
    let snippets = tokio::task::spawn_blocking(move || {
        let request = GroundingRequest {
            tenant_id: 100_001,
            organization_id: 0,
            actor_id: Some(actor_id),
            agent_id: "agent.chat.default",
            session_id: "session.1",
            query: "refund preferences",
            knowledge: &[],
            memory: &bindings,
            auth_token: None,
            access_token: None,
        };
        resolver.resolve_memory(&request)
    })
    .await
    .expect("grounding task")
    .expect("memory snippets");

    assert!(!snippets.is_empty(), "written memory must be retrievable");
    assert!(snippets[0].content.contains("30 days"));
}

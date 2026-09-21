//! Regression test: the agents cloudrouter executor must send BOTH the caller
//! auth token (`Authorization: Bearer <auth>`) and the access token
//! (`Access-Token: <access>`) when invoking the cloudrouter gateway
//! (API_SPEC §819/§824 dual-token access).
//!
//! A previous SDK defect made `set_access_token` remove the `Authorization`
//! bearer; the gateway then rejected the turn with
//! `401 missing api key credential`. The executor applies `set_access_token`
//! before `set_auth_token` so both headers always reach the wire, and this
//! test asserts the exact headers of the HTTP request.

use std::io::{BufRead, BufReader, Write};
use std::net::TcpListener;
use std::sync::{Arc, Mutex};
use std::thread;

use sdkwork_agent_kernel::{
    FilesystemRequest, HostProvider, KernelError, KernelResult, ModelRequest, NetworkRequest,
    ProcessRequest, ProviderHealth, ProviderManifest, ProviderSecretValue, SecretRef,
};
use sdkwork_agent_provider_rig::{RigBackendConfig, RigBackendExecutor, RigBackendMode};
use sdkwork_agents_tool_cloudrouter::RigCloudRouterExecutor;

/// Host that never resolves secrets — the dual-token path must not touch it.
struct UnusedSecretHost;

impl HostProvider for UnusedSecretHost {
    fn provider_manifest(&self) -> ProviderManifest {
        ProviderManifest::new(
            "provider.test-host",
            "test",
            "Test secret host",
            "0.1.0",
            Vec::new(),
        )
    }

    fn health(&self) -> ProviderHealth {
        ProviderHealth::available()
    }

    fn filesystem(
        &self,
        _request: FilesystemRequest,
    ) -> KernelResult<sdkwork_agent_kernel::FilesystemResult> {
        Err(KernelError::CapabilityMissing {
            capability_id: "filesystem".to_string(),
        })
    }

    fn process(
        &self,
        _request: ProcessRequest,
    ) -> KernelResult<sdkwork_agent_kernel::ProcessResult> {
        Err(KernelError::CapabilityMissing {
            capability_id: "process".to_string(),
        })
    }

    fn network(
        &self,
        _request: NetworkRequest,
    ) -> KernelResult<sdkwork_agent_kernel::NetworkResult> {
        Err(KernelError::CapabilityMissing {
            capability_id: "network".to_string(),
        })
    }

    fn resolve_secret(&self, secret_ref: SecretRef) -> KernelResult<ProviderSecretValue> {
        Err(KernelError::CapabilityMissing {
            capability_id: secret_ref.secret_ref_id,
        })
    }
}

/// Serves one OpenAI chat completion and records the request's
/// `Authorization` and `Access-Token` header values.
fn spawn_header_recording_server() -> (String, Arc<Mutex<Option<(String, String)>>>) {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind loopback");
    let addr = listener.local_addr().expect("local addr");
    let recorded: Arc<Mutex<Option<(String, String)>>> = Arc::new(Mutex::new(None));
    let recorder = Arc::clone(&recorded);
    thread::spawn(move || {
        let (mut stream, _) = listener.accept().expect("accept");
        let mut reader = BufReader::new(stream.try_clone().expect("clone stream"));
        let mut head = String::new();
        loop {
            let mut line = String::new();
            let read = reader.read_line(&mut line).expect("read line");
            if read == 0 || line == "\r\n" || line == "\n" {
                break;
            }
            head.push_str(&line);
        }
        let mut authorization = None;
        let mut access_token = None;
        for line in head.lines() {
            if let Some((name, value)) = line.split_once(':') {
                let name = name.trim().to_ascii_lowercase();
                let value = value.trim().to_string();
                if name == "authorization" {
                    authorization = Some(value);
                } else if name == "access-token" {
                    access_token = Some(value);
                }
            }
        }
        *recorder.lock().expect("recorder lock") = Some((
            authorization.unwrap_or_default(),
            access_token.unwrap_or_default(),
        ));
        let body = r#"{"id":"chatcmpl-test","object":"chat.completion","created":1750000000,"model":"default","choices":[{"index":0,"message":{"role":"assistant","content":"pong"},"finish_reason":"stop"}],"usage":{"prompt_tokens":1,"completion_tokens":1,"total_tokens":2}}"#;
        let response = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            body.len(),
            body
        );
        stream.write_all(response.as_bytes()).expect("write");
        stream.flush().expect("flush");
    });
    (format!("http://{addr}"), recorded)
}

#[test]
fn rig_executor_sends_dual_tokens_on_chat_completion() {
    let (base_url, recorded) = spawn_header_recording_server();
    let config = RigBackendConfig {
        mode: RigBackendMode::Live,
        provider_id: None,
        api_key_secret_ref: Some("secret://rig/cloudrouter".to_string()),
        base_url: None,
    };
    let executor =
        RigCloudRouterExecutor::with_base_url(config, Arc::new(UnusedSecretHost), base_url);

    let request = ModelRequest::new("request-1", vec!["user: hello".to_string()]).for_caller(
        Some("auth-token-abc".to_string()),
        Some("access-token-xyz".to_string()),
    );

    let response = executor.invoke_model(request).expect("chat completion");
    assert_eq!(response.messages.join("\n"), "pong");

    let (authorization, access_token) = recorded
        .lock()
        .expect("recorded lock")
        .clone()
        .expect("headers recorded");
    assert_eq!(
        authorization, "Bearer auth-token-abc",
        "Authorization bearer must carry the caller auth token"
    );
    assert_eq!(
        access_token, "access-token-xyz",
        "Access-Token must carry the caller access token"
    );
}

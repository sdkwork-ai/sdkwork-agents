//! HTTP-backed generations tool port for the turn loop.
//!
//! The generations MCP tools execute against the federated generations app
//! API mounted on the cloudrouter gateway (`/app/v3/api/generations/*`),
//! authenticating with the caller's dual tokens so tenant scope, billing, and
//! durable generation records resolve server-side. Keeping the port HTTP
//! preserves the service boundary: the agents process never embeds the
//! generations database or provider adapters.

use std::sync::OnceLock;
use std::time::Duration;

use reqwest::blocking::Client;

use sdkwork_generations_mcp_service::{
    GenerateImageInput, GenerateMusicInput, GenerateSoundEffectInput, GenerateVideoInput,
    GenerationRetrieveInput, SynthesizeSpeechInput,
};

const GENERATIONS_CONNECT_TIMEOUT: Duration = Duration::from_secs(10);
const GENERATIONS_REQUEST_TIMEOUT: Duration = Duration::from_secs(300);

/// Endpoint paths of the federated generations app API, keyed by
/// `(modality, operation)`.
fn generation_endpoint(modality: &str, operation: &str) -> Option<String> {
    match (modality, operation) {
        ("image", "text_to_image") => {
            Some("/app/v3/api/generations/images/text_to_image".to_string())
        }
        ("image", "image_edit") => Some("/app/v3/api/generations/images/image_edit".to_string()),
        ("video", "text_to_video") => {
            Some("/app/v3/api/generations/videos/text_to_video".to_string())
        }
        ("video", "image_to_video") => {
            Some("/app/v3/api/generations/videos/image_to_video".to_string())
        }
        ("video", "video_extend") => {
            Some("/app/v3/api/generations/videos/video_extend".to_string())
        }
        ("music", "text_to_music") => {
            Some("/app/v3/api/generations/music/text_to_music".to_string())
        }
        ("music", "lyrics_to_music") => {
            Some("/app/v3/api/generations/music/lyrics_to_music".to_string())
        }
        ("voice", "speech") => Some("/app/v3/api/generations/voice/speech".to_string()),
        ("sfx", "sound_effects") => {
            Some("/app/v3/api/generations/sound_effects".to_string())
        }
        _ => None,
    }
}

/// Whether a port error string is an insufficient-balance rejection.
///
/// HTTP failures surface through the stable `"generations api returned
/// {status}: {body}"` prefix produced by [`HttpGenerationsPort::post`] and
/// [`HttpGenerationsPort::get`]; the status token and body feed the shared
/// cloudrouter detector (402 / code 40201 / legacy precharge shapes) so the
/// tool loop can escalate the turn to the recharge affordance.
pub fn is_insufficient_balance_failure(error: &str) -> bool {
    let Some(rest) = error.strip_prefix("generations api returned ") else {
        return false;
    };
    let (status_token, body) = match rest.split_once(':') {
        Some((status_token, body)) => (status_token, body),
        None => (rest, ""),
    };
    let status = status_token
        .split_whitespace()
        .next()
        .and_then(|token| token.parse::<u16>().ok())
        .unwrap_or(0);
    sdkwork_agents_tool_cloudrouter::is_cloudrouter_insufficient_balance(status, body)
}

/// Blocking HTTP port for the generations app API.
///
/// The HTTP client is built lazily: a misconfigured environment degrades to
/// reqwest's default client instead of panicking on library construction
/// (RUST_CODE_SPEC: no `unwrap`/`expect`/`panic!` reachable from public API).
#[derive(Debug, Clone)]
pub struct HttpGenerationsPort {
    base_url: String,
    client: OnceLock<Client>,
}

impl HttpGenerationsPort {
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into(),
            client: OnceLock::new(),
        }
    }

    fn client(&self) -> &Client {
        self.client.get_or_init(|| {
            Client::builder()
                .connect_timeout(GENERATIONS_CONNECT_TIMEOUT)
                .timeout(GENERATIONS_REQUEST_TIMEOUT)
                .build()
                .unwrap_or_else(|_| Client::new())
        })
    }

    /// Creates one generation command. Returns the raw `data.item` payload.
    #[allow(clippy::too_many_arguments)]
    pub fn create_generation(
        &self,
        modality: &str,
        operation: &str,
        tenant_id: u64,
        prompt: &str,
        model: Option<&str>,
        parameters: serde_json::Value,
        input_asset_ids: Option<Vec<String>>,
        auth_token: &str,
        access_token: Option<&str>,
        idempotency_key: &str,
    ) -> Result<serde_json::Value, String> {
        let endpoint = generation_endpoint(modality, operation)
            .ok_or_else(|| format!("generations has no endpoint for {modality}.{operation}"))?;
        // Context selectors (tenant/organization) resolve from the
        // authenticated session server-side (API_SPEC §10.0); a client-supplied
        // `tenantId` body field is rejected with 40001.
        let mut body = serde_json::json!({
            "prompt": prompt,
        });
        if let Some(model) = model.filter(|value| !value.trim().is_empty()) {
            body["model"] = serde_json::json!(model);
        }
        if !parameters.is_null() {
            body["parameters"] = parameters;
        }
        if let Some(input_asset_ids) = input_asset_ids.filter(|ids| !ids.is_empty()) {
            body["inputAssetIds"] = serde_json::json!(input_asset_ids);
        }
        let payload = self.post(
            &endpoint,
            body,
            auth_token,
            access_token,
            Some(idempotency_key),
        )?;
        payload
            .get("data")
            .and_then(|data| data.get("item"))
            .cloned()
            .ok_or_else(|| format!("generations create response missing data.item: {payload}"))
    }

    /// Fetches one generation record (refreshing async tasks on read).
    pub fn get_generation(
        &self,
        generation_id: &str,
        auth_token: &str,
        access_token: Option<&str>,
    ) -> Result<serde_json::Value, String> {
        let endpoint = format!("/app/v3/api/generations/{generation_id}");
        self.get(&endpoint, auth_token, access_token)
            .map(|payload| {
                payload
                    .get("data")
                    .and_then(|data| data.get("item"))
                    .cloned()
                    .unwrap_or(payload)
            })
    }

    /// Lists the persisted results of one generation.
    pub fn list_results(
        &self,
        generation_id: &str,
        auth_token: &str,
        access_token: Option<&str>,
    ) -> Result<serde_json::Value, String> {
        let endpoint = format!("/app/v3/api/generations/{generation_id}/results");
        self.get(&endpoint, auth_token, access_token)
    }

    fn post(
        &self,
        endpoint: &str,
        body: serde_json::Value,
        auth_token: &str,
        access_token: Option<&str>,
        idempotency_key: Option<&str>,
    ) -> Result<serde_json::Value, String> {
        let url = format!("{}{}", self.base_url.trim_end_matches('/'), endpoint);
        let mut request = self.client().post(&url).bearer_auth(auth_token).json(&body);
        if let Some(access_token) = access_token.filter(|token| !token.trim().is_empty()) {
            request = request.header("Access-Token", access_token);
        }
        if let Some(idempotency_key) = idempotency_key {
            request = request.header("Idempotency-Key", idempotency_key);
        }
        let response = request
            .send()
            .map_err(|error| format!("generations request failed: {error}"))?;
        let status = response.status();
        if !status.is_success() {
            let body = response.text().unwrap_or_default();
            return Err(format!("generations api returned {status}: {body}"));
        }
        response
            .json::<serde_json::Value>()
            .map_err(|error| format!("generations api returned invalid JSON: {error}"))
    }

    fn get(
        &self,
        endpoint: &str,
        auth_token: &str,
        access_token: Option<&str>,
    ) -> Result<serde_json::Value, String> {
        let url = format!("{}{}", self.base_url.trim_end_matches('/'), endpoint);
        let mut request = self.client().get(&url).bearer_auth(auth_token);
        if let Some(access_token) = access_token.filter(|token| !token.trim().is_empty()) {
            request = request.header("Access-Token", access_token);
        }
        let response = request
            .send()
            .map_err(|error| format!("generations request failed: {error}"))?;
        let status = response.status();
        if !status.is_success() {
            let body = response.text().unwrap_or_default();
            return Err(format!("generations api returned {status}: {body}"));
        }
        response
            .json::<serde_json::Value>()
            .map_err(|error| format!("generations api returned invalid JSON: {error}"))
    }
}

/// Builds the `parameters` payload for image creation tools.
pub fn image_parameters(input: &GenerateImageInput) -> serde_json::Value {
    let mut parameters = serde_json::Map::new();
    if let Some(vendor) = input.vendor.as_deref() {
        parameters.insert("vendor".to_string(), serde_json::json!(vendor));
    }
    if input.aspect_ratio.is_some() || input.image_count.is_some() || input.quality.is_some() {
        parameters.insert(
            "generationConfig".to_string(),
            serde_json::json!({
                "aspectRatio": input.aspect_ratio,
                "imageCount": input.image_count.unwrap_or(1),
                "quality": input.quality,
            }),
        );
    }
    if input.size.is_some() {
        parameters.insert("size".to_string(), serde_json::json!(input.size));
    }
    if let Some(seed) = input.seed {
        parameters.insert("seed".to_string(), serde_json::json!(seed));
    }
    if !input.reference_images.is_empty() {
        parameters.insert(
            "referenceImages".to_string(),
            serde_json::json!(input
                .reference_images
                .iter()
                .map(|url| serde_json::json!({ "url": url }))
                .collect::<Vec<_>>()),
        );
    }
    serde_json::Value::Object(parameters)
}

/// Builds the `parameters` payload for video creation tools.
pub fn video_parameters(input: &GenerateVideoInput) -> serde_json::Value {
    let mut parameters = serde_json::Map::new();
    if let Some(vendor) = input.vendor.as_deref() {
        parameters.insert("vendor".to_string(), serde_json::json!(vendor));
    }
    if input.duration_seconds.is_some()
        || input.aspect_ratio.is_some()
        || input.resolution.is_some()
    {
        parameters.insert(
            "generationConfig".to_string(),
            serde_json::json!({
                "durationSeconds": input.duration_seconds,
                "aspectRatio": input.aspect_ratio,
                "resolution": input.resolution,
            }),
        );
    }
    if !input.reference_images.is_empty() {
        parameters.insert(
            "referenceImages".to_string(),
            serde_json::json!(input
                .reference_images
                .iter()
                .map(|url| serde_json::json!({ "url": url }))
                .collect::<Vec<_>>()),
        );
    }
    if let Some(last_frame) = input.last_frame.as_deref() {
        // The command extractor reads `lastFrame` / `imageTail` as a bare URL
        // string; wrapping it in `{url}` would drop the tail frame before the
        // vendor adapter ever sees it.
        parameters.insert("lastFrame".to_string(), serde_json::json!(last_frame));
    }
    if let Some(seed) = input.seed {
        parameters.insert("seed".to_string(), serde_json::json!(seed));
    }
    if let Some(negative_prompt) = input.negative_prompt.as_deref() {
        parameters.insert("negativePrompt".to_string(), serde_json::json!(negative_prompt));
    }
    if let Some(mode) = input.mode.as_deref() {
        parameters.insert("mode".to_string(), serde_json::json!(mode));
    }
    if let Some(cfg_scale) = input.cfg_scale {
        parameters.insert("cfgScale".to_string(), serde_json::json!(cfg_scale));
    }
    serde_json::Value::Object(parameters)
}

/// Builds the `parameters` payload for speech synthesis tools.
pub fn speech_parameters(input: &SynthesizeSpeechInput) -> serde_json::Value {
    let mut parameters = serde_json::Map::new();
    if let Some(voice) = input.voice.as_deref() {
        parameters.insert("voice".to_string(), serde_json::json!(voice));
    }
    if let Some(format) = input.response_format.as_deref() {
        parameters.insert("responseFormat".to_string(), serde_json::json!(format));
    }
    if let Some(speed) = input.speed {
        parameters.insert("speed".to_string(), serde_json::json!(speed));
    }
    serde_json::Value::Object(parameters)
}

/// Builds the `parameters` payload for music creation tools.
pub fn music_parameters(input: &GenerateMusicInput) -> serde_json::Value {
    let mut parameters = serde_json::Map::new();
    // Vendor forwarding mirrors image_parameters: without it the generations
    // service resolves the modality default vendor (suno) and the LLM's
    // vendor/model selection is silently dropped.
    if let Some(vendor) = input.vendor.as_deref() {
        parameters.insert("vendor".to_string(), serde_json::json!(vendor));
    }
    if let Some(tags) = input.tags.as_deref() {
        parameters.insert("tags".to_string(), serde_json::json!(tags));
    }
    if let Some(title) = input.title.as_deref() {
        parameters.insert("title".to_string(), serde_json::json!(title));
    }
    if let Some(lyrics) = input.lyrics.as_deref() {
        parameters.insert("lyrics".to_string(), serde_json::json!(lyrics));
    }
    if let Some(duration) = input.duration_seconds {
        parameters.insert(
            "generationConfig".to_string(),
            serde_json::json!({ "durationSeconds": duration }),
        );
    }
    if let Some(negative_tags) = input.negative_tags.as_deref() {
        parameters.insert("negativeTags".to_string(), serde_json::json!(negative_tags));
    }
    if let Some(is_instrumental) = input.is_instrumental {
        parameters.insert("isInstrumental".to_string(), serde_json::json!(is_instrumental));
    }
    if let Some(lyrics_optimizer) = input.lyrics_optimizer {
        parameters.insert("lyricsOptimizer".to_string(), serde_json::json!(lyrics_optimizer));
    }
    serde_json::Value::Object(parameters)
}

/// Builds the `parameters` payload for the sound-effect tool.
pub fn sound_effect_parameters(input: &GenerateSoundEffectInput) -> serde_json::Value {
    let mut parameters = serde_json::Map::new();
    if let Some(vendor) = input.vendor.as_deref() {
        parameters.insert("vendor".to_string(), serde_json::json!(vendor));
    }
    if let Some(duration) = input.duration_seconds {
        parameters.insert(
            "generationConfig".to_string(),
            serde_json::json!({ "durationSeconds": duration }),
        );
    }
    if let Some(format) = input.response_format.as_deref() {
        parameters.insert("responseFormat".to_string(), serde_json::json!(format));
    }
    serde_json::Value::Object(parameters)
}

/// Parses a retrieve tool's arguments into the generation id.
pub fn retrieve_generation_id(input: &GenerationRetrieveInput) -> String {
    input.generation_id.clone()
}


#[cfg(test)]
mod insufficient_balance_tests {
    use super::is_insufficient_balance_failure;

    #[test]
    fn recognizes_the_current_402_contract() {
        assert!(is_insufficient_balance_failure(
            r#"generations api returned 402 Payment Required: {"code":40201,"detail":"insufficient available balance"}"#
        ));
        assert!(is_insufficient_balance_failure(
            "generations api returned 402: insufficient balance"
        ));
    }

    #[test]
    fn recognizes_the_legacy_precharge_shape() {
        assert!(is_insufficient_balance_failure(
            "generations api returned 503 Service Unavailable: {\"failedStage\":\"billing_precharge\"}"
        ));
    }

    #[test]
    fn rejects_unrelated_failures() {
        assert!(!is_insufficient_balance_failure(
            "generations api returned 500 Internal Server Error: boom"
        ));
        assert!(!is_insufficient_balance_failure(
            "generations api returned 404 Not Found: {\"code\":40401}"
        ));
        assert!(!is_insufficient_balance_failure(
            "generations create response missing data.item: {}"
        ));
        assert!(!is_insufficient_balance_failure("generations request failed: timeout"));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn endpoint_mapping_covers_the_default_tool_set() {
        assert_eq!(
            generation_endpoint("image", "text_to_image").as_deref(),
            Some("/app/v3/api/generations/images/text_to_image")
        );
        assert_eq!(
            generation_endpoint("voice", "speech").as_deref(),
            Some("/app/v3/api/generations/voice/speech")
        );
        assert_eq!(
            generation_endpoint("music", "text_to_music").as_deref(),
            Some("/app/v3/api/generations/music/text_to_music")
        );
        assert_eq!(
            generation_endpoint("sfx", "sound_effects").as_deref(),
            Some("/app/v3/api/generations/sound_effects")
        );
        assert_eq!(generation_endpoint("image", "unknown"), None);
    }

    #[test]
    fn image_parameters_carry_vendor_and_generation_config() {
        let input = GenerateImageInput {
            prompt: "a cat".to_string(),
            model: Some("gpt-image-2".to_string()),
            vendor: Some("openai".to_string()),
            aspect_ratio: Some("1:1".to_string()),
            image_count: Some(2),
            quality: Some("high".to_string()),
            size: None,
            seed: Some(7),
            reference_images: vec![],
            reference_asset_ids: vec![],
        };
        let parameters = image_parameters(&input);
        assert_eq!(parameters["vendor"], "openai");
        assert_eq!(parameters["generationConfig"]["imageCount"], 2);
        assert_eq!(parameters["seed"], 7);
    }

    #[test]
    fn video_parameters_carry_tail_frame_as_bare_url_and_seed() {
        let input = GenerateVideoInput {
            prompt: "waves".to_string(),
            model: Some("viduq2".to_string()),
            vendor: Some("vidu".to_string()),
            duration_seconds: Some(5),
            aspect_ratio: Some("16:9".to_string()),
            resolution: Some("1080p".to_string()),
            seed: Some(42),
            negative_prompt: Some("no text overlays".to_string()),
            mode: Some("pro".to_string()),
            cfg_scale: Some(0.75),
            reference_images: vec!["https://cdn.example/start.png".to_string()],
            reference_asset_ids: vec![],
            last_frame: Some("https://cdn.example/end.png".to_string()),
        };
        let parameters = video_parameters(&input);
        // The command extractor reads `lastFrame` as a bare URL string; an
        // object wrapper would silently drop the tail frame.
        assert_eq!(parameters["lastFrame"], "https://cdn.example/end.png");
        assert_eq!(parameters["seed"], 42);
        assert_eq!(parameters["negativePrompt"], "no text overlays");
        assert_eq!(parameters["mode"], "pro");
        assert_eq!(parameters["cfgScale"], 0.75);
        assert_eq!(
            parameters["generationConfig"]["durationSeconds"], 5,
            "durationSeconds must survive the builder"
        );
    }

    #[test]
    fn music_parameters_carry_instrumental_and_lyrics_flags() {
        let input = GenerateMusicInput {
            prompt: "a bright piano loop".to_string(),
            vendor: Some("minimax".to_string()),
            tags: None,
            title: None,
            lyrics: None,
            duration_seconds: None,
            negative_tags: None,
            is_instrumental: Some(true),
            lyrics_optimizer: Some(false),
            model: Some("music-cover".to_string()),
        };
        let parameters = music_parameters(&input);
        assert_eq!(parameters["isInstrumental"], true);
        assert_eq!(parameters["lyricsOptimizer"], false);
        assert_eq!(parameters["vendor"], "minimax");
    }

    #[test]
    fn sound_effect_parameters_carry_duration_and_format() {
        let input = GenerateSoundEffectInput {
            prompt: "thunder over a metal roof".to_string(),
            vendor: None,
            model: Some("eleven_text_to_sound_v2".to_string()),
            duration_seconds: Some(6.0),
            response_format: Some("wav".to_string()),
        };
        let parameters = sound_effect_parameters(&input);
        assert_eq!(parameters["generationConfig"]["durationSeconds"], 6.0);
        assert_eq!(parameters["responseFormat"], "wav");
    }
}

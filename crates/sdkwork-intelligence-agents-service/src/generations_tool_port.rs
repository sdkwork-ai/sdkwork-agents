//! HTTP-backed generations tool port for the turn loop.
//!
//! The generations MCP tools execute against the federated generations app
//! API mounted on the cloudrouter gateway (`/app/v3/api/generations/*`),
//! authenticating with the caller's dual tokens so tenant scope, billing, and
//! durable generation records resolve server-side. Keeping the port HTTP
//! preserves the service boundary: the agents process never embeds the
//! generations database or provider adapters.

use std::time::Duration;

use reqwest::blocking::Client;

use sdkwork_generations_mcp_service::{
    GenerateImageInput, GenerateMusicInput, GenerateVideoInput, GenerationRetrieveInput,
    SynthesizeSpeechInput,
};

const GENERATIONS_CONNECT_TIMEOUT: Duration = Duration::from_secs(10);
const GENERATIONS_REQUEST_TIMEOUT: Duration = Duration::from_secs(300);

/// Endpoint paths of the federated generations app API, keyed by
/// `(modality, operation)`.
fn generation_endpoint(modality: &str, operation: &str) -> Option<String> {
    match (modality, operation) {
        ("image", "text_to_image") => Some("/app/v3/api/generations/images/text_to_image".to_string()),
        ("image", "image_edit") => Some("/app/v3/api/generations/images/image_edit".to_string()),
        ("video", "text_to_video") => Some("/app/v3/api/generations/videos/text_to_video".to_string()),
        ("video", "image_to_video") => Some("/app/v3/api/generations/videos/image_to_video".to_string()),
        ("video", "video_extend") => Some("/app/v3/api/generations/videos/video_extend".to_string()),
        ("music", "text_to_music") => Some("/app/v3/api/generations/music/text_to_music".to_string()),
        ("music", "lyrics_to_music") => Some("/app/v3/api/generations/music/lyrics_to_music".to_string()),
        ("voice", "speech") => Some("/app/v3/api/generations/voice/speech".to_string()),
        _ => None,
    }
}

/// Blocking HTTP port for the generations app API.
#[derive(Debug, Clone)]
pub struct HttpGenerationsPort {
    base_url: String,
    client: Client,
}

impl HttpGenerationsPort {
    pub fn new(base_url: impl Into<String>) -> Self {
        let client = Client::builder()
            .connect_timeout(GENERATIONS_CONNECT_TIMEOUT)
            .timeout(GENERATIONS_REQUEST_TIMEOUT)
            .build()
            .expect("generations http client");
        Self {
            base_url: base_url.into(),
            client,
        }
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
        let mut body = serde_json::json!({
            "tenantId": tenant_id.to_string(),
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
        let mut request = self
            .client
            .post(&url)
            .bearer_auth(auth_token)
            .json(&body);
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
        let mut request = self.client.get(&url).bearer_auth(auth_token);
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
        parameters.insert(
            "lastFrame".to_string(),
            serde_json::json!({ "url": last_frame }),
        );
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
    serde_json::Value::Object(parameters)
}

/// Parses a retrieve tool's arguments into the generation id.
pub fn retrieve_generation_id(input: &GenerationRetrieveInput) -> String {
    input.generation_id.clone()
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
            reference_images: vec![],
        };
        let parameters = image_parameters(&input);
        assert_eq!(parameters["vendor"], "openai");
        assert_eq!(parameters["generationConfig"]["imageCount"], 2);
    }
}

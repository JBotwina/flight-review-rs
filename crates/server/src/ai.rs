//! OpenRouter-backed flight-log analysis.
//!
//! The model never receives the raw ULog or GPS track. Instead, it receives a
//! bounded, privacy-conscious projection of the deterministic Rust analysis:
//! flight statistics, diagnostics, field summaries, flight modes, and log
//! messages. The resulting JSON is stored beside the log as `ai-analysis.json`.

use chrono::{DateTime, Utc};
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

const ANALYSIS_SCHEMA_VERSION: u8 = 1;
const DEFAULT_BASE_URL: &str = "https://openrouter.ai/api/v1";

#[derive(Debug, thiserror::Error)]
pub enum AiError {
    #[error("AI analysis is not configured; set OPENROUTER_API_KEY")]
    NotConfigured,
    #[error("invalid OpenRouter model id")]
    InvalidModel,
    #[error("OpenRouter request failed: {0}")]
    Request(#[from] reqwest::Error),
    #[error("OpenRouter returned {status}: {message}")]
    Provider { status: u16, message: String },
    #[error("OpenRouter returned no assistant content")]
    MissingContent,
    #[error("model returned invalid analysis JSON: {0}")]
    InvalidResponse(String),
}

#[derive(Debug, Clone)]
pub struct OpenRouterClient {
    api_key: String,
    base_url: String,
    default_model: String,
    site_url: Option<String>,
    app_name: String,
    http: reqwest::Client,
}

impl OpenRouterClient {
    pub fn from_env(http: reqwest::Client) -> Option<Self> {
        let api_key = std::env::var("OPENROUTER_API_KEY")
            .ok()
            .filter(|key| !key.trim().is_empty())?;
        Some(Self {
            api_key,
            base_url: std::env::var("OPENROUTER_BASE_URL")
                .unwrap_or_else(|_| DEFAULT_BASE_URL.to_string())
                .trim_end_matches('/')
                .to_string(),
            default_model: std::env::var("OPENROUTER_DEFAULT_MODEL")
                .unwrap_or_else(|_| "openrouter/auto".to_string()),
            site_url: std::env::var("OPENROUTER_SITE_URL")
                .ok()
                .filter(|url| !url.trim().is_empty())
                .or_else(|| {
                    std::env::var("RAILWAY_PUBLIC_DOMAIN")
                        .ok()
                        .filter(|domain| !domain.trim().is_empty())
                        .map(|domain| format!("https://{domain}"))
                }),
            app_name: std::env::var("OPENROUTER_APP_NAME")
                .unwrap_or_else(|_| "PX4 Flight Review".to_string()),
            http,
        })
    }

    #[cfg(test)]
    fn for_test() -> Self {
        Self {
            api_key: "test-key".into(),
            base_url: "https://example.invalid".into(),
            default_model: "openrouter/auto".into(),
            site_url: None,
            app_name: "test".into(),
            http: reqwest::Client::new(),
        }
    }

    pub fn default_model(&self) -> &str {
        &self.default_model
    }

    fn request(&self, builder: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
        let mut builder = builder
            .header(AUTHORIZATION, format!("Bearer {}", self.api_key))
            .header(CONTENT_TYPE, "application/json")
            .header("X-Title", &self.app_name);
        if let Some(url) = &self.site_url {
            builder = builder.header("HTTP-Referer", url);
        }
        builder
    }

    pub async fn list_models(&self) -> Result<Vec<AiModel>, AiError> {
        let response = self
            .request(self.http.get(format!("{}/models/user", self.base_url)))
            .send()
            .await?;
        let status = response.status();
        let body = response.text().await?;
        if !status.is_success() {
            return Err(provider_error(status.as_u16(), &body));
        }

        let envelope: ModelsEnvelope = serde_json::from_str(&body)
            .map_err(|e| AiError::InvalidResponse(format!("models response: {e}")))?;
        let mut models: Vec<AiModel> = envelope
            .data
            .into_iter()
            .filter(|model| {
                let text_output = model
                    .architecture
                    .as_ref()
                    .is_none_or(|a| a.output_modalities.iter().any(|m| m == "text"));
                let structured = model
                    .supported_parameters
                    .iter()
                    .any(|parameter| parameter == "structured_outputs");
                text_output && structured
            })
            .map(|model| AiModel {
                id: model.id,
                name: model.name,
                description: model.description,
                context_length: model.context_length,
                pricing: model.pricing,
            })
            .collect();
        models.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
        Ok(models)
    }

    pub async fn key_balance(&self) -> Result<AiBalanceResponse, AiError> {
        let response = self
            .request(self.http.get(format!("{}/key", self.base_url)))
            .send()
            .await?;
        let status = response.status();
        let body = response.text().await?;
        if !status.is_success() {
            return Err(provider_error(status.as_u16(), &body));
        }

        let envelope: KeyEnvelope = serde_json::from_str(&body)
            .map_err(|e| AiError::InvalidResponse(format!("key balance response: {e}")))?;
        Ok(AiBalanceResponse {
            enabled: true,
            limit: envelope.data.limit,
            limit_remaining: envelope.data.limit_remaining,
            limit_reset: envelope.data.limit_reset,
            usage: envelope.data.usage,
            usage_daily: envelope.data.usage_daily,
            usage_weekly: envelope.data.usage_weekly,
            usage_monthly: envelope.data.usage_monthly,
            is_free_tier: envelope.data.is_free_tier,
        })
    }

    pub async fn analyze(&self, model: &str, metadata: &Value) -> Result<AiAnalysis, AiError> {
        validate_model_id(model)?;
        let input = build_analysis_input(metadata);
        let input_json = serde_json::to_string_pretty(&input)
            .map_err(|e| AiError::InvalidResponse(e.to_string()))?;

        let request = json!({
            "model": model,
            "messages": [
                {
                    "role": "system",
                    "content": SYSTEM_PROMPT
                },
                {
                    "role": "user",
                    "content": format!("Analyze this PX4 flight-log evidence. Return only the requested JSON object.\n\n{input_json}")
                }
            ],
            "response_format": {
                "type": "json_schema",
                "json_schema": {
                    "name": "px4_flight_analysis",
                    "strict": true,
                    "schema": analysis_response_schema()
                }
            },
            "temperature": 0.15,
            "max_tokens": 3500
        });

        let response = self
            .request(
                self.http
                    .post(format!("{}/chat/completions", self.base_url)),
            )
            .json(&request)
            .send()
            .await?;
        let status = response.status();
        let body = response.text().await?;
        if !status.is_success() {
            return Err(provider_error(status.as_u16(), &body));
        }

        let envelope: ChatEnvelope = serde_json::from_str(&body)
            .map_err(|e| AiError::InvalidResponse(format!("completion envelope: {e}")))?;
        if let Some(error) = envelope.error {
            return Err(AiError::Provider {
                status: 502,
                message: error.message,
            });
        }
        let content = envelope
            .choices
            .first()
            .and_then(|choice| choice.message.as_ref())
            .and_then(|message| message.content.as_deref())
            .ok_or(AiError::MissingContent)?;
        let mut draft = parse_analysis_content(content)?;
        normalize_analysis(&mut draft);

        Ok(AiAnalysis {
            schema_version: ANALYSIS_SCHEMA_VERSION,
            generated_at: Utc::now(),
            requested_model: model.to_string(),
            model: envelope.model.unwrap_or_else(|| model.to_string()),
            summary: draft.summary,
            risk_level: draft.risk_level,
            confidence: draft.confidence.map(|v| v.clamp(0.0, 1.0)),
            findings: draft.findings,
            positive_observations: draft.positive_observations,
            recommendations: draft.recommendations,
            limitations: draft.limitations,
            usage: envelope.usage,
        })
    }
}

fn provider_error(status: u16, body: &str) -> AiError {
    let message = serde_json::from_str::<Value>(body)
        .ok()
        .and_then(|value| {
            value
                .pointer("/error/message")
                .and_then(Value::as_str)
                .map(str::to_string)
        })
        .unwrap_or_else(|| body.chars().take(500).collect());
    AiError::Provider { status, message }
}

pub fn validate_model_id(model: &str) -> Result<(), AiError> {
    let valid = !model.is_empty()
        && model.len() <= 200
        && model.contains('/')
        && model
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '/' | '-' | '_' | '.' | ':'));
    if valid {
        Ok(())
    } else {
        Err(AiError::InvalidModel)
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct AiModelsResponse {
    pub enabled: bool,
    pub default_model: Option<String>,
    pub models: Vec<AiModel>,
}

#[derive(Debug, Clone, Serialize)]
pub struct AiBalanceResponse {
    pub enabled: bool,
    pub limit: Option<f64>,
    pub limit_remaining: Option<f64>,
    pub limit_reset: Option<String>,
    pub usage: f64,
    pub usage_daily: f64,
    pub usage_weekly: f64,
    pub usage_monthly: f64,
    pub is_free_tier: bool,
}

impl AiBalanceResponse {
    pub fn disabled() -> Self {
        Self {
            enabled: false,
            limit: None,
            limit_remaining: None,
            limit_reset: None,
            usage: 0.0,
            usage_daily: 0.0,
            usage_weekly: 0.0,
            usage_monthly: 0.0,
            is_free_tier: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiModel {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub context_length: Option<u64>,
    #[serde(default)]
    pub pricing: Option<ModelPricing>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelPricing {
    #[serde(default)]
    pub prompt: Option<String>,
    #[serde(default)]
    pub completion: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ModelsEnvelope {
    data: Vec<OpenRouterModel>,
}

#[derive(Debug, Deserialize)]
struct KeyEnvelope {
    data: OpenRouterKey,
}

#[derive(Debug, Deserialize)]
struct OpenRouterKey {
    #[serde(default)]
    limit: Option<f64>,
    #[serde(default)]
    limit_remaining: Option<f64>,
    #[serde(default)]
    limit_reset: Option<String>,
    #[serde(default)]
    usage: f64,
    #[serde(default)]
    usage_daily: f64,
    #[serde(default)]
    usage_weekly: f64,
    #[serde(default)]
    usage_monthly: f64,
    #[serde(default)]
    is_free_tier: bool,
}

#[derive(Debug, Deserialize)]
struct OpenRouterModel {
    id: String,
    name: String,
    #[serde(default)]
    description: Option<String>,
    #[serde(default)]
    context_length: Option<u64>,
    #[serde(default)]
    pricing: Option<ModelPricing>,
    #[serde(default)]
    architecture: Option<ModelArchitecture>,
    #[serde(default)]
    supported_parameters: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct ModelArchitecture {
    #[serde(default)]
    output_modalities: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiAnalysis {
    pub schema_version: u8,
    pub generated_at: DateTime<Utc>,
    pub requested_model: String,
    pub model: String,
    pub summary: String,
    pub risk_level: RiskLevel,
    pub confidence: Option<f32>,
    pub findings: Vec<AiFinding>,
    pub positive_observations: Vec<String>,
    pub recommendations: Vec<AiRecommendation>,
    pub limitations: Vec<String>,
    pub usage: Option<AiUsage>,
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RiskLevel {
    Low,
    Moderate,
    High,
    Critical,
    #[default]
    #[serde(other)]
    Unknown,
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FindingSeverity {
    #[default]
    Info,
    Warning,
    Critical,
    #[serde(other)]
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AiFinding {
    #[serde(default)]
    pub category: String,
    #[serde(default)]
    pub severity: FindingSeverity,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub explanation: String,
    #[serde(default)]
    pub evidence: Vec<String>,
    #[serde(default)]
    pub time_range_s: Option<TimeRange>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeRange {
    pub start: f64,
    #[serde(default)]
    pub end: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AiRecommendation {
    #[serde(default)]
    pub priority: RecommendationPriority,
    #[serde(default)]
    pub action: String,
    #[serde(default)]
    pub rationale: String,
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RecommendationPriority {
    High,
    #[default]
    Medium,
    Low,
    #[serde(other)]
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiUsage {
    #[serde(default)]
    pub prompt_tokens: Option<u64>,
    #[serde(default)]
    pub completion_tokens: Option<u64>,
    #[serde(default)]
    pub total_tokens: Option<u64>,
    #[serde(default)]
    pub cost: Option<f64>,
}

#[derive(Debug, Deserialize)]
struct ChatEnvelope {
    #[serde(default)]
    model: Option<String>,
    #[serde(default)]
    choices: Vec<ChatChoice>,
    #[serde(default)]
    usage: Option<AiUsage>,
    #[serde(default)]
    error: Option<ChatError>,
}

#[derive(Debug, Deserialize)]
struct ChatChoice {
    #[serde(default)]
    message: Option<ChatMessage>,
}

#[derive(Debug, Deserialize)]
struct ChatMessage {
    #[serde(default)]
    content: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ChatError {
    message: String,
}

#[derive(Debug, Deserialize, Default)]
struct AiAnalysisDraft {
    #[serde(default)]
    summary: String,
    #[serde(default)]
    risk_level: RiskLevel,
    #[serde(default)]
    confidence: Option<f32>,
    #[serde(default)]
    findings: Vec<AiFinding>,
    #[serde(default)]
    positive_observations: Vec<String>,
    #[serde(default)]
    recommendations: Vec<AiRecommendation>,
    #[serde(default)]
    limitations: Vec<String>,
}

fn parse_analysis_content(content: &str) -> Result<AiAnalysisDraft, AiError> {
    let trimmed = content.trim();
    let unwrapped = trimmed
        .strip_prefix("```json")
        .or_else(|| trimmed.strip_prefix("```"))
        .unwrap_or(trimmed)
        .strip_suffix("```")
        .unwrap_or(trimmed)
        .trim();
    let draft: AiAnalysisDraft =
        serde_json::from_str(unwrapped).map_err(|e| AiError::InvalidResponse(e.to_string()))?;
    if draft.summary.trim().is_empty() {
        return Err(AiError::InvalidResponse(
            "response did not include a summary".to_string(),
        ));
    }
    Ok(draft)
}

fn normalize_analysis(draft: &mut AiAnalysisDraft) {
    draft.summary.truncate(4_000);
    draft.findings.truncate(12);
    for finding in &mut draft.findings {
        finding.title.truncate(240);
        finding.explanation.truncate(2_000);
        finding.evidence.truncate(6);
        for evidence in &mut finding.evidence {
            evidence.truncate(500);
        }
    }
    draft.positive_observations.truncate(8);
    draft.recommendations.truncate(8);
    draft.limitations.truncate(8);
}

fn analysis_response_schema() -> Value {
    json!({
        "type": "object",
        "additionalProperties": false,
        "properties": {
            "summary": { "type": "string" },
            "risk_level": { "type": "string", "enum": ["low", "moderate", "high", "critical", "unknown"] },
            "confidence": { "type": ["number", "null"], "minimum": 0, "maximum": 1 },
            "findings": {
                "type": "array",
                "items": {
                    "type": "object",
                    "additionalProperties": false,
                    "properties": {
                        "category": { "type": "string" },
                        "severity": { "type": "string", "enum": ["info", "warning", "critical"] },
                        "title": { "type": "string" },
                        "explanation": { "type": "string" },
                        "evidence": { "type": "array", "items": { "type": "string" } },
                        "time_range_s": {
                            "anyOf": [
                                { "type": "null" },
                                {
                                    "type": "object",
                                    "additionalProperties": false,
                                    "properties": {
                                        "start": { "type": "number" },
                                        "end": { "type": ["number", "null"] }
                                    },
                                    "required": ["start", "end"]
                                }
                            ]
                        }
                    },
                    "required": ["category", "severity", "title", "explanation", "evidence", "time_range_s"]
                }
            },
            "positive_observations": { "type": "array", "items": { "type": "string" } },
            "recommendations": {
                "type": "array",
                "items": {
                    "type": "object",
                    "additionalProperties": false,
                    "properties": {
                        "priority": { "type": "string", "enum": ["high", "medium", "low"] },
                        "action": { "type": "string" },
                        "rationale": { "type": "string" }
                    },
                    "required": ["priority", "action", "rationale"]
                }
            },
            "limitations": { "type": "array", "items": { "type": "string" } }
        },
        "required": [
            "summary", "risk_level", "confidence", "findings",
            "positive_observations", "recommendations", "limitations"
        ]
    })
}

fn value_at(metadata: &Value, pointer: &str) -> Value {
    metadata.pointer(pointer).cloned().unwrap_or(Value::Null)
}

fn limited_array(metadata: &Value, pointer: &str, limit: usize) -> Value {
    Value::Array(
        metadata
            .pointer(pointer)
            .and_then(Value::as_array)
            .map(|items| items.iter().take(limit).cloned().collect())
            .unwrap_or_default(),
    )
}

/// Build the evidence packet sent to OpenRouter. Deliberately excludes the GPS
/// track, raw parameters, vehicle UUID, exact location, and raw topic samples.
pub fn build_analysis_input(metadata: &Value) -> Value {
    let mut topics: Vec<Value> = metadata
        .get("topics")
        .and_then(Value::as_object)
        .map(|topics| {
            topics
                .iter()
                .map(|(name, info)| {
                    json!({
                        "name": name,
                        "message_count": info.get("message_count").cloned().unwrap_or(Value::Null)
                    })
                })
                .collect()
        })
        .unwrap_or_default();
    topics.sort_by(|a, b| a["name"].as_str().cmp(&b["name"].as_str()));
    topics.truncate(160);

    json!({
        "vehicle": {
            "system": value_at(metadata, "/sys_name"),
            "hardware": value_at(metadata, "/ver_hw"),
            "firmware": value_at(metadata, "/ver_sw_release_str"),
        },
        "log": {
            "duration_s": value_at(metadata, "/flight_duration_s"),
            "completeness": value_at(metadata, "/completeness"),
            "dropout_count": value_at(metadata, "/dropout_count"),
            "dropout_total_ms": value_at(metadata, "/dropout_total_ms"),
            "topics": topics,
        },
        "flight_modes": limited_array(metadata, "/analysis/flight_modes", 80),
        "vtol_states": limited_array(metadata, "/analysis/vtol_states", 80),
        "flight_stats": value_at(metadata, "/analysis/stats"),
        "battery": value_at(metadata, "/analysis/battery"),
        "gps_quality": value_at(metadata, "/analysis/gps_quality"),
        "vibration": value_at(metadata, "/analysis/vibration"),
        "deterministic_diagnostics": limited_array(metadata, "/analysis/diagnostics", 100),
        "field_statistics": limited_array(metadata, "/analysis/field_stats", 300),
        "non_default_parameters": limited_array(metadata, "/analysis/non_default_params", 80),
        "parameters_changed_in_flight": limited_array(metadata, "/changed_parameters", 80),
        "logged_messages": limited_array(metadata, "/logged_messages", 120),
        "tagged_logged_messages": limited_array(metadata, "/tagged_logged_messages", 80),
    })
}

const SYSTEM_PROMPT: &str = r#"You are a conservative PX4 flight-log analyst assisting an engineer or pilot.

Use only the supplied evidence. Never invent a measurement, event, causal explanation, or timestamp. Treat deterministic diagnostics as strong signals but explain their evidence. Distinguish observed facts from plausible interpretations. A clean result means "no issue was detected in the supplied signals", not proof of airworthiness. Do not give regulatory, legal, or safety certification advice. Never expose or infer a precise location.

Choose the overall risk_level from: low, moderate, high, critical, unknown. Use unknown when evidence is too incomplete. Findings must prioritize anomaly detection, flight-control health, estimator/GPS health, power, vibration, logging quality, and notable performance. Include positive observations when supported. Recommendations should be concrete follow-up checks, not generic filler.

Return exactly one JSON object with this shape:
{
  "summary": "concise 2-4 sentence flight debrief",
  "risk_level": "low|moderate|high|critical|unknown",
  "confidence": 0.0,
  "findings": [
    {
      "category": "power|navigation|control|propulsion|vibration|logging|performance|other",
      "severity": "info|warning|critical",
      "title": "short title",
      "explanation": "what was observed and why it matters, with uncertainty",
      "evidence": ["specific supplied measurements or messages"],
      "time_range_s": {"start": 0.0, "end": 1.0}
    }
  ],
  "positive_observations": ["specific evidence-backed observation"],
  "recommendations": [
    {"priority": "high|medium|low", "action": "specific action", "rationale": "evidence-based reason"}
  ],
  "limitations": ["important unavailable signal or analysis limitation"]
}

Set time_range_s to null when no trustworthy time mapping is available. Keep evidence strings compact. Return valid JSON and no markdown."#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_model_ids() {
        assert!(validate_model_id("anthropic/claude-sonnet-4").is_ok());
        assert!(validate_model_id("openai/gpt-5:online").is_ok());
        assert!(validate_model_id("missing-slash").is_err());
        assert!(validate_model_id("openai/gpt 5").is_err());
    }

    #[test]
    fn input_excludes_sensitive_and_large_fields() {
        let metadata = json!({
            "sys_uuid": "secret-id",
            "gps_first_fix": {"lat_deg": 1.0, "lon_deg": 2.0},
            "parameters": {"A": 1},
            "analysis": {"gps_track": [{"lat_deg": 1.0}], "stats": {"max_speed_m_s": 4.0}},
            "topics": {"vehicle_status": {"message_count": 10}}
        });
        let packet = build_analysis_input(&metadata);
        let text = serde_json::to_string(&packet).unwrap();
        assert!(!text.contains("secret-id"));
        assert!(!text.contains("lat_deg"));
        assert!(!text.contains("\"parameters\""));
        assert_eq!(packet["flight_stats"]["max_speed_m_s"], 4.0);
    }

    #[test]
    fn parses_fenced_json_for_less_strict_models() {
        let draft = parse_analysis_content(
            "```json\n{\"summary\":\"Nominal flight.\",\"risk_level\":\"low\"}\n```",
        )
        .unwrap();
        assert_eq!(draft.summary, "Nominal flight.");
    }

    #[test]
    fn test_client_has_expected_default() {
        assert_eq!(
            OpenRouterClient::for_test().default_model(),
            "openrouter/auto"
        );
    }

    #[test]
    fn parses_key_balance_without_retaining_key_identity() {
        let envelope: KeyEnvelope = serde_json::from_value(json!({
            "data": {
                "label": "redacted-label",
                "limit": 10.0,
                "limit_remaining": 9.25,
                "limit_reset": null,
                "usage": 0.75,
                "usage_daily": 0.25,
                "usage_weekly": 0.5,
                "usage_monthly": 0.75,
                "is_free_tier": false
            }
        }))
        .unwrap();

        assert_eq!(envelope.data.limit_remaining, Some(9.25));
        assert_eq!(envelope.data.usage_daily, 0.25);
    }
}

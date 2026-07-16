use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ProviderProtocol {
    #[serde(rename = "openai_compatible")]
    OpenAiCompatible,
    #[serde(rename = "anthropic_compatible")]
    AnthropicCompatible,
    #[serde(rename = "codex_experimental")]
    CodexExperimental,
}

impl ProviderProtocol {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::OpenAiCompatible => "openai_compatible",
            Self::AnthropicCompatible => "anthropic_compatible",
            Self::CodexExperimental => "codex_experimental",
        }
    }
}

impl std::str::FromStr for ProviderProtocol {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "openai_compatible" => Ok(Self::OpenAiCompatible),
            "anthropic_compatible" => Ok(Self::AnthropicCompatible),
            "codex_experimental" => Ok(Self::CodexExperimental),
            _ => Err(format!("Unsupported provider protocol: {value}")),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Provider {
    #[serde(default)]
    pub id: String,
    pub name: String,
    pub base_url: String,
    pub protocol: ProviderProtocol,
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default)]
    pub default_model: String,
    #[serde(default)]
    pub models: Vec<String>,
    #[serde(default)]
    pub model_prefixes: Vec<String>,
    #[serde(default = "default_timeout_ms")]
    pub timeout_ms: i64,
    #[serde(default = "default_max_tokens_field")]
    pub max_tokens_field: String,
    #[serde(default)]
    pub secret_ref: Option<String>,
    #[serde(default)]
    pub has_api_key: bool,
    #[serde(default)]
    pub health_status: String,
    #[serde(default)]
    pub created_at: i64,
    #[serde(default)]
    pub updated_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderInput {
    pub name: String,
    pub base_url: String,
    pub protocol: ProviderProtocol,
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default)]
    pub default_model: String,
    #[serde(default)]
    pub models: Vec<String>,
    #[serde(default)]
    pub model_prefixes: Vec<String>,
    #[serde(default = "default_timeout_ms")]
    pub timeout_ms: i64,
    #[serde(default = "default_max_tokens_field")]
    pub max_tokens_field: String,
    #[serde(default)]
    pub api_key: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GatewayLog {
    pub id: i64,
    pub time: i64,
    pub client_type: String,
    pub endpoint: String,
    pub requested_model: String,
    pub resolved_provider: Option<String>,
    pub resolved_model: Option<String>,
    pub protocol_in: String,
    pub protocol_out: Option<String>,
    pub status_code: i64,
    pub latency_ms: i64,
    pub stream: bool,
    pub token_estimate: Option<i64>,
    pub input_tokens: Option<i64>,
    pub output_tokens: Option<i64>,
    pub total_tokens: Option<i64>,
    pub cache_creation_input_tokens: Option<i64>,
    pub cache_read_input_tokens: Option<i64>,
    pub fidelity_mode: String,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewGatewayLog {
    pub client_type: String,
    pub endpoint: String,
    pub requested_model: String,
    pub resolved_provider: Option<String>,
    pub resolved_model: Option<String>,
    pub protocol_in: String,
    pub protocol_out: Option<String>,
    pub status_code: i64,
    pub latency_ms: i64,
    pub stream: bool,
    pub token_estimate: Option<i64>,
    pub input_tokens: Option<i64>,
    pub output_tokens: Option<i64>,
    pub total_tokens: Option<i64>,
    pub cache_creation_input_tokens: Option<i64>,
    pub cache_read_input_tokens: Option<i64>,
    pub fidelity_mode: String,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GatewayStatus {
    pub running: bool,
    pub host: String,
    pub port: u16,
    pub openai_base_url: String,
    pub anthropic_base_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKeyStatus {
    pub exists: bool,
    pub preview: Option<String>,
}

fn default_true() -> bool {
    true
}

fn default_timeout_ms() -> i64 {
    60_000
}

fn default_max_tokens_field() -> String {
    "max_tokens".to_string()
}

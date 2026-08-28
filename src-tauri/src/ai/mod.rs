pub mod groq;
pub mod prompt_export;
pub mod validator;

pub use validator::{AICssRule, AIPreviewResponse, CssValidator};


use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParentContext {
    pub tag: String,
    #[serde(default)]
    pub id: Option<String>,
    #[serde(default)]
    pub classes: Vec<String>,
    #[serde(default)]
    pub layout_styles: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElementContext {
    pub tag: String,
    #[serde(default)]
    pub id: Option<String>,
    #[serde(default)]
    pub classes: Vec<String>,
    #[serde(default)]
    pub attributes: HashMap<String, String>,
    #[serde(default)]
    pub text: Option<String>,
    pub selector: String,
    #[serde(default)]
    pub computed_styles: HashMap<String, String>,
    #[serde(default)]
    pub parent_context: Option<ParentContext>,
    #[serde(default)]
    pub grandparent_context: Option<ParentContext>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIPreviewRequest {
    pub instruction: String,
    pub element_context: ElementContext,
    #[serde(default)]
    pub current_preview_css: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIExportRequest {
    pub instruction: String,
    pub element_context: ElementContext,
    pub applied_css_rules: Vec<AICssRule>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AISettings {
    pub provider: String,
    pub api_key: String,
    pub model: String,
}

impl Default for AISettings {
    fn default() -> Self {
        Self {
            provider: "groq".to_string(),
            api_key: String::new(),
            model: "openai/gpt-oss-120b".to_string(),
        }
    }
}

#[async_trait::async_trait]
pub trait AIProvider: Send + Sync {
    async fn generate_preview(&self, req: &AIPreviewRequest) -> Result<AIPreviewResponse, String>;
    async fn export_prompt(&self, req: &AIExportRequest) -> Result<String, String>;
}

pub fn create_ai_provider(settings: &AISettings) -> Box<dyn AIProvider> {
    match settings.provider.to_lowercase().as_str() {
        "groq" | _ => Box::new(groq::GroqProvider::new(
            settings.api_key.clone(),
            Some(settings.model.clone()),
        )),
    }
}


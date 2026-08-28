use super::validator::CssValidator;
use super::{AIExportRequest, AIPreviewRequest, AIPreviewResponse, AIProvider};

use serde::{Deserialize, Serialize};
use serde_json::json;
use std::time::Duration;

pub struct GroqProvider {
    api_key: String,
    model: String,
    client: reqwest::Client,
}

#[derive(Deserialize, Debug)]
struct ChatCompletionResponse {
    choices: Vec<ChatChoice>,
}

#[derive(Deserialize, Debug)]
struct ChatChoice {
    message: ChatMessage,
}

#[derive(Deserialize, Serialize, Debug)]
struct ChatMessage {
    role: String,
    #[serde(default)]
    content: Option<String>,
    #[serde(default)]
    reasoning_content: Option<String>,
    #[serde(default)]
    reasoning: Option<String>,
}

impl GroqProvider {
    pub fn new(api_key: String, model: Option<String>) -> Self {
        let selected_model = model
            .filter(|m| !m.trim().is_empty())
            .unwrap_or_else(|| "openai/gpt-oss-120b".to_string());

        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(25))
            .build()
            .unwrap_or_default();

        Self {
            api_key,
            model: selected_model,
            client,
        }
    }

    fn build_preview_system_prompt(&self, selector: &str) -> String {
        format!(
            r#"You are an expert CSS & UI/UX design assistant inside Halka Browser's temporary visual sandbox.
Your task is to generate precise CSS rules to visually modify a selected element based on the user's design instruction.

CRITICAL CONSTRAINTS:
1. ONLY return visual CSS properties. NEVER output JavaScript, HTML tags, or external urls.
2. Target the element using its selector "{selector}" or child/adjacent selectors where relevant.
3. Be subtle, modern, and preserve responsive visual integrity.
4. Output STRICT JSON adhering exactly to this schema:
{{
  "summary": "Brief description of the visual changes",
  "css_rules": [
    {{
      "selector": "{selector}",
      "properties": {{
        "css-property": "value"
      }}
    }}
  ],
  "explanation": "Brief design rationale"
}}
DO NOT include markdown backticks or explanations outside the JSON."#
        )
    }

    fn build_preview_user_prompt(&self, req: &AIPreviewRequest) -> String {
        let el = &req.element_context;
        json!({
            "user_instruction": req.instruction,
            "element": {
                "tag": el.tag,
                "id": el.id,
                "classes": el.classes,
                "text": el.text,
                "attributes": el.attributes,
                "selector": el.selector,
            },
            "current_computed_styles": el.computed_styles,
            "parent_layout_context": el.parent_context,
            "grandparent_layout_context": el.grandparent_context,
            "existing_preview_css": req.current_preview_css
        })
        .to_string()
    }

    /// Extracts the JSON object substring { ... } from text
    fn extract_json_slice(text: &str) -> &str {
        let trimmed = text.trim();
        if let (Some(start), Some(end)) = (trimmed.find('{'), trimmed.rfind('}')) {
            if start <= end {
                return &trimmed[start..=end];
            }
        }
        trimmed
    }
}

#[async_trait::async_trait]
impl AIProvider for GroqProvider {
    async fn generate_preview(&self, req: &AIPreviewRequest) -> Result<AIPreviewResponse, String> {
        if self.api_key.trim().is_empty() {
            return Err("Groq API key is not configured. Please set your API key in AI Settings.".to_string());
        }

        let system_prompt = self.build_preview_system_prompt(&req.element_context.selector);
        let user_prompt = self.build_preview_user_prompt(req);

        let payload = json!({
            "model": self.model,
            "temperature": 0.2,
            "max_tokens": 2048,
            "messages": [
                { "role": "system", "content": system_prompt },
                { "role": "user", "content": format!("{}\n\nPlease output valid JSON adhering strictly to the schema.", user_prompt) }
            ]
        });

        let response = self
            .client
            .post("https://api.groq.com/openai/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", self.api_key.trim()))
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await
            .map_err(|e| format!("Network error communicating with Groq: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_body = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(format!("Groq API Error (Status {}): {}", status, error_body));
        }

        let response_text = response
            .text()
            .await
            .map_err(|e| format!("Failed to read Groq response body: {}", e))?;

        let chat_res: ChatCompletionResponse = serde_json::from_str(&response_text)
            .map_err(|e| format!("Failed to parse Groq response JSON: {} (Raw: {})", e, response_text))?;

        let first_choice = chat_res
            .choices
            .first()
            .ok_or_else(|| format!("Groq returned an empty choice list. Raw response: {}", response_text))?;

        // Support standard content or reasoning tokens
        let raw_content = first_choice
            .message
            .content
            .as_deref()
            .filter(|s| !s.trim().is_empty())
            .or_else(|| {
                first_choice
                    .message
                    .reasoning_content
                    .as_deref()
                    .filter(|s| !s.trim().is_empty())
            })
            .or_else(|| {
                first_choice
                    .message
                    .reasoning
                    .as_deref()
                    .filter(|s| !s.trim().is_empty())
            })
            .unwrap_or("");

        // Extract JSON substring bounded by first '{' and last '}'
        let cleaned_json = Self::extract_json_slice(raw_content);

        let parsed_response: AIPreviewResponse = serde_json::from_str(cleaned_json)
            .map_err(|e| {
                format!(
                    "Failed to parse AI response JSON: {} (Extracted: '{}', Raw: '{}')",
                    e,
                    cleaned_json,
                    if raw_content.is_empty() { &response_text } else { raw_content }
                )
            })?;

        // Sanitize and validate all selectors and CSS properties
        CssValidator::sanitize_response(parsed_response)
    }

    async fn export_prompt(&self, req: &AIExportRequest) -> Result<String, String> {
        // If API key is available, we can ask Groq for a refined prompt, or use deterministic exporter
        if !self.api_key.trim().is_empty() {
            let system_prompt = r#"You are an expert prompt engineer creating instructions for AI coding assistants (like Cursor, Claude Code, GitHub Copilot).
Given an element, its original styles, user intent, and applied preview CSS changes, produce a clean, structured, actionable implementation prompt.
Rules:
- Clearly identify the component/element.
- List exact CSS property changes with before/after context where helpful.
- Note layout, responsiveness, and functional constraints.
- Explicitly state what should NOT be modified.
- Output ONLY the ready-to-paste prompt without conversational filler."#;

            let user_prompt = json!({
                "original_element": {
                    "tag": req.element_context.tag,
                    "id": req.element_context.id,
                    "classes": req.element_context.classes,
                    "text": req.element_context.text,
                    "selector": req.element_context.selector,
                    "computed_styles": req.element_context.computed_styles
                },
                "user_intent": req.instruction,
                "applied_preview_changes": req.applied_css_rules
            })
            .to_string();

            let payload = json!({
                "model": self.model,
                "temperature": 0.2,
                "max_tokens": 1024,
                "messages": [
                    { "role": "system", "content": system_prompt },
                    { "role": "user", "content": user_prompt }
                ]
            });

            if let Ok(res) = self
                .client
                .post("https://api.groq.com/openai/v1/chat/completions")
                .header("Authorization", format!("Bearer {}", self.api_key.trim()))
                .header("Content-Type", "application/json")
                .json(&payload)
                .send()
                .await
            {
                if res.status().is_success() {
                    if let Ok(chat_res) = res.json::<ChatCompletionResponse>().await {
                        if let Some(choice) = chat_res.choices.first() {
                            if let Some(ref content) = choice.message.content {
                                if !content.trim().is_empty() {
                                    return Ok(content.trim().to_string());
                                }
                            }
                        }
                    }
                }
            }
        }

        // Fallback to deterministic prompt exporter
        Ok(super::prompt_export::PromptExporter::generate_export_prompt(req))
    }
}

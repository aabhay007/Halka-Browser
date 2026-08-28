use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AICssRule {
    pub selector: String,
    pub properties: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIPreviewResponse {
    pub summary: String,
    pub css_rules: Vec<AICssRule>,
    #[serde(default)]
    pub explanation: Option<String>,
}

pub struct CssValidator;

impl CssValidator {
    /// Validates and sanitizes an AI preview response
    pub fn sanitize_response(mut response: AIPreviewResponse) -> Result<AIPreviewResponse, String> {
        if response.css_rules.is_empty() {
            return Err("AI response contains no CSS rules".to_string());
        }

        let mut sanitized_rules = Vec::new();

        for rule in response.css_rules {
            let sanitized_selector = Self::sanitize_selector(&rule.selector)?;
            let sanitized_props = Self::sanitize_properties(&rule.properties)?;

            if !sanitized_props.is_empty() {
                sanitized_rules.push(AICssRule {
                    selector: sanitized_selector,
                    properties: sanitized_props,
                });
            }
        }

        if sanitized_rules.is_empty() {
            return Err("No valid, safe CSS properties found in AI response".to_string());
        }

        response.css_rules = sanitized_rules;
        Ok(response)
    }

    /// Sanitizes CSS selector to avoid script execution or CSS injection breakouts
    pub fn sanitize_selector(selector: &str) -> Result<String, String> {
        let trimmed = selector.trim();
        if trimmed.is_empty() {
            return Err("Empty selector".to_string());
        }

        // Disallow braces, semicolons, backslashes, @ rules, or HTML tag injection
        let forbidden_chars = ['{', '}', ';', '\\', '<', '@'];
        for c in forbidden_chars {
            if trimmed.contains(c) {
                return Err(format!("Forbidden character '{}' in selector", c));
            }
        }

        // Limit selector length
        if trimmed.len() > 300 {
            return Err("Selector exceeds maximum allowed length".to_string());
        }

        Ok(trimmed.to_string())
    }

    /// Sanitizes property names and values
    pub fn sanitize_properties(
        properties: &HashMap<String, String>,
    ) -> Result<HashMap<String, String>, String> {
        let mut sanitized = HashMap::new();

        for (key, val) in properties {
            let key_norm = key.trim().to_lowercase();
            let val_norm = val.trim().to_string();

            if key_norm.is_empty() || val_norm.is_empty() {
                continue;
            }

            // Disallow forbidden property names
            if Self::is_forbidden_property(&key_norm) {
                continue;
            }

            // Disallow forbidden values (JS execution vectors in CSS)
            if Self::is_forbidden_value(&val_norm) {
                continue;
            }

            // Remove any trailing semicolons or !important (we handle !important in preview generator)
            let clean_val = val_norm
                .trim_end_matches(';')
                .replace("!important", "")
                .trim()
                .to_string();

            if !clean_val.is_empty() {
                sanitized.insert(key_norm, clean_val);
            }
        }

        Ok(sanitized)
    }

    /// Check if property name is forbidden or dangerous
    fn is_forbidden_property(prop: &str) -> bool {
        let forbidden = [
            "behavior",
            "-moz-binding",
            "binding",
            "content", // prevent pseudo-element content injection
        ];
        forbidden.iter().any(|&f| prop == f || prop.starts_with(f))
    }

    /// Check if CSS value contains dangerous script or exploit payloads
    fn is_forbidden_value(val: &str) -> bool {
        let lower = val.to_lowercase();
        let dangerous_keywords = [
            "javascript:",
            "vbscript:",
            "data:text/html",
            "expression(",
            "-moz-binding",
            "@import",
            "</style>",
            "<script",
            "url(", // block external resource loading via url() in V1 preview
        ];

        dangerous_keywords.iter().any(|&k| lower.contains(k))
    }

    /// Generates safe CSS stylesheet string from validated rules with !important
    pub fn compile_to_stylesheet(rules: &[AICssRule]) -> String {
        let mut css = String::new();
        css.push_str("/* Halka AI Preview Mode - Temporary Sandbox Styles */\n");

        for rule in rules {
            css.push_str(&rule.selector);
            css.push_str(" {\n");
            for (prop, val) in &rule.properties {
                css.push_str(&format!("  {}: {} !important;\n", prop, val));
            }
            css.push_str("}\n\n");
        }

        css
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_safe_rules() {
        let mut props = HashMap::new();
        props.insert("border-radius".to_string(), "12px".to_string());
        props.insert("background-color".to_string(), "#3b82f6".to_string());
        props.insert("transform".to_string(), "translateX(10px)".to_string());

        let response = AIPreviewResponse {
            summary: "Modernized button".to_string(),
            css_rules: vec![AICssRule {
                selector: "#checkout-btn".to_string(),
                properties: props,
            }],
            explanation: None,
        };

        let validated = CssValidator::sanitize_response(response).expect("Should pass validation");
        assert_eq!(validated.css_rules.len(), 1);
        let compiled = CssValidator::compile_to_stylesheet(&validated.css_rules);
        assert!(compiled.contains("border-radius: 12px !important;"));
        assert!(compiled.contains("transform: translateX(10px) !important;"));
    }

    #[test]
    fn test_rejects_dangerous_javascript() {
        let mut props = HashMap::new();
        props.insert("background".to_string(), "javascript:alert(1)".to_string());
        props.insert("behavior".to_string(), "url(exploit.htc)".to_string());

        let sanitized = CssValidator::sanitize_properties(&props).unwrap();
        assert!(sanitized.is_empty());
    }

    #[test]
    fn test_rejects_selector_breakout() {
        assert!(CssValidator::sanitize_selector("div { color: red }").is_err());
        assert!(CssValidator::sanitize_selector("button; @import 'evil'").is_err());
        assert!(CssValidator::sanitize_selector("button#id.class").is_ok());
        assert!(CssValidator::sanitize_selector("header > nav > button.btn").is_ok());
        assert!(CssValidator::sanitize_selector("div.container > div:nth-child(2)").is_ok());
    }
}

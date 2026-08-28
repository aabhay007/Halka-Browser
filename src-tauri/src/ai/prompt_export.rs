use super::AIExportRequest;

pub struct PromptExporter;

impl PromptExporter {
    /// Generates a clean, developer-ready implementation prompt from preview state
    pub fn generate_export_prompt(req: &AIExportRequest) -> String {
        let el = &req.element_context;
        let mut prompt = String::new();

        // 1. Title & Intent Header
        prompt.push_str(&format!(
            "Please implement the following visual design updates to `{}`.\n\n",
            el.selector
        ));

        if !req.instruction.trim().is_empty() {
            prompt.push_str(&format!("**Design Goal**: {}\n\n", req.instruction.trim()));
        }

        // 2. Component Target
        prompt.push_str("### Target Component / Element\n");
        prompt.push_str(&format!("- **Selector**: `{}`\n", el.selector));
        prompt.push_str(&format!("- **Tag**: `<{}>`\n", el.tag));
        if let Some(id) = &el.id {
            prompt.push_str(&format!("- **ID**: `#{}`\n", id));
        }
        if !el.classes.is_empty() {
            prompt.push_str(&format!("- **Classes**: `.{}`\n", el.classes.join(" .")));
        }
        if let Some(text) = &el.text {
            let truncated = if text.len() > 60 {
                format!("{}...", &text[..60])
            } else {
                text.clone()
            };
            prompt.push_str(&format!("- **Text content**: \"{}\"\n", truncated));
        }
        prompt.push('\n');

        // 3. Exact Applied CSS Changes
        prompt.push_str("### Required CSS Changes\n");
        if req.applied_css_rules.is_empty() {
            prompt.push_str("*(No CSS modifications were applied in preview)*\n\n");
        } else {
            for rule in &req.applied_css_rules {
                if rule.selector != el.selector {
                    prompt.push_str(&format!("For `{}`:\n", rule.selector));
                }
                for (prop, val) in &rule.properties {
                    // Check if we have original computed style for comparison
                    if let Some(orig_val) = el.computed_styles.get(prop) {
                        if orig_val != val {
                            prompt.push_str(&format!(
                                "- Set `{}` to `{}` (previously `{}`)\n",
                                prop, val, orig_val
                            ));
                            continue;
                        }
                    }
                    prompt.push_str(&format!("- Set `{}` to `{}`\n", prop, val));
                }
                prompt.push('\n');
            }
        }

        // 4. Constraints & Preservation Rules
        prompt.push_str("### Constraints & Guidelines\n");
        prompt.push_str("- Preserve all existing functionality, event handlers, and business logic.\n");
        prompt.push_str("- Ensure the modified component remains responsive across desktop and mobile screens.\n");
        prompt.push_str("- Maintain existing theme variables and color palettes where applicable.\n");
        prompt.push_str("- Do not alter unrelated sibling or parent elements.\n");

        prompt
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ai::{AICssRule, ElementContext};
    use std::collections::HashMap;

    #[test]
    fn test_export_prompt_generation() {
        let mut computed = HashMap::new();
        computed.insert("border-radius".to_string(), "4px".to_string());
        computed.insert("padding".to_string(), "8px 16px".to_string());

        let el = ElementContext {
            tag: "button".to_string(),
            id: Some("checkout-btn".to_string()),
            classes: vec!["btn".to_string(), "btn-primary".to_string()],
            attributes: HashMap::new(),
            text: Some("Checkout Now".to_string()),
            selector: "#checkout-btn".to_string(),
            computed_styles: computed,
            parent_context: None,
            grandparent_context: None,
        };

        let mut applied_props = HashMap::new();
        applied_props.insert("border-radius".to_string(), "12px".to_string());
        applied_props.insert("transform".to_string(), "translateX(10px)".to_string());

        let req = AIExportRequest {
            instruction: "Make button modern with rounded corners and move right".to_string(),
            element_context: el,
            applied_css_rules: vec![AICssRule {
                selector: "#checkout-btn".to_string(),
                properties: applied_props,
            }],
        };

        let prompt = PromptExporter::generate_export_prompt(&req);
        assert!(prompt.contains("Target Component / Element"));
        assert!(prompt.contains("Set `border-radius` to `12px` (previously `4px`)"));
        assert!(prompt.contains("Set `transform` to `translateX(10px)`"));
        assert!(prompt.contains("Preserve all existing functionality"));
    }
}

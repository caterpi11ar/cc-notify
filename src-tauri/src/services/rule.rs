use crate::models::Rule;
use regex::Regex;

pub struct RuleEngine;

impl RuleEngine {
    pub fn evaluate(rule: &Rule, content: &str) -> bool {
        match rule.rule_type.as_str() {
            "keyword" => content.to_lowercase().contains(&rule.pattern.to_lowercase()),
            "regex" => {
                if let Ok(re) = Regex::new(&rule.pattern) {
                    re.is_match(content)
                } else {
                    false
                }
            }
            _ => false,
        }
    }
}

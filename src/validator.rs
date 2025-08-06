use crate::events::*;
use chrono::{DateTime, Utc};
use log::{debug, info};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationRule {
    pub id: String,
    pub name: String,
    pub rule_type: RuleType,
    pub enabled: bool,
    pub parameters: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RuleType {
    Blacklist,
    ContentFilter,
    RateLimit,
    UserLevel,
    Custom,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ValidationResult {
    Allow,
    Ignore,
    Warn(String),
}

#[derive(Debug)]
pub struct TextValidator {
    rules: Vec<ValidationRule>,
    user_stats: HashMap<String, UserStats>,
}

#[derive(Debug, Clone)]
pub struct UserStats {
    last_message_time: DateTime<Utc>,
    message_count: u32,
    #[allow(unused)]
    warning_count: u32,
}

impl TextValidator {
    pub fn new() -> Self {
        Self {
            rules: Self::default_rules(),
            user_stats: HashMap::new(),
        }
    }

    fn default_rules() -> Vec<ValidationRule> {
        vec![
            ValidationRule {
                id: "blacklist".to_string(),
                name: "敏感词黑名单".to_string(),
                rule_type: RuleType::Blacklist,
                enabled: true,
                parameters: serde_json::json!({
                    "words": ["垃圾", "骗子", "广告", "刷单"]
                }),
            },
            ValidationRule {
                id: "rate_limit".to_string(),
                name: "频率限制".to_string(),
                rule_type: RuleType::RateLimit,
                enabled: true,
                parameters: serde_json::json!({
                    "max_messages_per_minute": 10,
                    "cooldown_seconds": 3
                }),
            },
            ValidationRule {
                id: "length_filter".to_string(),
                name: "长度过滤".to_string(),
                rule_type: RuleType::ContentFilter,
                enabled: true,
                parameters: serde_json::json!({
                    "min_length": 1,
                    "max_length": 200
                }),
            },
        ]
    }

    pub fn validate(&mut self, event: &TextInputEvent) -> ValidationResult {
        let anonymous = "anonymous".to_string();
        let user_id = event.metadata.user_id.as_ref().unwrap_or(&anonymous);

        debug!("Validating message from {}: {}", user_id, event.text);

        // Clone rules to avoid borrowing issues
        let rules = self.rules.clone();
        for rule in &rules {
            if !rule.enabled {
                continue;
            }

            match self.apply_rule(rule, event, user_id) {
                ValidationResult::Allow => continue,
                result => {
                    info!(
                        "Rule {} triggered for user {}: {:?}",
                        rule.name, user_id, result
                    );
                    return result;
                }
            }
        }

        ValidationResult::Allow
    }

    fn apply_rule(
        &mut self,
        rule: &ValidationRule,
        event: &TextInputEvent,
        user_id: &str,
    ) -> ValidationResult {
        match rule.rule_type {
            RuleType::Blacklist => self.check_blacklist(rule, &event.text),
            RuleType::RateLimit => self.check_rate_limit(rule, user_id),
            RuleType::ContentFilter => self.check_content_filter(rule, &event.text),
            RuleType::UserLevel => ValidationResult::Allow, // TODO: 实现用户等级检查
            RuleType::Custom => ValidationResult::Allow,    // TODO: 实现自定义规则
        }
    }

    fn check_blacklist(&self, rule: &ValidationRule, text: &str) -> ValidationResult {
        if let Some(words) = rule.parameters.get("words").and_then(|w| w.as_array()) {
            for word in words {
                if let Some(word_str) = word.as_str() {
                    if text.contains(word_str) {
                        return ValidationResult::Warn(format!("包含敏感词: {}", word_str));
                    }
                }
            }
        }
        ValidationResult::Allow
    }

    fn check_rate_limit(&mut self, rule: &ValidationRule, user_id: &str) -> ValidationResult {
        let max_messages = rule
            .parameters
            .get("max_messages_per_minute")
            .and_then(|m| m.as_u64())
            .unwrap_or(10) as u32;

        let cooldown_seconds = rule
            .parameters
            .get("cooldown_seconds")
            .and_then(|c| c.as_u64())
            .unwrap_or(3);

        let now = Utc::now();
        
        // 检查用户是否存在，如果不存在则创建新用户（第一条消息总是允许的）
        let is_new_user = !self.user_stats.contains_key(user_id);
        let user_stats = self
            .user_stats
            .entry(user_id.to_string())
            .or_insert(UserStats {
                last_message_time: now,
                message_count: 0,
                warning_count: 0,
            });

        // 如果不是新用户，检查冷却时间
        if !is_new_user {
            let time_since_last = now.signed_duration_since(user_stats.last_message_time);
            if time_since_last.num_seconds() < cooldown_seconds as i64 {
                return ValidationResult::Ignore;
            }
        }

        // 检查每分钟消息数量
        let time_since_last = now.signed_duration_since(user_stats.last_message_time);
        if time_since_last.num_seconds() < 60 {
            user_stats.message_count += 1;
            if user_stats.message_count > max_messages {
                return ValidationResult::Warn("发言过于频繁，请稍后再试".to_string());
            }
        } else {
            user_stats.message_count = 1;
        }

        user_stats.last_message_time = now;
        ValidationResult::Allow
    }

    fn check_content_filter(&self, rule: &ValidationRule, text: &str) -> ValidationResult {
        let min_length = rule
            .parameters
            .get("min_length")
            .and_then(|l| l.as_u64())
            .unwrap_or(1) as usize;

        let max_length = rule
            .parameters
            .get("max_length")
            .and_then(|l| l.as_u64())
            .unwrap_or(200) as usize;

        if text.len() < min_length {
            return ValidationResult::Ignore;
        }

        if text.len() > max_length {
            return ValidationResult::Warn("消息过长，请简化内容".to_string());
        }

        ValidationResult::Allow
    }

    #[allow(unused)]
    pub fn add_rule(&mut self, rule: ValidationRule) {
        let rule_name = rule.name.clone();
        self.rules.push(rule);
        info!("Added validation rule: {}", rule_name);
    }

    #[allow(unused)]
    pub fn remove_rule(&mut self, rule_id: &str) {
        self.rules.retain(|r| r.id != rule_id);
        info!("Removed validation rule: {}", rule_id);
    }

    #[allow(unused)]
    pub fn update_rule(&mut self, rule_id: &str, rule: ValidationRule) {
        if let Some(existing_rule) = self.rules.iter_mut().find(|r| r.id == rule_id) {
            *existing_rule = rule;
            info!("Updated validation rule: {}", rule_id);
        }
    }

    #[allow(unused)]
    pub fn get_user_stats(&self, user_id: &str) -> Option<&UserStats> {
        self.user_stats.get(user_id)
    }
}

impl Default for TextValidator {
    fn default() -> Self {
        Self::new()
    }
}

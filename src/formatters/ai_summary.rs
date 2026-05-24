use std::collections::HashMap;

use chrono::{DateTime, Utc};

use crate::repositories::messages::StoredMessage;

pub fn display_name(msg: &StoredMessage) -> String {
    if let Some(u) = &msg.username {
        format!("@{}", u)
    } else if !msg.first_name.trim().is_empty() && msg.first_name != "Unknown" {
        msg.first_name.clone()
    } else {
        format!("user_{}", msg.user_id)
    }
}

pub fn build_name_map(messages: &[StoredMessage]) -> HashMap<i64, String> {
    let mut map: HashMap<i64, String> = HashMap::new();
    for m in messages {
        let candidate = display_name(m);
        let candidate_is_handle = candidate.starts_with('@');
        match map.get(&m.user_id) {
            Some(existing) if existing.starts_with('@') => {}
            Some(_) if !candidate_is_handle => {}
            _ => {
                map.insert(m.user_id, candidate);
            }
        }
    }
    map
}

pub fn format_messages_for_llm(messages: &[StoredMessage]) -> String {
    let names = build_name_map(messages);

    messages
        .iter()
        .rev()
        .filter_map(|m| {
            let formatted_message = format_message(m.text.as_ref()?, &m.created_at)?;
            let name = names.get(&m.user_id).map(String::as_str).unwrap_or("user");
            Some(format!("--- {} {}", name, formatted_message))
        })
        .collect::<Vec<_>>()
        .join("\n\n")
}

pub fn format_message(text: &str, created_at: &DateTime<Utc>) -> Option<String> {
    let trimmed = text.trim();
    if trimmed.is_empty() || trimmed.starts_with('/') {
        return None;
    }
    let time = created_at.format("%H:%M");
    Some(format!("| {} ---\n{}", time, trimmed))
}

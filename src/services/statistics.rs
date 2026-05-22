use std::collections::HashMap;
use std::sync::Arc;

use sqlx::PgPool;
use teloxide::utils::html::{bold, escape, italic};

#[derive(Debug, serde::Deserialize)]
pub struct ToxicityReport {
    pub assessments: Vec<UserToxicity>,
}

#[derive(Debug, serde::Deserialize)]
pub struct UserToxicity {
    #[serde(rename = "userId")]
    pub user_id: i64,
    pub t_level: u8,
    pub categories: Vec<ToxicityCategory>,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ToxicityCategory {
    Mat,
    Sexism,
    Racism,
    Homophobia,
    Harassment,
    Threats,
    Slurs,
    Doxxing,
}

impl ToxicityCategory {
    fn label(&self) -> &'static str {
        match self {
            ToxicityCategory::Mat => "мат",
            ToxicityCategory::Sexism => "сексизм",
            ToxicityCategory::Racism => "расизм",
            ToxicityCategory::Homophobia => "гомофобия",
            ToxicityCategory::Harassment => "харассмент",
            ToxicityCategory::Threats => "угрозы",
            ToxicityCategory::Slurs => "оскорбления",
            ToxicityCategory::Doxxing => "деанон",
        }
    }
}

use crate::{
    errors::AppError,
    infra::ai_client::AiClient,
    repositories::{self, chat_members::MemberWithMessages},
};

pub async fn top_members(
    pool: &PgPool,
    ai_client: Arc<AiClient>,
    system_prompt: &str,
    chat_id: i64,
) -> Result<String, AppError> {
    let top_members =
        repositories::chat_members::get_top_members_with_messages(pool, chat_id, 10, 10).await?;

    let user_prompt = build_toxicity_input(&top_members);
    let raw_response = ai_client
        .make_request(system_prompt, &user_prompt, true)
        .await?;

    let report: ToxicityReport = serde_json::from_str(&raw_response)
        .map_err(|err| AppError::AiResponse(format!("invalid json: {}", err)))?;

    Ok(format_top_members(&top_members, &report))
}

fn build_toxicity_input(rows: &[MemberWithMessages]) -> String {
    rows.iter()
        .map(build_user_block)
        .collect::<Vec<_>>()
        .join("\n")
}

fn build_user_block(member: &MemberWithMessages) -> String {
    let mut out = format!("=== USER_ID: {} ===\n", member.id);

    for msg in member.messages.iter().rev() {
        let time = msg.created_at.format("%H:%M");
        out.push_str(&format!("[{}] {}\n", time, msg.text));
    }

    out
}

fn toxicity_emoji(level: u8) -> &'static str {
    match level {
        0..=3 => "🟢",
        4..=6 => "🟡",
        _ => "🔴",
    }
}

fn format_top_members(rows: &[MemberWithMessages], report: &ToxicityReport) -> String {
    if rows.is_empty() {
        return "В этом чате пока нет активных участников или бот еще не успел их запомнить."
            .to_string();
    }

    let toxicity: HashMap<i64, &UserToxicity> =
        report.assessments.iter().map(|a| (a.user_id, a)).collect();

    let mut out = bold("🏆 Топ участников чата");
    out.push('\n');
    for (i, m) in rows.iter().enumerate() {
        out.push_str(&format!(
            "{}. {} — <b>{}</b> сообщ.\n",
            i + 1,
            display_name(m),
            m.message_count,
        ));

        if let Some(tox) = toxicity.get(&m.id) {
            out.push_str(&format!(
                "   {} токсичность: <b>{}/10</b>",
                toxicity_emoji(tox.t_level),
                tox.t_level,
            ));
            if !tox.categories.is_empty() {
                let cats = tox
                    .categories
                    .iter()
                    .map(ToxicityCategory::label)
                    .collect::<Vec<_>>()
                    .join(", ");
                out.push_str(&format!(" — {}", italic(&cats)));
            }
            out.push('\n');
        }
    }
    out
}

fn display_name(member: &MemberWithMessages) -> String {
    let nick = match member.username.as_deref() {
        Some(u) => format!("@{}", escape(u)),
        None => italic(&escape(&member.first_name)),
    };

    let suffix = if member.is_premium {
        italic("премиум")
    } else {
        italic("бомж")
    };

    format!("{nick} {suffix}")
}

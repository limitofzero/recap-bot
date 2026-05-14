use sqlx::PgPool;
use teloxide::utils::html::{bold, escape, italic};

use crate::{
    errors::AppError,
    repositories::{self, chat_members::MemberWithMessages},
};

pub async fn top_members(pool: &PgPool, chat_id: i64) -> Result<String, AppError> {
    let top_members = repositories::chat_members::get_top_members(pool, chat_id, 10).await?;

    Ok(format_top_members(&top_members))
}

fn format_top_members(rows: &[MemberWithMessages]) -> String {
    if rows.is_empty() {
        return "В этом чате пока нет активных участников или бот еще не успел их запомнить."
            .to_string();
    }
    let mut out = bold("🏆 Топ участников чата");
    out.push('\n');
    for (i, m) in rows.iter().enumerate() {
        out.push_str(&format!(
            "{}. {} — <b>{}</b> сообщ.\n",
            i + 1,
            escape(&display_name(m)), // ← username/имя/whatever
            m.message_count,          // число — экранировать не надо
        ));
    }
    out
}

fn display_name(member: &MemberWithMessages) -> String {
    let nick = member
        .username
        .as_ref()
        .map(|u| format!("@{u}"))
        .unwrap_or_else(|| italic(&member.first_name));

    let premium_suffix = if member.is_premium {
        "*зажиточный*"
    } else {
        "*нет денег на премиум*"
    };
    format!("{nick} {}", italic(premium_suffix))
}

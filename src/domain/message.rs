use serde::Serialize;

type UserId = u64;

#[derive(Serialize)]
pub struct MessagePayload {
    message_id: i32,
    chat: Chat,
    date: i64,
    text: Option<String>,
    edit_date: Option<i64>,
    new_chat_members: Option<Vec<ChatMemberUpdate>>,
    left_chat_member: Option<Vec<ChatMemberUpdate>>,
}

#[derive(Serialize)]
pub struct Chat {
    id: i64,
}

#[derive(Serialize)]
pub struct From {
    id: UserId,
}

#[derive(Serialize)]
pub struct ChatMemberUpdate {
    id: UserId,
}

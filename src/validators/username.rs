pub fn is_valid(username: &str) -> bool {
    let trimmed = username.trim();
    trimmed.starts_with('@')
        && trimmed.len() > 1
        && trimmed[1..]
            .chars()
            .all(|c| c.is_alphanumeric() || c == '_')
}

pub fn normalize(username: &str) -> &str {
    &username.trim()[1..]
}

use teloxide::types::User;

pub fn get_username(from: &User) -> String {
    match from.username.as_ref() {
        Some(u) => format!("@{}", u),
        None => from.first_name.clone(),
    }
}

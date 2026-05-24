use std::fmt;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Prompt {
    TopMembers,
    Recap,
    ResponseToUser,
    UserRecap,
}

impl fmt::Display for Prompt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let str = match self {
            Prompt::Recap => "recap",
            Prompt::TopMembers => "top_members",
            Prompt::ResponseToUser => "response_to_user",
            Prompt::UserRecap => "user_recap",
        };

        write!(f, "{}", str)
    }
}

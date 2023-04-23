use std::fmt::Display;

use url::Url;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum SourceIdentifier {
    Invalid(String),
    GitHub { username: String },
    GitLab { username: String },
    Http { address: Url },
}

impl From<String> for SourceIdentifier {
    fn from(value: String) -> Self {
        let mut value_parts = value.split_terminator(":");
        let source_type = value_parts.next();

        // TODO: Make this less shit
        match source_type {
            Some("github") => Self::GitHub {
                username: value_parts.next().unwrap().to_string(),
            },
            Some("gitlab") => Self::GitLab {
                username: value_parts.next().unwrap().to_string(),
            },
            Some("http" | "https") => Self::Http {
                address: value.parse().unwrap(),
            },
            _ => Self::Invalid(value),
        }
    }
}

impl Display for SourceIdentifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Invalid(value) => value.fmt(f),
            Self::GitHub { username } => f.write_fmt(format_args!("github:{username}")),
            Self::GitLab { username } => f.write_fmt(format_args!("gitlab:{username}")),
            Self::Http { address } => f.write_str(address.as_str()),
        }
    }
}

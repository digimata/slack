//! Structured errors and the stable exit-code contract.

use thiserror::Error;

/// Process exit codes per the UX contract (plan §2.3).
pub mod exit {
    pub const USAGE: i32 = 2;
    pub const AUTH: i32 = 3;
    pub const API: i32 = 4;
    pub const TRANSPORT: i32 = 5;
    pub const RATE_LIMIT: i32 = 6;
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("{0}")]
    Usage(String),

    #[error("{0}")]
    Auth(String),

    #[error("slack api {method}: {code}")]
    Api { method: String, code: String },

    #[error("transport: {0}")]
    Transport(#[from] reqwest::Error),

    #[error("rate limited on {method}; retry budget exhausted")]
    RateLimit { method: String },

    #[error("ambiguous {kind} '{query}': {}", candidates.join(", "))]
    Ambiguous {
        kind: &'static str,
        query: String,
        candidates: Vec<String>,
    },

    #[error("no {kind} matching '{query}'")]
    NotFound { kind: &'static str, query: String },

    #[error("io: {0}")]
    Io(#[from] std::io::Error),

    #[error("json: {0}")]
    Json(#[from] serde_json::Error),
}

impl Error {
    /// Map this failure to its stable exit code.
    pub fn exit_code(&self) -> i32 {
        match self {
            Error::Usage(_) | Error::Ambiguous { .. } | Error::NotFound { .. } => exit::USAGE,
            Error::Auth(_) => exit::AUTH,
            Error::Api { .. } => exit::API,
            Error::Transport(_) | Error::Io(_) | Error::Json(_) => exit::TRANSPORT,
            Error::RateLimit { .. } => exit::RATE_LIMIT,
        }
    }
}

pub type Result<T> = std::result::Result<T, Error>;

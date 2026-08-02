//! Slack Web API client: transport, envelope handling, pagination, retries.

mod client;
mod types;

pub use client::Client;
pub use types::{Channel, Message, User};

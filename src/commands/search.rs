//! Message search (user/session tokens only; page-based pagination).

use serde_json::Value;

use crate::cli::SearchCmd;
use crate::config::TokenKind;
use crate::error::{Error, Result};
use crate::output::{print_json, ts_local};

use super::Ctx;

pub fn run(ctx: &Ctx, cmd: SearchCmd) -> Result<()> {
    match cmd {
        SearchCmd::Messages { query, sort, limit } => messages(ctx, &query.join(" "), &sort, limit),
    }
}

fn messages(ctx: &Ctx, query: &str, sort: &str, limit: usize) -> Result<()> {
    if query.trim().is_empty() {
        return Err(Error::Usage("empty search query".into()));
    }
    if ctx.profile.kind() == TokenKind::Bot {
        return Err(Error::Usage(
            "search requires a user or session token; this profile is a bot".into(),
        ));
    }
    if !matches!(sort, "score" | "timestamp") {
        return Err(Error::Usage("--sort must be score or timestamp".into()));
    }

    let mut matches: Vec<Value> = Vec::new();
    let mut page: u64 = 1;
    loop {
        let v = ctx.client.call(
            "search.messages",
            &[
                ("query", query.to_string()),
                ("sort", sort.to_string()),
                ("count", "100".to_string()),
                ("page", page.to_string()),
            ],
        )?;
        let batch = v
            .pointer("/messages/matches")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default();
        let pages = v
            .pointer("/messages/paging/pages")
            .and_then(Value::as_u64)
            .unwrap_or(1);
        matches.extend(batch);
        if matches.len() >= limit || page >= pages {
            break;
        }
        page += 1;
    }
    matches.truncate(limit);

    if ctx.json {
        print_json(&serde_json::json!({ "query": query, "matches": matches }));
        return Ok(());
    }
    for m in &matches {
        let s = |ptr: &str| m.pointer(ptr).and_then(Value::as_str).unwrap_or("");
        let channel = s("/channel/name");
        let where_ = if channel.is_empty() {
            s("/channel/id").to_string()
        } else {
            format!("#{channel}")
        };
        println!(
            "[{}] {} {}: {}",
            ts_local(s("/ts")),
            where_,
            s("/username"),
            s("/text"),
        );
        let link = s("/permalink");
        if !link.is_empty() {
            println!("    {link}");
        }
    }
    Ok(())
}

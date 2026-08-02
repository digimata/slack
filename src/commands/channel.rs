//! Channel commands: list, info.

use serde_json::Value;

use crate::cli::ChannelCmd;
use crate::error::Result;
use crate::output::print_json;
use crate::slack::Channel;

use super::Ctx;

pub fn run(ctx: &Ctx, cmd: ChannelCmd) -> Result<()> {
    match cmd {
        ChannelCmd::List {
            types,
            archived,
            limit,
            all,
        } => list(ctx, &types, archived, if all { None } else { Some(limit) }),
        ChannelCmd::Info { channel } => info(ctx, &channel),
    }
}

fn list(ctx: &Ctx, types: &str, archived: bool, limit: Option<usize>) -> Result<()> {
    let params = [
        ("types", types.to_string()),
        ("exclude_archived", (!archived).to_string()),
        ("limit", "1000".to_string()),
    ];
    let (items, next) = ctx
        .client
        .paged("conversations.list", &params, "channels", limit)?;
    if ctx.json {
        print_json(&serde_json::json!({ "channels": items, "next_cursor": next }));
        return Ok(());
    }
    let dir = ctx.dir();
    let mut channels: Vec<Channel> = items
        .into_iter()
        .filter_map(|v| serde_json::from_value(v).ok())
        .collect();
    channels.sort_by(|a, b| a.name.cmp(&b.name));
    for c in &channels {
        let name = if c.is_im {
            let partner = c.user.as_deref().unwrap_or("?");
            format!("@{}", dir.user_label(partner))
        } else {
            c.handle()
        };
        let mut flags = Vec::new();
        if c.is_private && !c.is_im && !c.is_mpim {
            flags.push("private");
        }
        if c.is_mpim {
            flags.push("group-dm");
        }
        if c.is_archived {
            flags.push("archived");
        }
        if c.is_member {
            flags.push("member");
        }
        let members = c
            .num_members
            .map(|n| format!(" {n} members"))
            .unwrap_or_default();
        let suffix = if flags.is_empty() {
            String::new()
        } else {
            format!(" ({})", flags.join(", "))
        };
        println!("{name}  {}{members}{suffix}", c.id);
    }
    if next.is_some() {
        eprintln!("more results available — raise --limit or pass --all");
    }
    Ok(())
}

fn info(ctx: &Ctx, channel: &str) -> Result<()> {
    let dir = ctx.dir();
    let c = dir.resolve_channel(channel)?;
    let v = ctx
        .client
        .call("conversations.info", &[("channel", c.id.clone())])?;
    if ctx.json {
        print_json(&v);
        return Ok(());
    }
    let obj = v.get("channel").cloned().unwrap_or(Value::Null);
    let s = |ptr: &str| {
        obj.pointer(ptr)
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string()
    };
    println!("{}  {}", c.handle(), c.id);
    let topic = s("/topic/value");
    let purpose = s("/purpose/value");
    if !topic.is_empty() {
        println!("topic: {topic}");
    }
    if !purpose.is_empty() {
        println!("purpose: {purpose}");
    }
    if let Some(n) = obj.get("num_members").and_then(Value::as_u64) {
        println!("members: {n}");
    }
    if let Some(created) = obj.get("created").and_then(Value::as_i64) {
        println!("created: {}", crate::output::ts_local(&created.to_string()));
    }
    Ok(())
}

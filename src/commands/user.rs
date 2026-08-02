//! User commands: list, info.

use serde_json::Value;

use crate::cli::UserCmd;
use crate::error::Result;
use crate::output::print_json;
use crate::slack::User;

use super::Ctx;

pub fn run(ctx: &Ctx, cmd: UserCmd) -> Result<()> {
    match cmd {
        UserCmd::List {
            include_deleted,
            limit,
            all,
        } => list(ctx, include_deleted, if all { None } else { Some(limit) }),
        UserCmd::Info { user } => info(ctx, &user),
    }
}

fn list(ctx: &Ctx, include_deleted: bool, limit: Option<usize>) -> Result<()> {
    let params = [("limit", "500".to_string())];
    let (items, next) = ctx.client.paged("users.list", &params, "members", limit)?;
    if ctx.json {
        print_json(&serde_json::json!({ "members": items, "next_cursor": next }));
        return Ok(());
    }
    let mut users: Vec<User> = items
        .into_iter()
        .filter_map(|v| serde_json::from_value(v).ok())
        .collect();
    users.retain(|u| include_deleted || !u.deleted);
    users.sort_by_key(|a| a.label().to_lowercase());
    for u in &users {
        let mut flags = Vec::new();
        if u.is_bot {
            flags.push("bot");
        }
        if u.deleted {
            flags.push("deactivated");
        }
        let suffix = if flags.is_empty() {
            String::new()
        } else {
            format!(" ({})", flags.join(", "))
        };
        let real = u.real_name.as_deref().unwrap_or("");
        println!("@{}  {}  {real}{suffix}", u.label(), u.id);
    }
    if next.is_some() {
        eprintln!("more members available — raise --limit or pass --all");
    }
    Ok(())
}

fn info(ctx: &Ctx, query: &str) -> Result<()> {
    let dir = ctx.dir();
    let u = dir.resolve_user(query)?;
    let v = ctx.client.call("users.info", &[("user", u.id.clone())])?;
    if ctx.json {
        print_json(&v);
        return Ok(());
    }
    let obj = v.get("user").cloned().unwrap_or(Value::Null);
    let s = |ptr: &str| {
        obj.pointer(ptr)
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string()
    };
    println!("@{}  {}", u.label(), u.id);
    let real = s("/real_name");
    if !real.is_empty() {
        println!("name: {real}");
    }
    let title = s("/profile/title");
    if !title.is_empty() {
        println!("title: {title}");
    }
    let tz = s("/tz");
    if !tz.is_empty() {
        println!("tz: {tz}");
    }
    if obj.get("is_bot").and_then(Value::as_bool).unwrap_or(false) {
        println!("bot: yes");
    }
    Ok(())
}

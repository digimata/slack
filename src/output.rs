//! Rendering helpers: timestamps, Slack markup cleanup, JSON emission.

use chrono::{Local, TimeZone};
use serde_json::Value;

use crate::resolve::Directory;

/// Slack ts (`"1754160000.000100"`) → local `YYYY-MM-DD HH:MM`.
pub fn ts_local(ts: &str) -> String {
    let secs: i64 = ts
        .split('.')
        .next()
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);
    match Local.timestamp_opt(secs, 0) {
        chrono::LocalResult::Single(dt) => dt.format("%Y-%m-%d %H:%M").to_string(),
        _ => ts.to_string(),
    }
}

/// Parse an operator time filter (RFC 3339 or `YYYY-MM-DD`, local midnight)
/// into a Slack timestamp string.
pub fn parse_time(s: &str) -> Option<String> {
    if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(s) {
        return Some(format!("{}.000000", dt.timestamp()));
    }
    let date = chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d").ok()?;
    let naive = date.and_hms_opt(0, 0, 0)?;
    let local = Local.from_local_datetime(&naive).single()?;
    Some(format!("{}.000000", local.timestamp()))
}

/// Rewrite Slack message markup for terminal display: `<@U…>` → `@name`,
/// `<#C…|name>` → `#name`, `<!here>` → `@here`, `<url|label>` → label,
/// and HTML entities unescaped.
pub fn clean_text(text: &str, dir: &Directory) -> String {
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(i) = rest.find('<') {
        out.push_str(&rest[..i]);
        let after = &rest[i + 1..];
        match after.find('>') {
            Some(j) => {
                out.push_str(&render_token(&after[..j], dir));
                rest = &after[j + 1..];
            }
            None => {
                out.push('<');
                rest = after;
            }
        }
    }
    out.push_str(rest);
    out.replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&amp;", "&")
}

fn render_token(token: &str, dir: &Directory) -> String {
    let (body, label) = match token.split_once('|') {
        Some((b, l)) => (b, Some(l)),
        None => (token, None),
    };
    if let Some(id) = body.strip_prefix('@') {
        let name = label
            .map(str::to_string)
            .unwrap_or_else(|| dir.user_label(id));
        format!("@{name}")
    } else if let Some(chan) = body.strip_prefix('#') {
        format!("#{}", label.unwrap_or(chan))
    } else if let Some(cmd) = body.strip_prefix('!') {
        format!("@{cmd}")
    } else {
        label.unwrap_or(body).to_string()
    }
}

/// Emit a value as pretty JSON on stdout.
pub fn print_json(v: &Value) {
    println!("{}", serde_json::to_string_pretty(v).unwrap_or_default());
}

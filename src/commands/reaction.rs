//! Reaction commands: add, remove.

use crate::cli::ReactionCmd;
use crate::error::Result;
use crate::output::print_json;

use super::Ctx;

pub fn run(ctx: &Ctx, cmd: ReactionCmd) -> Result<()> {
    match cmd {
        ReactionCmd::Add {
            channel,
            timestamp,
            emoji,
        } => apply(ctx, "reactions.add", &channel, &timestamp, &emoji),
        ReactionCmd::Remove {
            channel,
            timestamp,
            emoji,
        } => apply(ctx, "reactions.remove", &channel, &timestamp, &emoji),
    }
}

fn apply(ctx: &Ctx, method: &str, channel: &str, ts: &str, emoji: &str) -> Result<()> {
    let dir = ctx.dir();
    let c = dir.resolve_channel(channel)?;
    let name = emoji.trim_matches(':').to_string();
    let v = ctx.client.call(
        method,
        &[
            ("channel", c.id.clone()),
            ("timestamp", ts.to_string()),
            ("name", name.clone()),
        ],
    )?;
    if ctx.json {
        print_json(&v);
        return Ok(());
    }
    let verb = if method.ends_with("add") {
        "added"
    } else {
        "removed"
    };
    println!("{verb} :{name}: on {} ts {ts}", c.handle());
    Ok(())
}

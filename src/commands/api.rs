//! Raw Web API escape hatch: `slack api <method> [key=value ...] [--data PATH|-]`.

use std::io::Read;

use serde_json::Value;

use crate::error::{Error, Result};
use crate::output::print_json;

use super::Ctx;

pub fn run(ctx: &Ctx, method: &str, params: &[String], data: Option<&str>) -> Result<()> {
    if data.is_some() && !params.is_empty() {
        return Err(Error::Usage(
            "pass key=value parameters or --data, not both".into(),
        ));
    }
    let v = match data {
        Some(source) => {
            let raw = if source == "-" {
                let mut buf = String::new();
                std::io::stdin().read_to_string(&mut buf)?;
                buf
            } else {
                std::fs::read_to_string(source)?
            };
            let body: Value = serde_json::from_str(&raw)?;
            ctx.client.call_json(method, &body)?
        }
        None => {
            let pairs: Vec<(&str, String)> = params
                .iter()
                .map(|p| {
                    p.split_once('=')
                        .map(|(k, v)| (k, v.to_string()))
                        .ok_or_else(|| Error::Usage(format!("expected key=value, got '{p}'")))
                })
                .collect::<Result<_>>()?;
            ctx.client.call(method, &pairs)?
        }
    };
    print_json(&v);
    Ok(())
}

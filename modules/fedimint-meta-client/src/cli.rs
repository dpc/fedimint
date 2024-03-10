use std::ffi;

use clap::Parser;
use serde::Serialize;

use super::MetaClientModule;

#[derive(Parser, Serialize)]
enum Opts {
    GetConsensus,
}

pub(crate) async fn handle_cli_command(
    _meta: &MetaClientModule,
    args: &[ffi::OsString],
) -> anyhow::Result<serde_json::Value> {
    let x = Opts::try_parse_from(args.iter())?;

    Ok(serde_json::to_value(x).expect("can't fail"))
}

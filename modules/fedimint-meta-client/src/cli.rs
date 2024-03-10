use std::{ffi, iter};

use anyhow::Context as _;
use clap::Parser;
use fedimint_meta_common::{MetaConsensusValue, MetaKey};
use serde::Serialize;
use serde_json::json;

use super::MetaClientModule;
use crate::api::MetaFederationApi;

#[derive(Parser, Serialize)]
enum Opts {
    GetConsensus {
        #[arg(long, default_value = "0")]
        key: MetaKey,
        #[arg(long)]
        raw: bool,
    },
}

pub(crate) async fn handle_cli_command(
    meta: &MetaClientModule,
    args: &[ffi::OsString],
) -> anyhow::Result<serde_json::Value> {
    let opts = Opts::parse_from(iter::once(&ffi::OsString::from("meta")).chain(args.iter()));

    let res = match opts {
        Opts::GetConsensus { key, raw } => {
            if let Some(MetaConsensusValue { revision, value }) =
                meta.module_api.get_consensus(key).await?
            {
                let value = if raw {
                    serde_json::to_value(value).expect("can't fail")
                } else {
                    serde_json::to_value(
                        serde_json::from_slice(value.as_slice())
                            .context("deserializating consensus value as json")?,
                    )
                    .expect("can't fail")
                };
                json!({
                    "revision": revision,
                    "value": value
                })
            } else {
                serde_json::Value::Null
            }
        }
    };

    Ok(res)
}

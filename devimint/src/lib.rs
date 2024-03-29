use std::ffi;

use clap::Parser as _;
use cli::cleanup_on_exit;
pub use devfed::{dev_fed, DevFed};
pub use external::{
    external_daemons, open_channel, ExternalDaemons, LightningNode, Lightningd,
    LightningdProcessHandle, Lnd,
};
use futures::Future;
pub use gatewayd::Gatewayd;
use tests::log_binary_versions;
use util::ProcessManager;

pub mod cli;
pub mod devfed;
pub mod envs;
pub mod external;
pub mod federation;
pub mod gatewayd;
pub mod tests;
pub mod util;
pub mod vars;
pub mod version_constants;

pub async fn run_test<F, FF>(f: F) -> anyhow::Result<()>
where
    F: FnOnce(ProcessManager) -> FF,
    FF: Future<Output = anyhow::Result<()>>,
{
    let args = cli::CommonArgs::parse_from::<_, ffi::OsString>(vec![]);

    let (process_mgr, task_group) = cli::setup(args).await?;
    log_binary_versions().await?;
    cleanup_on_exit(f(process_mgr), task_group).await?;
    Ok(())
}

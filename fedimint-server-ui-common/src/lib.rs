use std::net::SocketAddr;
use std::pin::Pin;
use std::sync::Arc;

use async_trait::async_trait;

/// A `fedimint-server` backend interface for `fedimint-server-ui`
///
/// Any functionality that the `fedimint-web-ui` needs from `fedimint-ui`
/// goes here. This way `fedimint-server` and `fedimint-server-ui` don't need
/// to know anything about their implementations, can be compiled separately,
/// etc.
#[async_trait]
pub trait IUiBackend {
    async fn are_local_params_set(&self) -> bool;
}

/// An instance of [`IUiBackend`]
pub type DynUiBackend = Arc<dyn IUiBackend + Send + Sync + 'static>;

/// An async function that starts the web ui
pub type WebUiStartFn = Box<
    dyn Fn(
            DynUiBackend,

            SocketAddr,
        ) -> Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send + 'static>>
        + Send
        + Sync
        + 'static,
>;

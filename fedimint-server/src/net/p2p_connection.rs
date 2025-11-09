use std::io::Cursor;
use std::time::Duration;

use anyhow::{Context, bail};
use async_trait::async_trait;
use bytes::Bytes;
use fedimint_core::encoding::{Decodable, Encodable};
use fedimint_logging::LOG_NET;
use futures::{SinkExt, StreamExt};
use iroh::endpoint::Connection;
use serde::Serialize;
use serde::de::DeserializeOwned;
use tokio::io::AsyncWriteExt as _;
use tokio::net::TcpStream;
use tokio_rustls::TlsStream;
use tokio_util::codec::{Framed, LengthDelimitedCodec};
use tracing::info;

pub type DynP2PConnection<M> = Box<dyn IP2PConnection<M>>;

#[async_trait]
pub trait IP2PConnection<M>: Send + 'static {
    async fn send(&mut self, message: M) -> anyhow::Result<()>;

    async fn receive(&mut self) -> anyhow::Result<M>;

    fn rtt(&self) -> Duration;

    fn into_dyn(self) -> DynP2PConnection<M>
    where
        Self: Sized,
    {
        Box::new(self)
    }
}

#[async_trait]
impl<M> IP2PConnection<M> for Framed<TlsStream<TcpStream>, LengthDelimitedCodec>
where
    M: Encodable + Decodable + Serialize + DeserializeOwned + Send + 'static,
{
    async fn send(&mut self, message: M) -> anyhow::Result<()> {
        let mut bytes = Vec::new();

        bincode::serialize_into(&mut bytes, &message)?;

        SinkExt::send(self, Bytes::from_owner(bytes)).await?;

        Ok(())
    }

    async fn receive(&mut self) -> anyhow::Result<M> {
        Ok(bincode::deserialize_from(Cursor::new(
            &self.next().await.context("Framed stream is closed")??,
        ))?)
    }

    fn rtt(&self) -> Duration {
        Duration::from_millis(0)
    }
}

#[async_trait]
impl<M> IP2PConnection<M> for Connection
where
    M: Serialize + DeserializeOwned + Send + 'static,
{
    async fn send(&mut self, message: M) -> anyhow::Result<()> {
        let mut bytes = Vec::new();

        bincode::serialize_into(&mut bytes, &message)?;

        let mut sink = self.open_uni().await?;

        sink.write_all(&(bytes.len() as u32).to_be_bytes()).await?;
        sink.write_all(&bytes).await?;
        sink.flush().await?;
        sink.finish()?;

        Ok(())
    }

    async fn receive(&mut self) -> anyhow::Result<M> {
        let mut stream = self.accept_uni().await?;

        let mut len = [0; 4];

        stream.read_exact(&mut len).await?;

        let len = u32::from_be_bytes(len) as usize;

        info!(target: LOG_NET, %len, "Received p2p message len");
        let mut bytes = Vec::with_capacity(len);
        info!(target: LOG_NET, %len, "Received p2p message itself");

        if 1_000_000 < len {
            bail!("Message too large");
        }
        bytes.resize(len, 0);
        stream.read_exact(&mut bytes).await?;

        Ok(bincode::deserialize_from(Cursor::new(&bytes))?)
    }

    fn rtt(&self) -> Duration {
        self.rtt()
    }
}

use async_trait::async_trait;
use fedimint_core::db::{Database, IDatabaseTransactionOpsCoreTyped, IRawDatabaseExt};
use fedimint_core::timing::{self, TimeReporter};
use fedimint_logging::TracingSetup;
use futures::StreamExt;
use tracing::{info, Level};

use crate::consensus::db::{AlephUnitsKey, AlephUnitsPrefix};
use crate::LOG_CONSENSUS;

pub struct BackupReader {
    db: Database,
}

impl BackupReader {
    pub fn new(db: Database) -> Self {
        Self { db }
    }
}

#[async_trait]
impl aleph_bft::BackupReader for BackupReader {
    async fn read(&mut self) -> std::io::Result<Vec<u8>> {
        let mut dbtx = self.db.begin_transaction_nc().await;

        let units = dbtx
            .find_by_prefix(&AlephUnitsPrefix)
            .await
            .map(|entry| entry.1)
            .collect::<Vec<Vec<u8>>>()
            .await;

        if !units.is_empty() {
            info!(target: LOG_CONSENSUS, units_len = %units.len(), "Recovering from an in-session-shutdown");
        }

        Ok(units.into_iter().flatten().collect())
    }
}

pub struct BackupWriter {
    db: Database,
}

impl BackupWriter {
    pub fn new(db: Database) -> Self {
        Self { db }
    }
}

#[async_trait]
impl aleph_bft::BackupWriter for BackupWriter {
    async fn append(&mut self, data: &[u8]) -> std::io::Result<()> {
        let mut dbtx = self.db.begin_transaction().await;

        let index = dbtx
            .find_by_prefix_sorted_descending(&AlephUnitsPrefix)
            .await
            .next()
            .await
            .map_or(0, |entry| (entry.0 .0) + 1);

        dbtx.insert_new_entry(&AlephUnitsKey(index), &data.to_owned())
            .await;

        dbtx.commit_tx_result()
            .await
            .expect("This is the only place where we write to this key");

        Ok(())
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn backup_writer_perf() -> anyhow::Result<()> {
    // Ensure tracing has been set once
    let _ = TracingSetup::default().init();

    let db = fedimint_rocksdb::RocksDb::open("../fedimint-dbg-copy-by-dpc/database")?;

    let db: Database = db.into_database();

    let _timing: TimeReporter /* logs on drop */ = timing::TimeReporter::new("alephbft-backup-writer-append").level(
        Level::DEBUG);

    let mut dbtx = db.begin_transaction().await;

    let index = dbtx
        .find_by_prefix_sorted_descending(&AlephUnitsPrefix)
        .await
        .next()
        .await
        .map_or(0, |entry| dbg!(entry.0 .0) + 1);

    drop(_timing);

    assert_eq!(index, 2484);
    Ok(())
}

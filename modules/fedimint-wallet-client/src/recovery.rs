use std::time::Duration;

use fedimint_bitcoind::{create_bitcoind, DynBitcoindRpc, IBitcoindRpcExt as _};
use fedimint_client::derivable_secret::{ChildId, DerivableSecret};
use fedimint_client::module::init::ClientModuleRecoverArgs;
use fedimint_client::module::recovery::{DynModuleBackup, ModuleBackup};
use fedimint_core::bitcoin_migration::bitcoin30_to_bitcoin29_script;
use fedimint_core::core::{IntoDynInstance, ModuleInstanceId};
use fedimint_core::db::{DatabaseTransaction, IDatabaseTransactionOpsCoreTyped as _};
use fedimint_core::encoding::{Decodable, Encodable};
use fedimint_core::task::sleep;
use fedimint_core::transaction::Transaction;
use fedimint_wallet_common::config::WalletClientConfig;
use fedimint_wallet_common::tweakable::Tweakable as _;
use fedimint_wallet_common::PegInDescriptor;
use serde::{Deserialize, Serialize};
use tracing::warn;

use crate::client_db::{RecoveryFinalizedKey, RecoveryStateKey};
use crate::{WalletClientInit, WalletClientModule};

#[derive(Clone, PartialEq, Eq, Debug, Encodable, Decodable)]
pub struct WalletBackup;

impl ModuleBackup for WalletBackup {}

impl IntoDynInstance for WalletBackup {
    type DynType = DynModuleBackup;

    fn into_dyn(self, instance_id: ModuleInstanceId) -> Self::DynType {
        DynModuleBackup::from_typed(instance_id, self)
    }
}

/// Wallet module recovery state persisted in the database
#[derive(Clone, PartialEq, Eq, Debug, Encodable, Decodable, Serialize, Deserialize, Default)]
pub struct WalletRecoveryState {
    last_used_idx: u64,
    next_to_check_idx: u64,
}

impl WalletRecoveryState {
    pub fn new() -> Self {
        Default::default()
    }

    fn unused_gap_under_limit(&self) -> bool {
        (self.next_to_check_idx - self.last_used_idx) < 100
    }
}

const TRANSACTION_STATUS_FETCH_INTERVAL: Duration = Duration::from_secs(1);

#[derive(Clone, Debug)]
pub struct WalletRecovery {
    state: WalletRecoveryState,
    module_record_secret: DerivableSecret,
    cfg: WalletClientConfig,
    rpc: DynBitcoindRpc,
    wallet_descriptor: PegInDescriptor,
}

impl WalletRecovery {
    pub async fn recover(
        args: &ClientModuleRecoverArgs<WalletClientInit>,
        rpc: DynBitcoindRpc,
        _snapshot: Option<&WalletBackup>,
    ) -> anyhow::Result<()> {
        args.db().ensure_isolated().expect("must be isolated db");

        let state = {
            let mut dbtx = args.db().begin_transaction_nc().await;
            if Self::load_finalized_dbtx(&mut dbtx)
                .await
                .unwrap_or_default()
            {
                warn!("Module recovery already finalized");
                return Ok(());
            }
            Self::load_state_dbtx(&mut dbtx).await.unwrap_or_default()
        };

        let cfg = args.cfg().to_owned();
        let mut s = Self {
            rpc,
            state,
            wallet_descriptor: cfg.peg_in_descriptor.clone(),
            module_record_secret: args.module_root_secret().clone(),
            cfg,
        };

        let mut all_used = Vec::new();

        while s.state.unused_gap_under_limit() {
            let mut used: Vec<_> = s
                .check_for_on_chain_deposit(s.state.next_to_check_idx)
                .await;

            if !used.is_empty() {
                s.state.last_used_idx = s.state.next_to_check_idx;
            }

            all_used.append(&mut used);

            s.state.next_to_check_idx += 1;
        }

        todo!()
    }

    async fn load_state_dbtx(dbtx: &mut DatabaseTransaction<'_>) -> Option<WalletRecoveryState> {
        dbtx.get_value(&RecoveryStateKey).await
    }

    async fn load_finalized_dbtx(dbtx: &mut DatabaseTransaction<'_>) -> Option<bool> {
        dbtx.get_value(&RecoveryFinalizedKey).await
    }

    async fn store_dbtx(&self, dbtx: &mut DatabaseTransaction<'_>, state: &WalletRecoveryState) {
        dbtx.insert_entry(&RecoveryStateKey, state).await;
    }

    async fn check_for_on_chain_deposit(&self, deposit_idx: u64) -> Vec<(Transaction, u32)> {
        let (secret_tweak_key, public_tweak_key, address, operation_id) =
            WalletClientModule::derive_deposit_address_static(
                &self.cfg,
                &self.module_record_secret,
                ChildId(deposit_idx),
            )
            .await;

        let script = bitcoin30_to_bitcoin29_script(
            self.wallet_descriptor
                .tweak(&public_tweak_key, &secp256k1::SECP256K1)
                .script_pubkey(),
        );
        self.rpc.watch_script_history_retry(&script).await;

        for (tx, out_idx) in self.rpc.get_script_history_outputs_retry(&script).await {
            let height = self.rpc.get_tx_block_height_retry(&tx.txid()).await;
            // let self.rpc
        }

        todo!()
    }
}

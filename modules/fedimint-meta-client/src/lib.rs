pub mod api;
#[cfg(feature = "cli")]
pub mod cli;
pub mod db;
pub mod states;

use std::collections::BTreeMap;

use db::DbKeyPrefix;
use fedimint_client::db::ClientMigrationFn;
use fedimint_client::module::init::{ClientModuleInit, ClientModuleInitArgs};
use fedimint_client::module::recovery::NoModuleBackup;
use fedimint_client::module::{ClientContext, ClientModule, IClientModule};
use fedimint_client::sm::Context;
use fedimint_core::core::Decoder;
use fedimint_core::db::{Database, DatabaseTransaction, DatabaseVersion};
use fedimint_core::module::{
    ApiVersion, ModuleCommon, ModuleInit, MultiApiVersion, TransactionItemAmount,
};
use fedimint_core::{apply, async_trait_maybe_send, Amount};
pub use fedimint_meta_common as common;
use fedimint_meta_common::config::MetaClientConfig;
use fedimint_meta_common::{MetaCommonInit, MetaModuleTypes};
use states::MetaStateMachine;
use strum::IntoEnumIterator;

#[derive(Debug)]
pub struct MetaClientModule {
    #[allow(dead_code)]
    cfg: MetaClientConfig,
    #[allow(dead_code)]
    client_ctx: ClientContext<Self>,
    #[allow(dead_code)]
    db: Database,
}

/// Data needed by the state machine
#[derive(Debug, Clone)]
pub struct MetaClientContext {
    pub meta_decoder: Decoder,
}

// TODO: Boiler-plate
impl Context for MetaClientContext {}

#[apply(async_trait_maybe_send!)]
impl ClientModule for MetaClientModule {
    type Init = MetaClientInit;
    type Common = MetaModuleTypes;
    type Backup = NoModuleBackup;
    type ModuleStateMachineContext = MetaClientContext;
    type States = MetaStateMachine;

    fn context(&self) -> Self::ModuleStateMachineContext {
        MetaClientContext {
            meta_decoder: self.decoder(),
        }
    }

    fn input_amount(
        &self,
        _input: &<Self::Common as ModuleCommon>::Input,
    ) -> Option<TransactionItemAmount> {
        unreachable!()
    }

    fn output_amount(
        &self,
        _output: &<Self::Common as ModuleCommon>::Output,
    ) -> Option<TransactionItemAmount> {
        unreachable!()
    }

    fn supports_being_primary(&self) -> bool {
        false
    }

    async fn get_balance(&self, _dbtx: &mut DatabaseTransaction<'_>) -> Amount {
        Amount::ZERO
    }

    #[cfg(feature = "cli")]
    async fn handle_cli_command(
        &self,
        args: &[std::ffi::OsString],
    ) -> anyhow::Result<serde_json::Value> {
        cli::handle_cli_command(self, args).await
    }
}

#[derive(Debug, Clone)]
pub struct MetaClientInit;

// TODO: Boilerplate-code
#[apply(async_trait_maybe_send!)]
impl ModuleInit for MetaClientInit {
    type Common = MetaCommonInit;
    const DATABASE_VERSION: DatabaseVersion = DatabaseVersion(2);

    async fn dump_database(
        &self,
        _dbtx: &mut DatabaseTransaction<'_>,
        prefix_names: Vec<String>,
    ) -> Box<dyn Iterator<Item = (String, Box<dyn erased_serde::Serialize + Send>)> + '_> {
        let items: BTreeMap<String, Box<dyn erased_serde::Serialize + Send>> = BTreeMap::new();
        let filtered_prefixes = DbKeyPrefix::iter().filter(|f| {
            prefix_names.is_empty() || prefix_names.contains(&f.to_string().to_lowercase())
        });

        #[allow(clippy::never_loop)]
        for _table in filtered_prefixes {
            match _table {}
        }

        Box::new(items.into_iter())
    }
}

/// Generates the client module
#[apply(async_trait_maybe_send!)]
impl ClientModuleInit for MetaClientInit {
    type Module = MetaClientModule;

    fn supported_api_versions(&self) -> MultiApiVersion {
        MultiApiVersion::try_from_iter([ApiVersion { major: 0, minor: 0 }])
            .expect("no version conflicts")
    }

    async fn init(&self, args: &ClientModuleInitArgs<Self>) -> anyhow::Result<Self::Module> {
        Ok(MetaClientModule {
            cfg: args.cfg().clone(),
            client_ctx: args.context(),
            db: args.db().clone(),
        })
    }

    fn get_database_migrations(&self) -> BTreeMap<DatabaseVersion, ClientMigrationFn> {
        BTreeMap::new()
    }
}

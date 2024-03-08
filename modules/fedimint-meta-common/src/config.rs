use fedimint_core::core::ModuleKind;
use fedimint_core::encoding::{Decodable, Encodable};
use fedimint_core::{plugin_types_trait_impl_config, Amount};
use serde::{Deserialize, Serialize};

use crate::MetaCommonInit;

/// Parameters necessary to generate this module's configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetaGenParams {
    pub local: MetaGenParamsLocal,
    pub consensus: MetaGenParamsConsensus,
}

/// Local parameters for config generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetaGenParamsLocal(pub String);

/// Consensus parameters for config generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetaGenParamsConsensus {
    pub tx_fee: Amount,
}

impl Default for MetaGenParams {
    fn default() -> Self {
        Self {
            local: MetaGenParamsLocal("example".to_string()),
            consensus: MetaGenParamsConsensus {
                tx_fee: Amount::ZERO,
            },
        }
    }
}

/// Contains all the configuration for the server
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MetaConfig {
    pub local: MetaConfigLocal,
    pub private: MetaConfigPrivate,
    pub consensus: MetaConfigConsensus,
}

/// Contains all the configuration for the client
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, Encodable, Decodable, Hash)]
pub struct MetaClientConfig {}

/// Locally unencrypted config unique to each member
#[derive(Clone, Debug, Serialize, Deserialize, Decodable, Encodable)]
pub struct MetaConfigLocal {}

/// Will be the same for every federation member
#[derive(Clone, Debug, Serialize, Deserialize, Decodable, Encodable)]
pub struct MetaConfigConsensus {}

/// Will be encrypted and not shared such as private key material
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MetaConfigPrivate;

// Wire together the configs for this module
plugin_types_trait_impl_config!(
    MetaCommonInit,
    MetaGenParams,
    MetaGenParamsLocal,
    MetaGenParamsConsensus,
    MetaConfig,
    MetaConfigLocal,
    MetaConfigPrivate,
    MetaConfigConsensus,
    MetaClientConfig
);

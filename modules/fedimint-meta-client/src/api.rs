use fedimint_core::api::{FederationApiExt as _, FederationResult, IModuleFederationApi};
use fedimint_core::module::ApiRequestErased;
use fedimint_core::task::{MaybeSend, MaybeSync};
use fedimint_core::{apply, async_trait_maybe_send};
use fedimint_meta_common::endpoint::{
    GetConsensusRequest, GET_CONSENSUS_ENDPOINT, GET_CONSENSUS_REV_ENDPOINT,
};
use fedimint_meta_common::{MetaConsensusValue, MetaKey, MetaValue};

#[apply(async_trait_maybe_send!)]
pub trait MetaFederationApi {
    async fn get_consensus(&self, key: MetaKey) -> FederationResult<Option<MetaConsensusValue>>;
    async fn get_consensus_rev(&self, key: MetaKey) -> FederationResult<Option<u64>>;
    async fn submit(&self, key: MetaKey, value: MetaValue) -> FederationResult<Option<u64>>;
}

#[apply(async_trait_maybe_send!)]
impl<T: ?Sized> MetaFederationApi for T
where
    T: IModuleFederationApi + MaybeSend + MaybeSync + 'static,
{
    async fn get_consensus(&self, key: MetaKey) -> FederationResult<Option<MetaConsensusValue>> {
        self.request_current_consensus(
            GET_CONSENSUS_ENDPOINT.to_string(),
            ApiRequestErased::new(GetConsensusRequest(key)),
        )
        .await
    }
    async fn get_consensus_rev(&self, key: MetaKey) -> FederationResult<Option<u64>> {
        self.request_current_consensus(
            GET_CONSENSUS_REV_ENDPOINT.to_string(),
            ApiRequestErased::new(GetConsensusRequest(key)),
        )
        .await
    }

    async fn submit(&self, key: MetaKey, value: MetaValue) -> FederationResult<Option<u64>> {
        self.request_current_consensus(
            GET_CONSENSUS_REV_ENDPOINT.to_string(),
            ApiRequestErased::new(GetConsensusRequest(key)),
        )
        .await
    }
}

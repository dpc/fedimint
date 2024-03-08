use fedimint_core::encoding::{Decodable, Encodable};
use fedimint_core::{impl_db_lookup, impl_db_record, PeerId};
use fedimint_meta_common::{MetaKey, MetaValue};
use serde::Serialize;
use strum_macros::EnumIter;

/// Namespaces DB keys for this module
#[repr(u8)]
#[derive(Clone, EnumIter, Debug)]
pub enum DbKeyPrefix {
    /// How we want to vote
    ///
    /// Private, not part of the consensus, but only local state.
    Desired = 0x00,
    /// Current consensuson
    Consensus = 0x01,
    /// Current submitted votes
    Submissions = 0x02,
}

// TODO: Boilerplate-code
impl std::fmt::Display for DbKeyPrefix {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

#[derive(Debug, Clone, Encodable, Decodable, Eq, PartialEq, Hash, Serialize)]
pub struct MetaDesiredKey(pub MetaKey);

#[derive(Debug, Encodable, Decodable)]
pub struct MetaDesiredKeyPrefix;

impl_db_record!(
    key = MetaDesiredKey,
    value = MetaValue,
    db_prefix = DbKeyPrefix::Desired,
);
impl_db_lookup!(key = MetaDesiredKey, query_prefix = MetaDesiredKeyPrefix,);
#[derive(Debug, Clone, Encodable, Decodable, Eq, PartialEq, Hash, Serialize)]
pub struct MetaConsensusKey(pub MetaKey);

#[derive(Debug, Encodable, Decodable, Serialize)]
pub struct MetaConsensusValue {
    pub revision: u64,
    pub value: MetaValue,
}

#[derive(Debug, Encodable, Decodable)]
pub struct MetaConsensusKeyPrefix;

impl_db_record!(
    key = MetaConsensusKey,
    value = MetaConsensusValue,
    db_prefix = DbKeyPrefix::Consensus,
);
impl_db_lookup!(
    key = MetaConsensusKey,
    query_prefix = MetaConsensusKeyPrefix,
);

#[derive(Debug, Clone, Encodable, Decodable, Eq, PartialEq, Hash, Serialize)]
pub struct MetaSubmissionsKey {
    pub key: MetaKey,
    pub peer_id: PeerId,
}

#[derive(Debug, Encodable, Decodable)]
pub struct MetaSubmissionsKeyPrefix;

#[derive(Debug, Encodable, Decodable)]
pub struct MetaSubmissionsByKeyPrefix(pub MetaKey);

impl_db_record!(
    key = MetaSubmissionsKey,
    value = MetaValue,
    db_prefix = DbKeyPrefix::Submissions,
);
impl_db_lookup!(
    key = MetaSubmissionsKey,
    query_prefix = MetaSubmissionsKeyPrefix,
);
impl_db_lookup!(
    key = MetaSubmissionsKey,
    query_prefix = MetaSubmissionsByKeyPrefix,
);

use fedimint_core::encoding::{Decodable, Encodable};
use fedimint_core::impl_db_record;
use serde::Serialize;
use strum_macros::EnumIter;

use crate::recovery::WalletRecoveryState;

#[derive(Clone, EnumIter, Debug)]
pub enum DbKeyPrefix {
    NextPegInTweakIndex = 0x2c,
    RecoveryState = 0x2d,
    RecoveryFinalized = 0x2e,
}

impl std::fmt::Display for DbKeyPrefix {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

#[derive(Clone, Debug, Encodable, Decodable, Serialize)]
pub struct NextPegInTweakIndexKey;

impl_db_record!(
    key = NextPegInTweakIndexKey,
    value = u64,
    db_prefix = DbKeyPrefix::NextPegInTweakIndex,
);

#[derive(Debug, Clone, Encodable, Decodable, Serialize)]
pub struct RecoveryStateKey;

#[derive(Debug, Clone, Encodable, Decodable)]
pub struct RestoreStateKeyPrefix;

impl_db_record!(
    key = RecoveryStateKey,
    value = WalletRecoveryState,
    db_prefix = DbKeyPrefix::RecoveryState,
);
#[derive(Debug, Clone, Encodable, Decodable, Serialize)]
pub struct RecoveryFinalizedKey;

#[derive(Debug, Clone, Encodable, Decodable)]
pub struct RecoveryFinalizedKeyPrefix;

impl_db_record!(
    key = RecoveryFinalizedKey,
    value = bool,
    db_prefix = DbKeyPrefix::RecoveryFinalized,
);

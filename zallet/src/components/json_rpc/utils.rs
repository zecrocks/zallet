use std::collections::HashSet;

use jsonrpsee::{
    core::{JsonValue, RpcResult},
    types::ErrorCode as RpcErrorCode,
};
use zcash_client_backend::data_api::{Account, WalletRead};
use zcash_client_sqlite::AccountUuid;
use zip32::DiversifierIndex;

use crate::components::database::DbConnection;

use super::server::LegacyCode;

/// The account identifier used for HD derivation of transparent and Sapling addresses via
/// the legacy `getnewaddress` and `z_getnewaddress` code paths.
const ZCASH_LEGACY_ACCOUNT: u32 = 0x7fff_ffff;

/// Parses the `account` parameter present in many wallet RPCs.
pub(super) fn parse_account_parameter(
    wallet: &DbConnection,
    account: &JsonValue,
) -> RpcResult<AccountUuid> {
    match account {
        // This might be a ZIP 32 account index (how zcashd accepted it).
        JsonValue::Number(n) => {
            let zip32_account_index = n
                .as_u64()
                .filter(|n| n < &ZCASH_LEGACY_ACCOUNT.into())
                .ok_or_else(|| {
                    LegacyCode::InvalidParameter
                        .with_static("Invalid account number, must be 0 <= account <= (2^31)-2.")
                })?;

            let mut distinct_seeds = HashSet::new();
            let mut account_id = None;

            for candidate_account_id in wallet
                .get_account_ids()
                .map_err(|e| LegacyCode::Database.with_message(e.to_string()))?
            {
                let account = wallet
                    .get_account(candidate_account_id)
                    .map_err(|e| LegacyCode::Database.with_message(e.to_string()))?
                    // This would be a race condition between this and account deletion.
                    .ok_or(RpcErrorCode::InternalError)?;

                // Ignore accounts from imported keys. `zcashd` did not support importing
                // UFVKs as "accounts"; the latter always descended from the single seed.
                if let Some(derivation) = account.source().key_derivation() {
                    distinct_seeds.insert(*derivation.seed_fingerprint());
                    if u64::from(u32::from(derivation.account_index())) == zip32_account_index {
                        account_id = Some(candidate_account_id);
                    }
                }
            }

            if distinct_seeds.len() == 1 {
                account_id.ok_or_else(|| {
                    LegacyCode::Wallet.with_message(format!(
                        "Error: account {} has not been generated by z_getnewaccount.",
                        zip32_account_index
                    ))
                })
            } else {
                Err(LegacyCode::Wallet.with_static("Account numbers are not supported in wallets with multiple seeds. Use the account UUID instead."))
            }
        }
        // This might be an account UUID.
        JsonValue::String(s) => s
            .parse()
            .map(AccountUuid::from_uuid)
            .map_err(|_| RpcErrorCode::InvalidParams.into()),
        _ => Err(RpcErrorCode::InvalidParams.into()),
    }
}

/// Parses the `diversifier_index` parameter present in many wallet RPCs.
pub(super) fn parse_diversifier_index(diversifier_index: u128) -> RpcResult<DiversifierIndex> {
    diversifier_index
        .try_into()
        .map_err(|_| LegacyCode::InvalidParameter.with_static("diversifier index is too large."))
}

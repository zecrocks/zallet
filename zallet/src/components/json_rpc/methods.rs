use async_trait::async_trait;
use jsonrpsee::{
    core::{JsonValue, RpcResult},
    proc_macros::rpc,
};

use crate::components::database::{Database, DbHandle};

mod get_address_for_account;
mod get_notes_count;
mod get_wallet_info;
mod list_accounts;
mod list_addresses;
mod list_unified_receivers;
mod list_unspent;

#[rpc(server)]
pub(crate) trait Rpc {
    #[method(name = "getwalletinfo")]
    fn get_wallet_info(&self) -> get_wallet_info::Response;

    #[method(name = "z_listaccounts")]
    async fn list_accounts(&self) -> list_accounts::Response;

    /// For the given account, derives a Unified Address in accordance with the remaining
    /// arguments:
    ///
    /// - If no list of receiver types is given (or the empty list `[]`), the best and
    ///   second-best shielded receiver types, along with the "p2pkh" (i.e. transparent)
    ///   receiver type, will be used.
    /// - If no diversifier index is given, then:
    ///   - If a transparent receiver would be included (either because no list of
    ///     receiver types is given, or the provided list includes "p2pkh"), the next
    ///     unused index (that is valid for the list of receiver types) will be selected.
    ///   - If only shielded receivers would be included (because a list of receiver types
    ///     is given that does not include "p2pkh"), a time-based index will be selected.
    ///
    /// The account parameter must be a UUID or account number that was previously
    /// generated by a call to the `z_getnewaccount` RPC method. The legacy account number
    /// is only supported for wallets containing a single seed phrase.
    ///
    /// Once a Unified Address has been derived at a specific diversifier index,
    /// re-deriving it (via a subsequent call to `z_getaddressforaccount` with the same
    /// account and index) will produce the same address with the same list of receiver
    /// types. An error will be returned if a different list of receiver types is
    /// requested, including when the empty list `[]` is provided (if the default receiver
    /// types don't match).
    #[method(name = "z_getaddressforaccount")]
    async fn get_address_for_account(
        &self,
        account: JsonValue,
        receiver_types: Option<Vec<String>>,
        diversifier_index: Option<u128>,
    ) -> get_address_for_account::Response;

    /// Lists the addresses managed by this wallet by source.
    ///
    /// Sources include:
    /// - Addresses generated from randomness by a legacy `zcashd` wallet.
    /// - Sapling addresses generated from the legacy `zcashd` HD seed.
    /// - Imported watchonly transparent addresses.
    /// - Shielded addresses tracked using imported viewing keys.
    /// - Addresses derived from mnemonic seed phrases.
    ///
    /// In the case that a source does not have addresses for a value pool, the key
    /// associated with that pool will be absent.
    ///
    /// REMINDER: It is recommended that you back up your wallet files regularly. If you
    /// have not imported externally-produced keys, it only necessary to have backed up
    /// the wallet's key storage file.
    #[method(name = "listaddresses")]
    async fn list_addresses(&self) -> list_addresses::Response;

    #[method(name = "z_listunifiedreceivers")]
    fn list_unified_receivers(&self, unified_address: &str) -> list_unified_receivers::Response;

    /// Returns an array of unspent shielded notes with between minconf and maxconf
    /// (inclusive) confirmations.
    ///
    /// Results may be optionally filtered to only include notes sent to specified
    /// addresses. When `minconf` is 0, unspent notes with zero confirmations are
    /// returned, even though they are not immediately spendable.
    ///
    /// # Arguments
    /// - `minconf` (default = 1)
    #[method(name = "z_listunspent")]
    async fn list_unspent(&self) -> list_unspent::Response;

    #[method(name = "z_getnotescount")]
    async fn get_notes_count(
        &self,
        minconf: Option<u32>,
        as_of_height: Option<i32>,
    ) -> get_notes_count::Response;
}

pub(crate) struct RpcImpl {
    wallet: Database,
}

impl RpcImpl {
    /// Creates a new instance of the RPC handler.
    pub(crate) fn new(wallet: Database) -> Self {
        Self { wallet }
    }

    async fn wallet(&self) -> RpcResult<DbHandle> {
        self.wallet
            .handle()
            .await
            .map_err(|_| jsonrpsee::types::ErrorCode::InternalError.into())
    }
}

#[async_trait]
impl RpcServer for RpcImpl {
    fn get_wallet_info(&self) -> get_wallet_info::Response {
        get_wallet_info::call()
    }

    async fn list_accounts(&self) -> list_accounts::Response {
        list_accounts::call(self.wallet().await?.as_ref())
    }

    async fn get_address_for_account(
        &self,
        account: JsonValue,
        receiver_types: Option<Vec<String>>,
        diversifier_index: Option<u128>,
    ) -> get_address_for_account::Response {
        get_address_for_account::call(
            self.wallet().await?.as_mut(),
            account,
            receiver_types,
            diversifier_index,
        )
    }

    async fn list_addresses(&self) -> list_addresses::Response {
        list_addresses::call(self.wallet().await?.as_ref())
    }

    fn list_unified_receivers(&self, unified_address: &str) -> list_unified_receivers::Response {
        list_unified_receivers::call(unified_address)
    }

    async fn list_unspent(&self) -> list_unspent::Response {
        list_unspent::call(self.wallet().await?.as_ref())
    }

    async fn get_notes_count(
        &self,
        minconf: Option<u32>,
        as_of_height: Option<i32>,
    ) -> get_notes_count::Response {
        get_notes_count::call(self.wallet().await?.as_ref(), minconf, as_of_height)
    }
}

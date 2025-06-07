use std::collections::HashMap;

use bs58_fixed_wasm::Bs58Array;
use sanctum_spl_stake_pool_core::StakeAccountLamports;
use serde::{Deserialize, Serialize};
use tsify_next::Tsify;
use wasm_bindgen::{prelude::wasm_bindgen, JsError};

use crate::err::account_missing_err;

#[tsify_next::declare]
pub type B58PK = Bs58Array<32, 44>;

#[derive(Debug, Default, Clone, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub struct AccountMap(pub HashMap<B58PK, OwnedAccount>);

#[derive(Debug, Default, Clone, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi, large_number_types_as_bigints)]
#[serde(rename_all = "camelCase")]
pub struct OwnedAccount {
    pub owner: B58PK,
    #[tsify(type = "Uint8Array")] // Instead of number[]
    pub data: Box<[u8]>,
    pub lamports: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi, large_number_types_as_bigints)]
#[serde(rename_all = "camelCase")]
pub struct QuoteParams {
    pub amount: u64,
    pub input_mint: B58PK,
    pub output_mint: B58PK,
}

#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi, large_number_types_as_bigints)]
#[serde(rename_all = "camelCase")]
pub struct DepositStakeParams {
    pub validator_vote: B58PK,
    pub output_mint: B58PK,
    pub stake_account_lamports: StakeAccountLamports,
}

#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi, large_number_types_as_bigints)]
#[serde(rename_all = "camelCase")]
pub struct SwapParams {
    pub amount: u64,
    /// Can either be a mint or a vote account
    pub source: B58PK,
    pub destination_mint: B58PK,
    pub source_token_account: B58PK,
    pub destination_token_account: B58PK,
    /// This can be the user or the program authority over the source_token_account.
    pub token_transfer_authority: B58PK,
}

#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub struct SplPoolAccounts {
    pub pool: B58PK,
    pub validator_list: B58PK,
}

pub(crate) fn get_account(
    accounts: &AccountMap,
    pubkey: [u8; 32],
) -> Result<&OwnedAccount, JsError> {
    accounts
        .0
        .get(&B58PK::new(pubkey))
        .ok_or(account_missing_err(&pubkey))
}

pub(crate) fn get_account_data(accounts: &AccountMap, pubkey: [u8; 32]) -> Result<&[u8], JsError> {
    get_account(accounts, pubkey).map(|account| account.data.as_ref())
}

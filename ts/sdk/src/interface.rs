use std::collections::HashMap;

use bs58_fixed::Bs58String;
use bs58_fixed_wasm::Bs58Array;
use sanctum_router_core::{
    DepositStakeQuote, Prefund, StakeAccountLamports, TokenQuote, WithRouterFee,
};
use serde::{Deserialize, Serialize};
use tsify_next::Tsify;
use wasm_bindgen::{prelude::wasm_bindgen, JsError};

use crate::err::account_missing_err;

#[tsify_next::declare]
pub type B58PK = Bs58Array<32, 44>;

pub type Bs58PkString = Bs58String<44>;

pub enum Role {
    Readonly,
    Writable,
    ReadonlySigner,
    WritableSigner,
}

impl Role {
    pub const fn from_signer_writable(signer: bool, writable: bool) -> Self {
        match (signer, writable) {
            (true, true) => Self::WritableSigner,
            (true, false) => Self::ReadonlySigner,
            (false, true) => Self::Writable,
            (false, false) => Self::Readonly,
        }
    }

    pub const fn as_u8(&self) -> u8 {
        match self {
            Self::Readonly => 0,
            Self::Writable => 1,
            Self::ReadonlySigner => 2,
            Self::WritableSigner => 3,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct AccountMeta {
    pub address: B58PK,

    /// Represents the role of an account in a transaction:
    /// - Readonly: 0
    /// - Writable: 1
    /// - ReadonlySigner: 2
    /// - WritableSigner: 3
    #[tsify(type = "0 | 1 | 2 | 3")]
    pub role: u8,
}

impl AccountMeta {
    pub(crate) const fn new(address: [u8; 32], role: Role) -> Self {
        Self {
            address: B58PK::new(address),
            role: role.as_u8(),
        }
    }
}

pub fn keys_signer_writer_to_account_metas<const N: usize>(
    keys: &[&[u8; 32]; N],
    signer: &[bool; N],
    writer: &[bool; N],
) -> [AccountMeta; N] {
    core::array::from_fn(|i| {
        let k = keys[i];
        AccountMeta::new(*k, Role::from_signer_writable(signer[i], writer[i]))
    })
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub struct Instruction {
    pub data: Box<[u8]>,
    pub accounts: Box<[AccountMeta]>,
    pub program_address: B58PK,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub struct AccountMap(pub HashMap<B58PK, OwnedAccount>);

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
pub struct TokenQuoteParams {
    pub amt: u64,
    pub inp_mint: B58PK,
    pub out_mint: B58PK,
}

// need to use a simple newtype here instead of type alias
// otherwise wasm_bindgen shits itself with missing generics
#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi, large_number_types_as_bigints)]
#[serde(rename_all = "camelCase")]
pub struct TokenQuoteWithRouterFee(pub(crate) WithRouterFee<TokenQuote>);

#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi, large_number_types_as_bigints)]
#[serde(rename_all = "camelCase")]
pub struct TokenSwapParams {
    pub amt: u64,

    /// Input mint
    pub inp: B58PK,

    /// Output mint
    pub out: B58PK,

    /// Input token account to transfer `amt` tokens from
    pub signer_inp: B58PK,

    /// Output token account to receive tokens to
    pub signer_out: B58PK,

    /// Signing authority of `self.signer_inp`; user making the swap.
    pub signer: B58PK,
}

#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi, large_number_types_as_bigints)]
#[serde(rename_all = "camelCase")]
pub struct DepositStakeQuoteParams {
    /// Validator vote account `inp_stake` is delegated to
    pub vote: B58PK,
    pub inp_stake: StakeAccountLamports,
    pub out_mint: B58PK,
}

// need to use a simple newtype here instead of type alias
// otherwise wasm_bindgen shits itself with missing generics
#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi, large_number_types_as_bigints)]
#[serde(rename_all = "camelCase")]
pub struct DepositStakeQuoteWithRouterFee(pub(crate) WithRouterFee<DepositStakeQuote>);

#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi, large_number_types_as_bigints)]
#[serde(rename_all = "camelCase")]
pub struct DepositStakeSwapParams {
    /// Vote account `self.signer_inp` stake account is delegated to
    pub inp: B58PK,

    /// Output mint
    pub out: B58PK,

    /// Stake account to deposit
    pub signer_inp: B58PK,

    /// Output token account to receive tokens to
    pub signer_out: B58PK,

    /// Signing authority of `self.signer_inp`; user making the swap.
    pub signer: B58PK,
}

#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi, large_number_types_as_bigints)]
#[serde(rename_all = "camelCase")]
pub struct WithdrawStakeQuoteParams {
    pub amt: u64,
    pub inp_mint: B58PK,

    /// Desired vote account of `out_stake`.
    /// If omitted, then any vote account of any validator in the stake pool
    /// may be used
    #[tsify(optional)]
    pub out_vote: Option<B58PK>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi, large_number_types_as_bigints)]
#[serde(rename_all = "camelCase")]
pub struct WithdrawStakeQuote {
    /// Input pool tokens
    pub inp: u64,

    /// The stake account that will be withdrawn
    pub out: StakeAccountLamports,

    /// In terms of input tokens, charged by the stake pool
    pub fee: u64,

    /// Validator vote account `out` stake account will be delegated to
    pub vote: B58PK,
}

// need to use a simple newtype here instead of type alias
// otherwise wasm_bindgen shits itself with missing generics
#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi, large_number_types_as_bigints)]
#[serde(rename_all = "camelCase")]
pub struct PrefundWithdrawStakeQuote(pub(crate) Prefund<WithdrawStakeQuote>);

#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi, large_number_types_as_bigints)]
#[serde(rename_all = "camelCase")]
pub struct WithdrawStakeSwapParams {
    pub amt: u64,

    /// Input mint
    pub inp: B58PK,

    /// Vote account the withdrawn stake account will be delegated to
    pub out: B58PK,

    /// Input token account to transfer `amt` tokens from
    pub signer_inp: B58PK,

    /// Bridge stake seed of the stake account to withdraw
    pub bridge_stake_seed: u32,

    /// Signing authority of `self.signer_inp`; user making the swap.
    pub signer: B58PK,
}

#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub struct SplPoolAccounts {
    pub pool: B58PK,
    pub validator_list: B58PK,
}

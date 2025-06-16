//! Common typescript interface types that are imported in more than one module

use std::collections::HashMap;

use bs58_fixed::Bs58String;
use bs58_fixed_wasm::Bs58Array;
use serde::{Deserialize, Serialize};
use serde_bytes::ByteBuf;
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

#[inline] // inlining reduces binary size by a bit
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
    pub data: ByteBuf,
    pub accounts: Box<[AccountMeta]>,
    pub program_address: B58PK,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub struct AccountMap(pub HashMap<B58PK, Account>);

// pass pubkey by value instead of ref to accomodate B58PK::new
pub(crate) fn get_account(accounts: &AccountMap, pubkey: [u8; 32]) -> Result<&Account, JsError> {
    accounts
        .0
        .get(&B58PK::new(pubkey))
        .ok_or_else(|| account_missing_err(&pubkey))
}

/// Basically HashMap.get(), but returns [`account_missing_err()`] if account missing instead of `None`
pub(crate) fn get_account_data(accounts: &AccountMap, pubkey: [u8; 32]) -> Result<&[u8], JsError> {
    get_account(accounts, pubkey).map(|account| account.data.as_ref())
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi, large_number_types_as_bigints)]
#[serde(rename_all = "camelCase")]
pub struct Account {
    pub owner: B58PK,
    pub data: ByteBuf,
    pub lamports: u64,
}

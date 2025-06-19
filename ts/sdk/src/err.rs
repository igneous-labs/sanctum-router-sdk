use std::fmt::Display;

use wasm_bindgen::{intern, JsError};

use crate::{interface::Bs58PkString, update::PoolUpdateType};

pub fn account_missing_err(pubkey: &[u8; 32]) -> JsError {
    JsError::new(&format!(
        "Account {} missing from AccountMap",
        Bs58PkString::encode(pubkey)
    ))
}

pub fn invalid_pda_err() -> JsError {
    JsError::new(intern("Invalid PDA"))
}

pub fn invalid_data_err() -> JsError {
    JsError::new(intern("Invalid data"))
}

// TODO: mint as arg
pub fn router_missing_err() -> JsError {
    JsError::new(intern("Router missing from Sanctum Router"))
}

pub fn generic_err(e: impl Display) -> JsError {
    JsError::new(&format!("{e}"))
}

pub fn unsupported_update(ty: PoolUpdateType, mint: &[u8; 32]) -> JsError {
    JsError::new(&format!(
        "{:?} not supported by pool of mint {}",
        ty,
        Bs58PkString::encode(mint)
    ))
}

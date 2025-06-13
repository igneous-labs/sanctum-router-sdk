use std::fmt::Display;

use wasm_bindgen::{intern, JsError};

use crate::interface::Bs58PkString;

pub fn account_missing_err(pubkey: &[u8; 32]) -> JsError {
    JsError::new(intern(&format!(
        "Account {:?} missing from AccountMap",
        Bs58PkString::encode(pubkey).to_string()
    )))
}

pub fn invalid_pda_err() -> JsError {
    JsError::new(intern("Invalid PDA"))
}

pub fn invalid_data_err() -> JsError {
    JsError::new(intern("Invalid data"))
}

pub fn router_missing_err() -> JsError {
    JsError::new(intern("Router missing from Sanctum Router"))
}

pub fn generic_err(e: impl Display) -> JsError {
    JsError::new(&format!("{e}"))
}

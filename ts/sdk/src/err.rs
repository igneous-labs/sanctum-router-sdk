use bs58_fixed::Bs58String;
use wasm_bindgen::{intern, JsError};

pub fn account_missing_err(pubkey: &[u8; 32]) -> JsError {
    JsError::new(intern(&format!(
        "Account {:?} missing from AccountMap",
        Bs58String::<44>::encode(pubkey).to_string()
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

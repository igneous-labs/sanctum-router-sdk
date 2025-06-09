use wasm_bindgen::JsError;

use crate::interface::AccountMap;

pub trait Update {
    fn get_accounts_to_update(&self) -> impl Iterator<Item = [u8; 32]>;

    fn update(&mut self, accounts: &AccountMap) -> Result<(), JsError>;
}

use bs58_fixed_wasm::Bs58Array;
use sanctum_router_core::{fee_token_acc_seeds, SANCTUM_ROUTER_PROGRAM};
use wasm_bindgen::prelude::*;

use crate::{
    err::invalid_pda_err,
    interface::B58PK,
    pda::{find_pda, FoundPda},
};

pub fn find_fee_token_account_pda_internal(mint: &[u8; 32]) -> Option<([u8; 32], u8)> {
    let (s1, s2) = fee_token_acc_seeds(mint);
    find_pda(&[s1.as_slice(), s2.as_slice()], &SANCTUM_ROUTER_PROGRAM)
}

#[wasm_bindgen(js_name = findFeeTokenAccountPda)]
pub fn find_fee_token_account_pda(Bs58Array(mint): &B58PK) -> Result<FoundPda, JsError> {
    find_fee_token_account_pda_internal(mint)
        .ok_or_else(invalid_pda_err)
        .map(|(p, b)| FoundPda(B58PK::new(p), b))
}

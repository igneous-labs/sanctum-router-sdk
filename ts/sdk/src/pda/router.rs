use bs58_fixed_wasm::Bs58Array;
use sanctum_router_core::{
    bridge_stake_seeds, fee_token_acc_seeds, SANCTUM_ROUTER_PROGRAM, SLUMDOG_SEED, STAKE_PROGRAM,
};
use wasm_bindgen::prelude::*;

use crate::{
    err::invalid_pda_err,
    interface::B58PK,
    pda::{find_pda, pk_create_with_seed, FoundPda},
};

pub fn find_fee_token_account_pda_internal(mint: &[u8; 32]) -> Option<([u8; 32], u8)> {
    let (s1, s2) = fee_token_acc_seeds(mint);
    find_pda(&[s1.as_slice(), s2.as_slice()], &SANCTUM_ROUTER_PROGRAM)
}

/// @param {B58PK} arg0 mint pubkey
#[wasm_bindgen(js_name = findFeeTokenAccountPda)]
pub fn find_fee_token_account_pda(Bs58Array(mint): &B58PK) -> Result<FoundPda, JsError> {
    find_fee_token_account_pda_internal(mint)
        .ok_or_else(invalid_pda_err)
        .map(|(p, b)| FoundPda(B58PK::new(p), b))
}

pub fn find_bridge_stake_acc_internal(
    user: &[u8; 32],
    bridge_stake_seed: u32,
) -> Option<([u8; 32], u8)> {
    let (s1, s2, s3) = bridge_stake_seeds(user, bridge_stake_seed);
    find_pda(
        &[s1.as_slice(), s2.as_slice(), s3.as_slice()],
        &SANCTUM_ROUTER_PROGRAM,
    )
}

/// @param {B58PK} arg0 user pubkey
/// @param {number} bridge_stake_seed u32 bridge stake seed
#[wasm_bindgen(js_name = findBridgeStakeAccPda)]
pub fn find_bridge_stake_acc_pda(
    Bs58Array(user): &B58PK,
    bridge_stake_seed: u32,
) -> Result<FoundPda, JsError> {
    find_bridge_stake_acc_internal(user, bridge_stake_seed)
        .ok_or_else(invalid_pda_err)
        .map(|(p, b)| FoundPda(B58PK::new(p), b))
}

pub fn create_slumdog_stake(bridge_stake: &[u8; 32]) -> [u8; 32] {
    // unwrap-safety:
    // - seed.len() <= MAX_SEED_LEN
    // - Stake program ID's last bytes are not PDA_MARKER
    pk_create_with_seed(bridge_stake, SLUMDOG_SEED, &STAKE_PROGRAM).unwrap()
}

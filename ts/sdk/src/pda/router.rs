use sanctum_router_core::{fee_token_acc_seeds, SANCTUM_ROUTER_PROGRAM};

use crate::pda::find_pda;

pub fn find_fee_token_account_pda_internal(mint: &[u8; 32]) -> Option<([u8; 32], u8)> {
    let (s1, s2) = fee_token_acc_seeds(mint);
    find_pda(&[s1.as_slice(), s2.as_slice()], &SANCTUM_ROUTER_PROGRAM)
}

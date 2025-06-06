use sanctum_router_core::{fee_token_acc_seeds, SANCTUM_ROUTER_PROGRAM};
use solana_sdk::pubkey::Pubkey;

#[inline]
pub(crate) fn find_fee_token_account(mint: &[u8; 32]) -> [u8; 32] {
    let (s1, s2) = fee_token_acc_seeds(mint);
    Pubkey::find_program_address(&[s1, s2], &Pubkey::from(SANCTUM_ROUTER_PROGRAM))
        .0
        .to_bytes()
}

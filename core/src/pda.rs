pub const FEE_SEED: [u8; 3] = *b"fee";
pub const BRIDGE_STAKE_SEED: [u8; 12] = *b"bridge_stake";

/// `create_with_seed(bridge_stake.pubkey, SLUMDOG_SEED, stake_program)`
/// to obtain the slumdog stake account address
pub const SLUMDOG_SEED: &str = "slumdog";

#[inline]
pub const fn fee_token_acc_seeds(mint: &[u8; 32]) -> (&[u8; 3], &[u8; 32]) {
    (&FEE_SEED, mint)
}

#[inline]
pub const fn bridge_stake_seeds(
    user: &[u8; 32],
    bridge_stake_seed: u32,
) -> (&[u8; 12], &[u8; 32], [u8; 4]) {
    (&BRIDGE_STAKE_SEED, user, bridge_stake_seed.to_le_bytes())
}

pub const FEE_SEED: [u8; 3] = *b"fee";

#[inline]
pub const fn fee_token_acc_seeds(mint: &[u8; 32]) -> (&[u8; 3], &[u8; 32]) {
    (&FEE_SEED, mint)
}

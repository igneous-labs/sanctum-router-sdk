mod stake_wrapped_sol;

pub use stake_wrapped_sol::*;

use solana_sdk::{instruction::AccountMeta, pubkey::Pubkey};

pub(crate) fn metas_from_keys_signer_writer<const N: usize>(
    keys: [[u8; 32]; N],
    is_signer: [bool; N],
    is_writer: [bool; N],
) -> [AccountMeta; N] {
    core::array::from_fn(|i| AccountMeta {
        pubkey: Pubkey::new_from_array(keys[i]),
        is_signer: is_signer[i],
        is_writable: is_writer[i],
    })
}

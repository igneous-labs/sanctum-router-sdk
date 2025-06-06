use solana_sdk::{instruction::AccountMeta, pubkey::Pubkey};

pub(crate) fn keys_signer_writer_to_account_metas<'a>(
    keys: &'a [[u8; 32]],
    signer: &'a [bool],
    writer: &'a [bool],
) -> impl Iterator<Item = AccountMeta> + 'a {
    keys.iter().enumerate().map(|(i, k)| AccountMeta {
        pubkey: Pubkey::from(*k),
        is_signer: signer[i],
        is_writable: writer[i],
    })
}

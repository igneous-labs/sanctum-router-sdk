use crate::find_fee_token_account;

use super::metas_from_keys_signer_writer;
use sanctum_router_core::{
    StakeWrappedSolPrefixKeysOwned, STAKE_WRAPPED_SOL_PREFIX_ACCS_LEN,
    STAKE_WRAPPED_SOL_PREFIX_IS_SIGNER, STAKE_WRAPPED_SOL_PREFIX_IS_WRITER, TOKEN_PROGRAM,
};
use solana_sdk::instruction::AccountMeta;

pub fn stake_wrapped_sol_prefix_metas(
    swap_params: &jupiter_amm_interface::SwapParams,
) -> [AccountMeta; STAKE_WRAPPED_SOL_PREFIX_ACCS_LEN] {
    let keys = StakeWrappedSolPrefixKeysOwned::default()
        .with_consts()
        .with_user(swap_params.token_transfer_authority.to_bytes())
        .with_wsol_mint(swap_params.source_mint.to_bytes())
        .with_out_mint(swap_params.destination_mint.to_bytes())
        .with_inp_wsol(swap_params.source_token_account.to_bytes())
        .with_out_token(swap_params.destination_token_account.to_bytes())
        .with_out_fee_token(find_fee_token_account(
            &swap_params.destination_mint.to_bytes(),
        ))
        .with_token_program(TOKEN_PROGRAM);

    metas_from_keys_signer_writer(
        keys.0,
        STAKE_WRAPPED_SOL_PREFIX_IS_SIGNER.0,
        STAKE_WRAPPED_SOL_PREFIX_IS_WRITER.0,
    )
}

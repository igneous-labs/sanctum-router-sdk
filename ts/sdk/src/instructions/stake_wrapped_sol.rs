use sanctum_router_core::{
    StakeWrappedSolIxData, StakeWrappedSolPrefixKeysOwned, STAKE_WRAPPED_SOL_PREFIX_ACCS_LEN,
    STAKE_WRAPPED_SOL_PREFIX_IS_SIGNER, STAKE_WRAPPED_SOL_PREFIX_IS_WRITER, TOKEN_PROGRAM,
};
use spl_stake_pool::{keys_signer_writer_to_account_metas, AccountMeta};
use wasm_bindgen::JsError;

use crate::{err::invalid_pda_err, pda::find_fee_token_account_pda_internal, utils::SwapParams};

pub(crate) fn get_deposit_sol_prefix_metas_and_data(
    swap_params: SwapParams,
) -> Result<
    (
        [AccountMeta; STAKE_WRAPPED_SOL_PREFIX_ACCS_LEN],
        StakeWrappedSolIxData,
    ),
    JsError,
> {
    let metas = keys_signer_writer_to_account_metas(
        &StakeWrappedSolPrefixKeysOwned::default()
            .with_consts()
            .with_user(swap_params.token_transfer_authority.0)
            .with_wsol_mint(swap_params.source.0)
            .with_dest_token_mint(swap_params.destination_mint.0)
            .with_wsol_from(swap_params.source_token_account.0)
            .with_dest_token_to(swap_params.destination_token_account.0)
            .with_token_program(TOKEN_PROGRAM)
            .with_dest_token_fee_token_account(
                find_fee_token_account_pda_internal(&swap_params.destination_mint.0)
                    .ok_or(invalid_pda_err())?
                    .0,
            )
            .as_borrowed()
            .0,
        &STAKE_WRAPPED_SOL_PREFIX_IS_SIGNER.0,
        &STAKE_WRAPPED_SOL_PREFIX_IS_WRITER.0,
    );

    let data = StakeWrappedSolIxData::new(swap_params.amount);

    Ok((metas, data))
}

use sanctum_router_core::{
    StakeWrappedSolIxData, StakeWrappedSolPrefixKeysOwned, STAKE_WRAPPED_SOL_PREFIX_ACCS_LEN,
    STAKE_WRAPPED_SOL_PREFIX_IS_SIGNER, STAKE_WRAPPED_SOL_PREFIX_IS_WRITER, TOKEN_PROGRAM,
};
use wasm_bindgen::JsError;

use crate::{
    err::invalid_pda_err,
    interface::{keys_signer_writer_to_account_metas, AccountMeta, TokenSwapParams},
    pda::router::find_fee_token_account_pda_internal,
};

pub(crate) fn get_deposit_sol_prefix_metas_and_data(
    swap_params: TokenSwapParams,
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
            .with_user(swap_params.signer.0)
            .with_wsol_mint(swap_params.inp.0)
            .with_out_mint(swap_params.out.0)
            .with_inp_wsol(swap_params.signer_inp.0)
            .with_out_token(swap_params.signer_out.0)
            .with_token_program(TOKEN_PROGRAM)
            .with_out_fee_token(
                find_fee_token_account_pda_internal(&swap_params.out.0)
                    .ok_or(invalid_pda_err())?
                    .0,
            )
            .as_borrowed()
            .0,
        &STAKE_WRAPPED_SOL_PREFIX_IS_SIGNER.0,
        &STAKE_WRAPPED_SOL_PREFIX_IS_WRITER.0,
    );

    let data = StakeWrappedSolIxData::new(swap_params.amt);

    Ok((metas, data))
}

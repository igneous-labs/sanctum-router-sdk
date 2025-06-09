use sanctum_router_core::{
    WithdrawWrappedSolIxData, WithdrawWrappedSolPrefixAccsBuilder, TOKEN_PROGRAM,
    WITHDRAW_WRAPPED_SOL_PREFIX_ACCS_LEN, WITHDRAW_WRAPPED_SOL_PREFIX_IS_SIGNER,
    WITHDRAW_WRAPPED_SOL_PREFIX_IS_WRITER,
};
use wasm_bindgen::JsError;

use crate::{
    err::invalid_pda_err,
    interface::{keys_signer_writer_to_account_metas, AccountMeta, SwapParams},
    pda::router::find_fee_token_account_pda_internal,
};

pub(crate) fn get_withdraw_wrapped_sol_prefix_metas_and_data(
    swap_params: SwapParams,
) -> Result<
    (
        [AccountMeta; WITHDRAW_WRAPPED_SOL_PREFIX_ACCS_LEN],
        WithdrawWrappedSolIxData,
    ),
    JsError,
> {
    let metas = keys_signer_writer_to_account_metas(
        &WithdrawWrappedSolPrefixAccsBuilder::start()
            .with_user(&swap_params.token_transfer_authority.0)
            .with_src_token_from(&swap_params.source_token_account.0)
            .with_wsol_to(&swap_params.destination_token_account.0)
            .with_wsol_fee_token_account(
                &find_fee_token_account_pda_internal(&swap_params.destination_mint.0)
                    .ok_or(invalid_pda_err())?
                    .0,
            )
            .with_src_token_mint(&swap_params.source.0)
            .with_wsol_mint(&swap_params.destination_mint.0)
            .with_token_program(&TOKEN_PROGRAM)
            .build()
            .0,
        &WITHDRAW_WRAPPED_SOL_PREFIX_IS_SIGNER.0,
        &WITHDRAW_WRAPPED_SOL_PREFIX_IS_WRITER.0,
    );

    let data = WithdrawWrappedSolIxData::new(swap_params.amount);

    Ok((metas, data))
}

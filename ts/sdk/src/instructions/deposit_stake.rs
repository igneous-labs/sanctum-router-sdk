use sanctum_router_core::{
    DepositStakeIxAccsBuilder, DepositStakeIxData, DEPOSIT_STAKE_IX_ACCS_LEN,
    DEPOSIT_STAKE_IX_IS_SIGNER, DEPOSIT_STAKE_IX_IS_WRITER,
};
use wasm_bindgen::JsError;

use crate::{
    err::invalid_pda_err,
    interface::{keys_signer_writer_to_account_metas, AccountMeta, SwapParams},
    pda::router::find_fee_token_account_pda_internal,
};

pub(crate) fn get_deposit_stake_prefix_metas_and_data(
    swap_params: SwapParams,
) -> Result<([AccountMeta; DEPOSIT_STAKE_IX_ACCS_LEN], DepositStakeIxData), JsError> {
    let metas = keys_signer_writer_to_account_metas(
        &DepositStakeIxAccsBuilder::start()
            .with_user(swap_params.token_transfer_authority.0)
            .with_out_token(swap_params.destination_token_account.0)
            .with_out_fee_token(
                find_fee_token_account_pda_internal(&swap_params.destination_mint.0)
                    .ok_or(invalid_pda_err())?
                    .0,
            )
            .with_out_mint(swap_params.destination_mint.0)
            .with_inp_stake(swap_params.source_token_account.0)
            .build()
            .as_borrowed()
            .0,
        &DEPOSIT_STAKE_IX_IS_SIGNER.0,
        &DEPOSIT_STAKE_IX_IS_WRITER.0,
    );

    let data = DepositStakeIxData::new();

    Ok((metas, data))
}

use sanctum_router_core::{
    WithdrawSolQuoter, WithdrawSolSufAccs, WithdrawWrappedSolIxData,
    WithdrawWrappedSolPrefixAccsBuilder, NATIVE_MINT, SANCTUM_ROUTER_PROGRAM, TOKEN_PROGRAM,
    WITHDRAW_WRAPPED_SOL_PREFIX_ACCS_LEN, WITHDRAW_WRAPPED_SOL_PREFIX_IS_SIGNER,
    WITHDRAW_WRAPPED_SOL_PREFIX_IS_WRITER,
};
use serde::{Deserialize, Serialize};
use serde_bytes::ByteBuf;
use tsify_next::Tsify;
use wasm_bindgen::prelude::*;

use crate::{
    err::{invalid_pda_err, spl_err, SanctumRouterError},
    interface::{keys_signer_writer_to_account_metas, AccountMeta, Instruction, B58PK},
    pda::router::find_fee_token_account_pda_internal,
    router::{token_pair::TokenQuoteWithRouterFee, SanctumRouterHandle},
};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi, large_number_types_as_bigints)]
#[serde(rename_all = "camelCase")]
pub struct WithdrawSolQuoteParams {
    /// Input LST amount
    pub amt: u64,

    /// Input mint
    pub inp: B58PK,
}

/// Requires `update()` to be called before calling this function
#[wasm_bindgen(js_name = quoteWithdrawSol)]
pub fn quote_withdraw_sol(
    this: &SanctumRouterHandle,
    params: WithdrawSolQuoteParams,
) -> Result<TokenQuoteWithRouterFee, SanctumRouterError> {
    let inp_mint = params.inp.0;
    this.0
        .try_find_spl_by_mint(&inp_mint)?
        .withdraw_sol_quoter(this.0.try_curr_epoch()?)?
        .quote_withdraw_sol(params.amt)
        .map(|q| TokenQuoteWithRouterFee(q.withdraw_sol_with_router_fee()))
        .map_err(spl_err)
}

#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi, large_number_types_as_bigints)]
#[serde(rename_all = "camelCase")]
pub struct WithdrawSolSwapParams {
    /// Input LST amount
    pub amt: u64,

    /// Input mint
    pub inp: B58PK,

    /// Input token account to transfer `amt` tokens from
    pub signer_inp: B58PK,

    /// Output token account to receive tokens to
    pub signer_out: B58PK,

    /// Signing authority of `self.signer_inp`; user making the swap.
    pub signer: B58PK,
}

/// Requires `update()` to be called before calling this function
#[wasm_bindgen(js_name = withdrawSolIx)]
pub fn withdraw_sol_ix(
    this: &SanctumRouterHandle,
    params: WithdrawSolSwapParams,
) -> Result<Instruction, SanctumRouterError> {
    let inp_mint = params.inp.0;
    let router = this.0.try_find_spl_by_mint(&inp_mint)?.sol_suf_accs()?;

    let (prefix_metas, data) = withdraw_wrapped_sol_prefix_metas_and_data(&params)?;

    let suffix_accounts = keys_signer_writer_to_account_metas(
        &router.suffix_accounts().as_borrowed().0,
        &router.suffix_is_signer().0,
        &router.suffix_is_writable().0,
    );

    Ok(Instruction {
        program_address: B58PK::new(SANCTUM_ROUTER_PROGRAM),
        accounts: [prefix_metas.as_ref(), suffix_accounts.as_ref()]
            .concat()
            .into(),
        data: ByteBuf::from(data.to_buf()),
    })
}

fn withdraw_wrapped_sol_prefix_metas_and_data(
    swap_params: &WithdrawSolSwapParams,
) -> Result<
    (
        [AccountMeta; WITHDRAW_WRAPPED_SOL_PREFIX_ACCS_LEN],
        WithdrawWrappedSolIxData,
    ),
    SanctumRouterError,
> {
    let metas = keys_signer_writer_to_account_metas(
        &WithdrawWrappedSolPrefixAccsBuilder::start()
            .with_user(&swap_params.signer.0)
            .with_inp_token(&swap_params.signer_inp.0)
            .with_out_wsol(&swap_params.signer_out.0)
            .with_wsol_fee_token(
                &find_fee_token_account_pda_internal(&NATIVE_MINT)
                    .ok_or_else(invalid_pda_err)?
                    .0,
            )
            .with_inp_mint(&swap_params.inp.0)
            .with_wsol_mint(&NATIVE_MINT)
            .with_token_program(&TOKEN_PROGRAM)
            .build()
            .0,
        &WITHDRAW_WRAPPED_SOL_PREFIX_IS_SIGNER.0,
        &WITHDRAW_WRAPPED_SOL_PREFIX_IS_WRITER.0,
    );

    let data = WithdrawWrappedSolIxData::new(swap_params.amt);

    Ok((metas, data))
}

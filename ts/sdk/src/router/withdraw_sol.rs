use sanctum_router_core::{WithdrawSolQuoter, WithdrawSolSufAccs, SANCTUM_ROUTER_PROGRAM};
use wasm_bindgen::prelude::*;

use crate::{
    err::{generic_err, router_missing_err},
    instructions::get_withdraw_wrapped_sol_prefix_metas_and_data,
    interface::{
        keys_signer_writer_to_account_metas, Instruction, TokenQuoteParams,
        TokenQuoteWithRouterFee, TokenSwapParams, B58PK,
    },
    router::SanctumRouterHandle,
};

/// Requires `update()` to be called before calling this function
#[wasm_bindgen(js_name = quoteWithdrawSol)]
pub fn quote_withdraw_sol(
    this: &SanctumRouterHandle,
    params: TokenQuoteParams,
) -> Result<TokenQuoteWithRouterFee, JsError> {
    let inp_mint = params.inp.0;
    this.0
        .spl_routers
        .iter()
        .find(|r| r.stake_pool.pool_mint == inp_mint)
        .ok_or_else(router_missing_err)?
        .withdraw_sol_quoter()
        .quote_withdraw_sol(params.amt)
        .map(|q| TokenQuoteWithRouterFee(q.withdraw_sol_with_router_fee()))
        .map_err(generic_err)
}

/// Requires `update()` to be called before calling this function
#[wasm_bindgen(js_name = withdrawSolIx)]
pub fn withdraw_sol_ix(
    this: &SanctumRouterHandle,
    params: TokenSwapParams,
) -> Result<Instruction, JsError> {
    let router = this
        .0
        .spl_routers
        .iter()
        .find(|r| r.stake_pool.pool_mint == params.inp.0)
        .ok_or_else(router_missing_err)?
        .sol_suf_accs();

    let (prefix_metas, data) = get_withdraw_wrapped_sol_prefix_metas_and_data(params)?;

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
        data: Box::new(data.to_buf()),
    })
}

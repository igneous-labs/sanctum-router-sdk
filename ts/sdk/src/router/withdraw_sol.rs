use sanctum_router_core::{WithdrawSol, SANCTUM_ROUTER_PROGRAM};
use wasm_bindgen::prelude::*;

use crate::{
    err::router_missing_err,
    instructions::get_withdraw_wrapped_sol_prefix_metas_and_data,
    interface::{
        keys_signer_writer_to_account_metas, Instruction, TokenQuoteParams, TokenSwapParams, B58PK,
    },
    router::{token_quote::TokenQuoteWithRouterFee, SanctumRouterHandle},
};

/// Requires `update()` to be called before calling this function
#[wasm_bindgen(js_name = getWithdrawSolQuote)]
pub fn get_withdraw_sol_quote(
    this: &SanctumRouterHandle,
    params: TokenQuoteParams,
) -> Option<TokenQuoteWithRouterFee> {
    this.0
        .spl_routers
        .iter()
        .find(|r| r.stake_pool.pool_mint == params.inp_mint.0)?
        .to_withdraw_sol_router()
        .get_withdraw_sol_quote(params.amt)
        .map(|q| TokenQuoteWithRouterFee(q.withdraw_sol_with_router_fee()))
}

/// Requires `update()` to be called before calling this function
#[wasm_bindgen(js_name = getWithdrawSolIx)]
pub fn get_withdraw_sol_ix(
    this: &SanctumRouterHandle,
    params: TokenSwapParams,
) -> Result<Instruction, JsError> {
    let router = this
        .0
        .spl_routers
        .iter()
        .find(|r| r.stake_pool.pool_mint == params.inp.0)
        .ok_or_else(router_missing_err)?
        .to_withdraw_sol_router();

    let (prefix_metas, data) = get_withdraw_wrapped_sol_prefix_metas_and_data(params)?;

    let suffix_accounts = keys_signer_writer_to_account_metas(
        &WithdrawSol::suffix_accounts(&router).as_borrowed().0,
        &WithdrawSol::suffix_is_signer(&router).0,
        &WithdrawSol::suffix_is_writable(&router).0,
    );

    Ok(Instruction {
        program_address: B58PK::new(SANCTUM_ROUTER_PROGRAM),
        accounts: [prefix_metas.as_ref(), suffix_accounts.as_ref()]
            .concat()
            .into(),
        data: Box::new(data.to_buf()),
    })
}

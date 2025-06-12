use sanctum_router_core::{DepositSol, WithRouterFee, SANCTUM_ROUTER_PROGRAM};
use wasm_bindgen::prelude::*;

use crate::{
    err::router_missing_err,
    instructions::get_deposit_sol_prefix_metas_and_data,
    interface::{
        keys_signer_writer_to_account_metas, AccountMeta, Instruction, TokenQuoteParams,
        TokenSwapParams, B58PK,
    },
    router::{token_quote::TokenQuoteWithRouterFee, SanctumRouterHandle},
};

/// Requires `update()` to be called before calling this function
#[wasm_bindgen(js_name = getDepositSolQuote)]
pub fn get_deposit_sol_quote(
    this: &SanctumRouterHandle,
    params: TokenQuoteParams,
) -> Option<TokenQuoteWithRouterFee> {
    match params.out_mint.0 {
        sanctum_marinade_liquid_staking_core::MSOL_MINT_ADDR => this
            .0
            .marinade_router
            .to_deposit_sol_router()
            .get_deposit_sol_quote(params.amt),
        mint => this
            .0
            .spl_routers
            .iter()
            .find(|r| r.stake_pool.pool_mint == mint)?
            .to_deposit_sol_router()
            .get_deposit_sol_quote(params.amt),
    }
    .map(|q| TokenQuoteWithRouterFee(WithRouterFee::zero(q)))
}

/// Requires `update()` to be called before calling this function
#[wasm_bindgen(js_name = getDepositSolIx)]
pub fn get_deposit_sol_ix(
    this: &SanctumRouterHandle,
    params: TokenSwapParams,
) -> Result<Instruction, JsError> {
    let out_mint = params.out.0;
    let (prefix_metas, data) = get_deposit_sol_prefix_metas_and_data(params)?;

    let metas: Box<[AccountMeta]> = match out_mint {
        sanctum_marinade_liquid_staking_core::MSOL_MINT_ADDR => {
            let router = this.0.marinade_router.to_deposit_sol_router();

            let suffix_accounts = keys_signer_writer_to_account_metas(
                &router.suffix_accounts().as_borrowed().0,
                &router.suffix_is_signer().0,
                &router.suffix_is_writable().0,
            );

            [prefix_metas.as_ref(), suffix_accounts.as_ref()]
                .concat()
                .into()
        }
        mint => {
            let router = this
                .0
                .spl_routers
                .iter()
                .find(|r| r.stake_pool.pool_mint == mint)
                .ok_or_else(router_missing_err)?
                .to_deposit_sol_router();

            let suffix_accounts = keys_signer_writer_to_account_metas(
                &DepositSol::suffix_accounts(&router).as_borrowed().0,
                &DepositSol::suffix_is_signer(&router).0,
                &DepositSol::suffix_is_writable(&router).0,
            );

            [prefix_metas.as_ref(), suffix_accounts.as_ref()]
                .concat()
                .into()
        }
    };

    let ix = Instruction {
        program_address: B58PK::new(SANCTUM_ROUTER_PROGRAM),
        accounts: metas,
        data: Box::new(data.to_buf()),
    };

    Ok(ix)
}

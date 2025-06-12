use sanctum_router_core::{
    DepositSolQuoter, DepositSolSufAccs, WithRouterFee, SANCTUM_ROUTER_PROGRAM,
};
use wasm_bindgen::prelude::*;

use crate::{
    err::{generic_err, router_missing_err},
    instructions::get_deposit_sol_prefix_metas_and_data,
    interface::{
        keys_signer_writer_to_account_metas, AccountMeta, Instruction, TokenQuoteParams,
        TokenSwapParams, B58PK,
    },
    router::{token_quote::TokenQuoteWithRouterFee, SanctumRouterHandle},
};

/// Requires `update()` to be called before calling this function
#[wasm_bindgen(js_name = quoteDepositSol)]
pub fn quote_deposit_sol(
    this: &SanctumRouterHandle,
    params: TokenQuoteParams,
) -> Result<TokenQuoteWithRouterFee, JsError> {
    match params.out_mint.0 {
        sanctum_marinade_liquid_staking_core::MSOL_MINT_ADDR => this
            .0
            .marinade_router
            .quoter()
            .quote_deposit_sol(params.amt)
            .map_err(generic_err),
        mint => this
            .0
            .spl_routers
            .iter()
            .find(|r| r.stake_pool.pool_mint == mint)
            .ok_or_else(router_missing_err)?
            .deposit_sol_quoter()
            .quote_deposit_sol(params.amt)
            .map_err(generic_err),
    }
    .map(|q| TokenQuoteWithRouterFee(WithRouterFee::zero(q)))
}

/// Requires `update()` to be called before calling this function
#[wasm_bindgen(js_name = depositSolIx)]
pub fn deposit_sol_ix(
    this: &SanctumRouterHandle,
    params: TokenSwapParams,
) -> Result<Instruction, JsError> {
    let out_mint = params.out.0;
    let (prefix_metas, data) = get_deposit_sol_prefix_metas_and_data(params)?;

    let metas: Box<[AccountMeta]> = match out_mint {
        sanctum_marinade_liquid_staking_core::MSOL_MINT_ADDR => {
            let router = this.0.marinade_router.deposit_sol_suf_accs();

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
                .sol_suf_accs();

            let suffix_accounts = keys_signer_writer_to_account_metas(
                &router.suffix_accounts().as_borrowed().0,
                &router.suffix_is_signer().0,
                &router.suffix_is_writable().0,
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

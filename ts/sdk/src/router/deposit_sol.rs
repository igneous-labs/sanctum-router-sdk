use sanctum_router_core::{
    DepositSolQuoter, DepositSolSufAccs, WithRouterFee, SANCTUM_ROUTER_PROGRAM,
};
use serde::{Deserialize, Serialize};
use tsify_next::Tsify;
use wasm_bindgen::prelude::*;

use crate::{
    err::{generic_err, router_missing_err},
    instructions::get_deposit_sol_prefix_metas_and_data,
    interface::{
        keys_signer_writer_to_account_metas, AccountMeta, Instruction, TokenQuoteWithRouterFee,
        TokenSwapParams, B58PK,
    },
    router::SanctumRouterHandle,
};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi, large_number_types_as_bigints)]
#[serde(rename_all = "camelCase")]
pub struct DepositSolQuoteParams {
    /// Input lamport amount
    pub amt: u64,

    /// Output mint
    pub out: B58PK,
}

/// Requires `update()` to be called before calling this function
#[wasm_bindgen(js_name = quoteDepositSol)]
pub fn quote_deposit_sol(
    this: &SanctumRouterHandle,
    params: DepositSolQuoteParams,
) -> Result<TokenQuoteWithRouterFee, JsError> {
    let out_mint = params.out.0;
    match out_mint {
        sanctum_marinade_liquid_staking_core::MSOL_MINT_ADDR => this
            .0
            .marinade_router
            .deposit_sol_quoter()
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

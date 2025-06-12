use sanctum_router_core::{
    DepositStakeQuoter, DepositStakeSufAccs, StakeAccountLamports, WithRouterFee,
    SANCTUM_ROUTER_PROGRAM,
};
use serde::{Deserialize, Serialize};
use tsify_next::Tsify;
use wasm_bindgen::prelude::*;

use crate::{
    err::{generic_err, router_missing_err},
    instructions::get_deposit_stake_prefix_metas_and_data,
    interface::{
        keys_signer_writer_to_account_metas, AccountMeta, DepositStakeQuoteParams,
        DepositStakeSwapParams, Instruction, B58PK,
    },
    router::SanctumRouterHandle,
};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi, large_number_types_as_bigints)]
#[serde(rename_all = "camelCase")]
pub struct DepositStakeQuote {
    pub inp_stake: StakeAccountLamports,

    pub validator_vote: B58PK,

    /// Output tokens, after subtracting fees
    pub out: u64,

    /// In terms of output tokens
    pub fee: u64,
}

// need to use a simple newtype here instead of type alias
// otherwise wasm_bindgen shits itself with missing generics
#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi, large_number_types_as_bigints)]
#[serde(rename_all = "camelCase")]
pub struct DepositStakeQuoteWithRouterFee(WithRouterFee<DepositStakeQuote>);

fn conv_quote(
    WithRouterFee {
        quote: sanctum_router_core::DepositStakeQuote { inp, out, fee },
        router_fee,
    }: WithRouterFee<sanctum_router_core::DepositStakeQuote>,
) -> DepositStakeQuoteWithRouterFee {
    DepositStakeQuoteWithRouterFee(WithRouterFee {
        quote: DepositStakeQuote {
            inp_stake: inp.lamports,
            validator_vote: B58PK::new(inp.vote),
            out,
            fee,
        },
        router_fee,
    })
}

/// Requires `update()` to be called before calling this function
#[wasm_bindgen(js_name = quoteDepositStake)]
pub fn quote_deposit_stake(
    this: &SanctumRouterHandle,
    params: DepositStakeQuoteParams,
) -> Result<DepositStakeQuoteWithRouterFee, JsError> {
    let active_stake_params = params.to_active_stake_params();
    match params.out_mint.0 {
        sanctum_router_core::NATIVE_MINT => this
            .0
            .reserve_router
            .deposit_stake_quoter()
            .quote_deposit_stake(active_stake_params)
            .map_err(generic_err),
        sanctum_marinade_liquid_staking_core::MSOL_MINT_ADDR => this
            .0
            .marinade_router
            .deposit_stake_quoter()
            .quote_deposit_stake(active_stake_params)
            .map_err(generic_err),
        mint => {
            let router = this
                .0
                .spl_routers
                .iter()
                .find(|r| r.stake_pool.pool_mint == mint)
                .ok_or_else(router_missing_err)?;
            router
                .deposit_stake_quoter()
                .quote_deposit_stake(active_stake_params)
                .map_err(generic_err)
        }
    }
    .map(|q| {
        conv_quote(if params.out_mint.0 != sanctum_router_core::NATIVE_MINT {
            q.with_router_fee()
        } else {
            WithRouterFee::zero(q)
        })
    })
}

/// Requires `update()` to be called before calling this function
/// Stake account to deposit should be set on `params.signerInp`
/// Vote account of the stake account to deposit should be set on `params.inp`
#[wasm_bindgen(js_name = depositStakeIx)]
pub fn deposit_stake_ix(
    this: &SanctumRouterHandle,
    params: DepositStakeSwapParams,
) -> Result<Instruction, JsError> {
    let out_mint = params.out.0;
    let vote_account = params.inp.0;
    let stake_account = params.signer_inp.0;
    let (prefix_metas, data) = get_deposit_stake_prefix_metas_and_data(params)?;

    let metas: Box<[AccountMeta]> = match out_mint {
        sanctum_router_core::NATIVE_MINT => {
            let router = this
                .0
                .reserve_router
                .deposit_stake_suf_accs(&stake_account)
                .ok_or_else(router_missing_err)?;

            let suffix_accounts = keys_signer_writer_to_account_metas(
                &router.suffix_accounts().as_borrowed().0,
                &router.suffix_is_signer().0,
                &router.suffix_is_writable().0,
            );

            [prefix_metas.as_ref(), suffix_accounts.as_ref()]
                .concat()
                .into()
        }
        sanctum_marinade_liquid_staking_core::MSOL_MINT_ADDR => {
            let router = this
                .0
                .marinade_router
                .deposit_stake_suf_accs(&vote_account)
                .ok_or_else(router_missing_err)?;

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
                .deposit_stake_suf_accs(&vote_account)
                .ok_or_else(router_missing_err)?;

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

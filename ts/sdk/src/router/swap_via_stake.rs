use bs58_fixed_wasm::Bs58Array;
use sanctum_marinade_liquid_staking_core::MSOL_MINT_ADDR;
use sanctum_router_core::{
    quote_prefund_swap_via_stake as core_quote, DepositStakeQuote, Prefund,
    SplWithdrawStakeValQuoter, WithRouterFee, WithdrawStakeQuote, NATIVE_MINT,
};
use serde::{Deserialize, Serialize};
use solido_legacy_core::STSOL_MINT_ADDR;
use tsify_next::Tsify;
use wasm_bindgen::prelude::*;

use crate::{
    err::generic_err,
    router::{token_pair::TokenQuoteParams, SanctumRouter, SanctumRouterHandle},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Tsify)]
#[serde(rename_all = "camelCase")]
#[tsify(into_wasm_abi, from_wasm_abi, large_number_types_as_bigints)]
pub struct SwapViaStakeQuote {
    /// Input tokens that will leave the user's wallet
    pub inp: u64,

    /// The amount of tokens received, after fees
    pub out: u64,

    /// Fee charged on withdraw stake leg, in terms of input tokens
    pub inp_fee: u64,

    /// Fee charged on deposit stake leg, in terms of output tokens
    pub out_fee: u64,
}

// need to use a simple newtype here instead of type alias
// otherwise wasm_bindgen shits itself with missing generics
// TODO: this type name is very long but keeps consistency with naming conventions
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi, large_number_types_as_bigints)]
#[serde(rename_all = "camelCase")]
pub struct PrefundSwapViaStakeQuoteWithRouterFee(
    pub(crate) Prefund<WithRouterFee<SwapViaStakeQuote>>,
);

/// Requires `update()` to be called before calling this function
#[wasm_bindgen(js_name = quotePrefundSwapViaStake)]
pub fn quote_prefund_swap_via_stake(
    this: &SanctumRouterHandle,
    TokenQuoteParams {
        amt,
        inp: Bs58Array(inp_mint),
        out: Bs58Array(out_mint),
    }: TokenQuoteParams,
) -> Result<PrefundSwapViaStakeQuoteWithRouterFee, JsError> {
    quote_prefund_swap_via_stake_inner(&this.0, amt, &inp_mint, &out_mint)
        .map(|(wsq, dsq)| map_quote(&out_mint, wsq, dsq))
}

#[inline] // inlining reduces binary size slightly
fn map_quote(
    out_mint: &[u8; 32],
    Prefund {
        quote: WithdrawStakeQuote {
            inp, fee: inp_fee, ..
        },
        prefund_fee,
    }: Prefund<WithdrawStakeQuote>,
    dsq: DepositStakeQuote,
) -> PrefundSwapViaStakeQuoteWithRouterFee {
    let WithRouterFee {
        quote: DepositStakeQuote {
            out, fee: out_fee, ..
        },
        router_fee,
    } = if *out_mint != sanctum_router_core::NATIVE_MINT {
        dsq.with_router_fee()
    } else {
        WithRouterFee::zero(dsq)
    };
    PrefundSwapViaStakeQuoteWithRouterFee(Prefund {
        quote: WithRouterFee {
            quote: SwapViaStakeQuote {
                inp,
                out,
                inp_fee,
                out_fee,
            },
            router_fee,
        },
        prefund_fee,
    })
}

// Used by both quote and ix
fn quote_prefund_swap_via_stake_inner(
    this: &SanctumRouter,
    amt: u64,
    inp_mint: &[u8; 32],
    out_mint: &[u8; 32],
) -> Result<(Prefund<WithdrawStakeQuote>, DepositStakeQuote), JsError> {
    let (reserves_balance, reserves_fee) = this.reserve_router.prefund_params();

    // TODO: if we used dyn or some other means we could reduce
    // number of total match arms (Withdraw + Deposit) from n^2 to n
    // but for now we have this macro in its place to reduce redundancy instead
    macro_rules! match_deposit_stake {
        ($w_itr:expr) => {
            match *out_mint {
                NATIVE_MINT => {
                    let d = this.reserve_router.deposit_stake_quoter().after_prefund()?;
                    core_quote($w_itr, d, amt, &reserves_balance, reserves_fee).map_err(generic_err)
                }
                MSOL_MINT_ADDR => {
                    let d = this.marinade_router.deposit_stake_quoter();
                    core_quote($w_itr, d, amt, &reserves_balance, reserves_fee).map_err(generic_err)
                }
                out => {
                    let d = this.try_find_spl_by_mint(&out)?.deposit_stake_quoter();
                    core_quote($w_itr, d, amt, &reserves_balance, reserves_fee).map_err(generic_err)
                }
            }
        };
    }

    match *inp_mint {
        STSOL_MINT_ADDR => {
            let w_itr = this.lido_router.withdraw_stake_quoter();
            match_deposit_stake!(w_itr)
        }
        inp => {
            let router = this.try_find_spl_by_mint(&inp)?;
            let w_itr = SplWithdrawStakeValQuoter::all(
                &router.stake_pool,
                &router.validator_list.validators,
                router.curr_epoch,
            )?;
            match_deposit_stake!(w_itr)
        }
    }
}

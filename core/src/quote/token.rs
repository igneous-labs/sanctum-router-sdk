use crate::{WithRouterFee, WITHDRAW_WRAPPED_SOL_GLOBAL_FEE_BPS};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "camelCase")
)]
#[cfg_attr(
    feature = "wasm",
    derive(tsify_next::Tsify),
    tsify(into_wasm_abi, from_wasm_abi, large_number_types_as_bigints)
)]
pub struct TokenQuote {
    /// Input tokens that will leave the user's wallet
    pub inp: u64,

    /// The amount of tokens received, after fees
    pub out: u64,

    /// In terms of output tokens
    pub fee: u64,
}

impl TokenQuote {
    /// Applies for WithdrawSol
    pub fn withdraw_sol_with_router_fee(self) -> WithRouterFee<Self> {
        let Self { inp, out, fee } = self;

        if out == 0 {
            return WithRouterFee::zero(self);
        }

        // cast-safety: GLOBAL_FEE_BPS is < 10_000, no overflow should happen
        let router_fee = ((self.out as u128) * (WITHDRAW_WRAPPED_SOL_GLOBAL_FEE_BPS as u128)
            / 10_000u128) as u64;
        let router_fee = router_fee.max(1);

        WithRouterFee {
            quote: Self {
                inp,
                out: out - router_fee,
                fee,
            },
            router_fee,
        }
    }
}

impl From<sanctum_spl_stake_pool_core::DepositSolQuote> for TokenQuote {
    fn from(quote: sanctum_spl_stake_pool_core::DepositSolQuote) -> Self {
        Self {
            inp: quote.in_amount,
            out: quote.out_amount,
            fee: quote.referral_fee + quote.manager_fee,
        }
    }
}

impl From<sanctum_marinade_liquid_staking_core::DepositSolQuote> for TokenQuote {
    fn from(quote: sanctum_marinade_liquid_staking_core::DepositSolQuote) -> Self {
        Self {
            inp: quote.in_amount,
            out: quote.out_amount,
            fee: 0,
        }
    }
}

impl From<sanctum_spl_stake_pool_core::WithdrawSolQuote> for TokenQuote {
    fn from(quote: sanctum_spl_stake_pool_core::WithdrawSolQuote) -> Self {
        Self {
            inp: quote.in_amount,
            out: quote.out_amount,
            fee: quote.manager_fee,
        }
    }
}

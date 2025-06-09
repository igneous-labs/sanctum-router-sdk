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
    pub in_amount: u64,

    /// The amount of tokens received, after fees
    pub out_amount: u64,

    pub fee_amount: u64,
}

impl TokenQuote {
    /// Applies for WithdrawSol
    pub fn withdraw_sol_with_router_fee(self) -> WithRouterFee<Self> {
        let Self {
            in_amount,
            out_amount,
            fee_amount,
        } = self;

        if out_amount == 0 {
            return WithRouterFee::zero(self);
        }

        // cast-safety: GLOBAL_FEE_BPS is < 10_000, no overflow should happen
        let router_fee = ((self.out_amount as u128) * (WITHDRAW_WRAPPED_SOL_GLOBAL_FEE_BPS as u128)
            / 10_000u128) as u64;
        let router_fee = router_fee.max(1);

        WithRouterFee {
            quote: Self {
                in_amount,
                out_amount: out_amount - router_fee,
                fee_amount,
            },
            router_fee,
        }
    }
}

impl From<sanctum_spl_stake_pool_core::DepositSolQuote> for TokenQuote {
    fn from(quote: sanctum_spl_stake_pool_core::DepositSolQuote) -> Self {
        Self {
            in_amount: quote.in_amount,
            out_amount: quote.out_amount,
            fee_amount: quote.referral_fee + quote.manager_fee,
        }
    }
}

impl From<sanctum_marinade_liquid_staking_core::DepositSolQuote> for TokenQuote {
    fn from(quote: sanctum_marinade_liquid_staking_core::DepositSolQuote) -> Self {
        Self {
            in_amount: quote.in_amount,
            out_amount: quote.out_amount,
            fee_amount: 0,
        }
    }
}

impl From<sanctum_spl_stake_pool_core::WithdrawSolQuote> for TokenQuote {
    fn from(quote: sanctum_spl_stake_pool_core::WithdrawSolQuote) -> Self {
        Self {
            in_amount: quote.in_amount,
            out_amount: quote.out_amount,
            fee_amount: quote.manager_fee,
        }
    }
}

use crate::{WithRouterFee, DEPOSIT_STAKE_GLOBAL_FEE_BPS};

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
pub struct DepositStakeQuote {
    /// Output tokens, after subtracting fees
    pub tokens_out: u64,

    /// In terms of output tokens - contains global fee if `with_global_fee` is called
    pub fee_amount: u64,
}

impl DepositStakeQuote {
    /// Applies for DepositStake
    pub fn with_router_fee(self) -> WithRouterFee<Self> {
        let Self {
            tokens_out,
            fee_amount,
        } = self;

        if tokens_out == 0 {
            return WithRouterFee::zero(self);
        }

        // cast-safety: GLOBAL_FEE_BPS is < 10_000, no overflow should happen
        let router_fee =
            ((tokens_out as u128) * (DEPOSIT_STAKE_GLOBAL_FEE_BPS as u128) / 10_000u128) as u64;
        let router_fee = router_fee.max(1);

        WithRouterFee {
            quote: Self {
                tokens_out: tokens_out - router_fee,
                fee_amount,
            },
            router_fee,
        }
    }
}

impl From<sanctum_spl_stake_pool_core::DepositStakeQuote> for DepositStakeQuote {
    fn from(quote: sanctum_spl_stake_pool_core::DepositStakeQuote) -> Self {
        Self {
            tokens_out: quote.tokens_out + quote.referral_fee,
            fee_amount: quote.manager_fee,
        }
    }
}

impl From<sanctum_marinade_liquid_staking_core::DepositStakeQuote> for DepositStakeQuote {
    fn from(quote: sanctum_marinade_liquid_staking_core::DepositStakeQuote) -> Self {
        Self {
            tokens_out: quote.tokens_out,
            fee_amount: 0,
        }
    }
}

impl From<sanctum_reserve_core::UnstakeQuote> for DepositStakeQuote {
    fn from(quote: sanctum_reserve_core::UnstakeQuote) -> Self {
        Self {
            tokens_out: quote.lamports_to_unstaker,
            fee_amount: quote.fee,
        }
    }
}

mod deposit_sol;
mod deposit_stake;
mod withdraw_sol;

use borsh::{BorshDeserialize, BorshSerialize};
pub use deposit_sol::*;
pub use deposit_stake::*;
pub use withdraw_sol::*;

use crate::{DEPOSIT_STAKE_GLOBAL_FEE_BPS, WITHDRAW_WRAPPED_SOL_GLOBAL_FEE_BPS};
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, BorshDeserialize, BorshSerialize)]
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
    pub fn with_global_fee(self) -> Self {
        if self.out_amount == 0 {
            return self;
        }

        // cast-safety: GLOBAL_FEE_BPS is < 10_000, no overflow should happen
        let fee = ((self.out_amount as u128) * (WITHDRAW_WRAPPED_SOL_GLOBAL_FEE_BPS as u128)
            / 10_000u128) as u64;
        let fee = fee.max(1);

        Self {
            in_amount: self.in_amount,
            out_amount: self.out_amount - fee,
            fee_amount: self.fee_amount + fee,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, BorshDeserialize, BorshSerialize)]
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
    pub fn with_global_fee(self) -> Self {
        if self.tokens_out == 0 {
            return self;
        }

        // cast-safety: GLOBAL_FEE_BPS is < 10_000, no overflow should happen
        let fee = ((self.tokens_out as u128) * (DEPOSIT_STAKE_GLOBAL_FEE_BPS as u128) / 10_000u128)
            as u64;
        let fee = fee.max(1);

        Self {
            tokens_out: self.tokens_out - fee,
            fee_amount: self.fee_amount + fee,
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

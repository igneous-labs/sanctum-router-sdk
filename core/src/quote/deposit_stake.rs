use crate::{StakeAccountLamports, WithRouterFee, DEPOSIT_STAKE_GLOBAL_FEE_BPS};

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
    /// The stake account to be deposited
    pub inp: StakeAccountLamports,

    /// Output tokens, after subtracting fees
    pub out: u64,

    /// In terms of output tokens
    pub fee: u64,
}

impl DepositStakeQuote {
    /// Applies for DepositStake
    pub fn with_router_fee(self) -> WithRouterFee<Self> {
        let Self { inp, out, fee } = self;

        if out == 0 {
            return WithRouterFee::zero(self);
        }

        // cast-safety: GLOBAL_FEE_BPS is < 10_000, no overflow should happen
        let router_fee =
            ((out as u128) * (DEPOSIT_STAKE_GLOBAL_FEE_BPS as u128) / 10_000u128) as u64;
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

impl From<sanctum_spl_stake_pool_core::DepositStakeQuote> for DepositStakeQuote {
    fn from(
        sanctum_spl_stake_pool_core::DepositStakeQuote {
            stake_account_lamports_in:
                sanctum_spl_stake_pool_core::StakeAccountLamports { staked, unstaked },
            tokens_out,
            manager_fee,
            referral_fee,
        }: sanctum_spl_stake_pool_core::DepositStakeQuote,
    ) -> Self {
        Self {
            inp: StakeAccountLamports { staked, unstaked },
            // we set referral destination = out token acc, so the user gets the referral fee
            out: tokens_out + referral_fee,
            fee: manager_fee,
        }
    }
}

impl From<sanctum_marinade_liquid_staking_core::DepositStakeQuote> for DepositStakeQuote {
    fn from(
        sanctum_marinade_liquid_staking_core::DepositStakeQuote {
            stake_account_lamports_in:
                sanctum_marinade_liquid_staking_core::StakeAccountLamports { staked, unstaked },
            tokens_out,
        }: sanctum_marinade_liquid_staking_core::DepositStakeQuote,
    ) -> Self {
        Self {
            inp: StakeAccountLamports { staked, unstaked },
            out: tokens_out,
            fee: 0,
        }
    }
}

pub fn conv_unstake_quote(
    sanctum_reserve_core::UnstakeQuote {
        lamports_to_unstaker,
        fee,
        ..
    }: sanctum_reserve_core::UnstakeQuote,
    inp: StakeAccountLamports,
) -> DepositStakeQuote {
    DepositStakeQuote {
        inp,
        out: lamports_to_unstaker,
        fee,
    }
}

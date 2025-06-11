use sanctum_spl_stake_pool_core::STAKE_ACCOUNT_RENT_EXEMPT_LAMPORTS;

use crate::StakeAccountLamports;

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
pub struct WithdrawStakeQuote {
    /// Input pool tokens
    pub inp: u64,

    /// The stake account that will be withdrawn
    pub out: StakeAccountLamports,

    /// In terms of input tokens, charged by the stake pool
    pub fee: u64,
}

impl WithdrawStakeQuote {
    /// Create a quote for a withdrawn stake account that has rent-exemption prefunded
    #[inline]
    pub const fn prefund(inp: u64, staked_lamports: u64, fee: u64) -> Self {
        Self {
            inp,
            out: StakeAccountLamports {
                staked: staked_lamports,
                unstaked: STAKE_ACCOUNT_RENT_EXEMPT_LAMPORTS,
            },
            fee,
        }
    }
}

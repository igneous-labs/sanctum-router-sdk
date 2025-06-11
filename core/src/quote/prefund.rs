use sanctum_reserve_core::{FeeEnum, PoolBalance};

use crate::{PREFUND_FLASH_LOAN_LAMPORTS, ZERO_DATA_ACC_RENT_EXEMPT_LAMPORTS};

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
pub struct Prefund<Q> {
    pub quote: Q,

    /// In terms of SOL. Part of the withdrawn stake account
    /// that's instant unstaked to repay the prefund flash loan
    pub prefund_fee: u64,
}

/// Computes the total lamports (including rent) that the slumdog stake account
/// should consist of when it gets instant unstaked in order to repay the prefund flash loan
#[inline]
pub fn slumdog_target_lamports(
    reserves_balance: &PoolBalance,
    reserves_fee: &FeeEnum,
) -> Option<u64> {
    reserves_fee.reverse_from_rem(reserves_balance, PREFUND_FLASH_LOAN_LAMPORTS)
}

#[inline]
pub const fn reserves_has_enough_for_slumdog(reserves_balance: &PoolBalance) -> bool {
    reserves_balance.sol_reserves_lamports
        >= PREFUND_FLASH_LOAN_LAMPORTS + ZERO_DATA_ACC_RENT_EXEMPT_LAMPORTS
}

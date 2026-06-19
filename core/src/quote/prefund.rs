use sanctum_reserve_core::{FeeEnum, PoolUnstakeParams, ProtocolFeeRatios, Rational, ReserveError};
use sanctum_spl_stake_pool_core::STAKE_ACCOUNT_RENT_EXEMPT_LAMPORTS;

use crate::{
    ActiveStakeParams, DepositStakeQuoter, ReserveDepositStakeQuoter, StakeAccountLamports,
    PREFUND_FLASH_LOAN_LAMPORTS, STAKE_PROG_MIN_DELEGATION_LAMPORTS,
};

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

/// total lamports (including rent) that the slumdog stake account
/// should consist of when it gets instant unstaked in order to repay the prefund flash loan
pub const SLUMDOG_TARGET_LAMPORTS: u64 = {
    // because of this >, slumdog target is min this value instead
    assert!(STAKE_PROG_MIN_DELEGATION_LAMPORTS > PREFUND_FLASH_LOAN_LAMPORTS);

    STAKE_ACCOUNT_RENT_EXEMPT_LAMPORTS + STAKE_PROG_MIN_DELEGATION_LAMPORTS
};

#[inline]
pub fn reserves_has_enough_for_slumdog(
    reserves_unstake_params: &PoolUnstakeParams,
    reserves_fee: &FeeEnum,
) -> Result<(), ReserveError> {
    (ReserveDepositStakeQuoter {
        fee_account: reserves_fee,
        pool_incoming_stake: reserves_unstake_params.pool_incoming_stake,
        pool_sol_reserves: reserves_unstake_params.sol_reserves_lamports,
        // dontcare, protocol fee values do not affect
        // outflow amount, only who the outflow goes to
        protocol_fee: &ProtocolFeeRatios {
            // NB: Rational::ZERO results in InternalError due to change in sanctum-u64-ratio behaviour
            fee_ratio: Rational { num: 0, denom: 1 },
            referrer_fee_ratio: Rational { num: 0, denom: 1 },
        },
    })
    .quote_deposit_stake(ActiveStakeParams {
        lamports: StakeAccountLamports {
            staked: SLUMDOG_TARGET_LAMPORTS - STAKE_ACCOUNT_RENT_EXEMPT_LAMPORTS,
            unstaked: STAKE_ACCOUNT_RENT_EXEMPT_LAMPORTS,
        },
        vote: Default::default(), // dontcare
    })
    .map(|_| ())
}

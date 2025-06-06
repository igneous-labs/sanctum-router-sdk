use sanctum_marinade_liquid_staking_core::{duplication_flag_seeds, MARINADE_STAKING_PROGRAM};
use sanctum_reserve_core::stake_account_record_seeds;
use sanctum_router_core::{fee_token_acc_seeds, SANCTUM_ROUTER_PROGRAM};
use spl_stake_pool::find_pda;

pub fn find_fee_token_account_pda_internal(mint: &[u8; 32]) -> Option<([u8; 32], u8)> {
    let (s1, s2) = fee_token_acc_seeds(mint);
    find_pda(&[s1.as_slice(), s2.as_slice()], &SANCTUM_ROUTER_PROGRAM)
}

/// Marinade Duplication Flag
pub fn find_marinade_duplication_flag_pda_internal(
    vote_account: &[u8; 32],
) -> Option<([u8; 32], u8)> {
    let (s1, s2, s3) = duplication_flag_seeds(
        &sanctum_marinade_liquid_staking_core::STATE_PUBKEY,
        vote_account,
    );
    find_pda(
        &[s1.as_slice(), s2.as_slice(), s3.as_slice()],
        &MARINADE_STAKING_PROGRAM,
    )
}

/// Reserve Stake Account Record
pub fn find_reserve_stake_account_record_pda_internal(
    stake_account_addr: &[u8; 32],
) -> Option<([u8; 32], u8)> {
    let (s1, s2) = stake_account_record_seeds(&sanctum_reserve_core::POOL, stake_account_addr);
    find_pda(
        &[s1.as_slice(), s2.as_slice()],
        &sanctum_reserve_core::UNSTAKE_PROGRAM,
    )
}

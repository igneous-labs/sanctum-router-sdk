use sanctum_reserve_core::stake_account_record_seeds;

use crate::pda::find_pda;

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

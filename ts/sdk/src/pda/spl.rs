use sanctum_router_std::sanctum_spl_stake_pool_core::{deposit_auth_seeds, withdraw_auth_seeds};

use crate::pda::find_pda;

pub fn find_withdraw_auth_pda_internal(
    program_id: &[u8; 32],
    stake_pool_addr: &[u8; 32],
) -> Option<([u8; 32], u8)> {
    let (s1, s2) = withdraw_auth_seeds(stake_pool_addr);
    find_pda(&[s1.as_slice(), s2.as_slice()], program_id)
}

pub fn find_deposit_auth_pda_internal(
    program_id: &[u8; 32],
    stake_pool_addr: &[u8; 32],
) -> Option<([u8; 32], u8)> {
    let (s1, s2) = deposit_auth_seeds(stake_pool_addr);
    find_pda(&[s1.as_slice(), s2.as_slice()], program_id)
}

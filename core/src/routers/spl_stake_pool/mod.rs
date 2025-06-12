use sanctum_spl_stake_pool_core::StakePool;

mod deposit_sol;
mod deposit_stake;
mod withdraw_sol;

pub use deposit_sol::*;
pub use deposit_stake::*;
pub use withdraw_sol::*;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SplSolSufAccs<'a> {
    pub stake_pool: &'a StakePool,
    pub stake_pool_program: &'a [u8; 32],
    pub stake_pool_addr: &'a [u8; 32],
    pub withdraw_authority_program_address: &'a [u8; 32],
}

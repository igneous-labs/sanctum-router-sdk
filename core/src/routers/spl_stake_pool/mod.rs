use sanctum_spl_stake_pool_core::{StakePool, ValidatorStakeInfo};

mod deposit_sol;
mod deposit_stake;
mod withdraw_sol;

pub use deposit_sol::*;
pub use deposit_stake::*;
pub use withdraw_sol::*;

#[derive(Debug, Clone)]
pub struct SplStakePoolWithdrawSolRouter<'a> {
    pub stake_pool_addr: &'a [u8; 32],
    pub stake_pool_program: &'a [u8; 32],
    pub stake_pool: &'a StakePool,
    pub curr_epoch: u64,
    pub withdraw_authority_program_address: &'a [u8; 32],
    pub reserve_stake_lamports: u64,
}

#[derive(Debug, Clone)]
pub struct SplStakePoolDepositStakeRouter<'a> {
    pub stake_pool_addr: &'a [u8; 32],
    pub stake_pool_program: &'a [u8; 32],
    pub stake_pool: &'a StakePool,
    pub current_epoch: u64,
    /// For Stake Pool's DepositStake Ix (Suffix)
    pub deposit_authority_program_address: &'a [u8; 32],
    pub withdraw_authority_program_address: &'a [u8; 32],
    pub validator_stake: [u8; 32],
    /// For Quoting
    pub validator_stake_info: &'a ValidatorStakeInfo,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SplSolSufAccs<'a> {
    pub stake_pool: &'a StakePool,
    pub stake_pool_program: &'a [u8; 32],
    pub stake_pool_addr: &'a [u8; 32],
    pub withdraw_authority_program_address: &'a [u8; 32],
}

use sanctum_spl_stake_pool_core::StakePool;
use solana_sdk::pubkey::Pubkey;
use std::sync::{atomic::AtomicU64, Arc};

mod amm;
mod base;

#[derive(Debug, Default, Clone)]
pub struct SplStakePoolSolAmm {
    pub stake_pool_addr: Pubkey,
    pub stake_pool_program: Pubkey,
    pub stake_pool: StakePool,
    pub curr_epoch: Arc<AtomicU64>,
    pub withdraw_authority_program_address: Pubkey,
}

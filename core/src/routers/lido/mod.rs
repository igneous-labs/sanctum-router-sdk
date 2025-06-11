mod withdraw_stake;

pub use withdraw_stake::*;

#[derive(Debug, Clone)]
pub struct LidoWithdrawStakeRouter<'a> {
    pub state: &'a solido_legacy_core::Lido,
    pub voter: &'a [u8; 32],
    pub stake_to_split: [u8; 32],
    pub curr_epoch: u64,
    pub validator_effective_stake_balance: u64,
}

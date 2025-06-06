use sanctum_router_core::{DepositSol, SplStakePoolDepositSolRouter, SANCTUM_ROUTER_PROGRAM};
use sanctum_spl_stake_pool_core::{withdraw_auth_seeds, StakePool};
use solana_sdk::pubkey::Pubkey;

use crate::base::StakePoolBase;

use super::SplStakePoolSolAmm;

impl StakePoolBase for SplStakePoolSolAmm {
    /// Keyed account: StakePool
    fn from_keyed_account(
        keyed_account: &jupiter_amm_interface::KeyedAccount,
        amm_context: &jupiter_amm_interface::AmmContext,
    ) -> anyhow::Result<Self>
    where
        Self: Sized,
    {
        let stake_pool_bytes = keyed_account.key.to_bytes();
        let withdraw_auth_seeds = withdraw_auth_seeds(&stake_pool_bytes);
        let stake_pool = StakePool::borsh_de(keyed_account.account.data.as_slice())?;

        if stake_pool.sol_deposit_authority.is_some() {
            return Err(anyhow::anyhow!("Deposit authority not supported"));
        }

        Ok(Self {
            stake_pool_addr: keyed_account.key,
            stake_pool_program: keyed_account.account.owner,
            curr_epoch: amm_context.clock_ref.epoch.clone(),
            withdraw_authority_program_address: Pubkey::find_program_address(
                &[
                    withdraw_auth_seeds.0.as_ref(),
                    withdraw_auth_seeds.1.as_ref(),
                ],
                &keyed_account.account.owner,
            )
            .0,
            stake_pool,
        })
    }

    fn program_id(&self) -> Pubkey {
        self.stake_pool_program
    }

    fn main_state_key(&self) -> Pubkey {
        Pubkey::find_program_address(
            &[self.stake_pool_addr.as_ref()],
            &Pubkey::from(SANCTUM_ROUTER_PROGRAM),
        )
        .0
    }

    fn staked_sol_mint(&self) -> Pubkey {
        Pubkey::from(self.stake_pool.pool_mint)
    }

    fn get_accounts_to_update(&self) -> Vec<Pubkey> {
        vec![self.stake_pool_addr]
    }

    fn update(&mut self, account_map: &jupiter_amm_interface::AccountMap) -> anyhow::Result<()> {
        let stake_pool = StakePool::borsh_de(
            account_map
                .get(&self.stake_pool_addr)
                .ok_or_else(|| anyhow::anyhow!("{} not in AccountMap", self.stake_pool_addr))?
                .data
                .as_slice(),
        )?;
        self.stake_pool = stake_pool;

        Ok(())
    }

    fn to_router(&self) -> impl DepositSol {
        SplStakePoolDepositSolRouter {
            stake_pool_addr: self.stake_pool_addr.as_array(),
            stake_pool_program: self.stake_pool_program.as_array(),
            stake_pool: &self.stake_pool,
            curr_epoch: self.curr_epoch.load(std::sync::atomic::Ordering::Relaxed),
            withdraw_authority_program_address: self.withdraw_authority_program_address.as_array(),
        }
    }
}

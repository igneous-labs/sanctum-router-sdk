use sanctum_router_core::{
    SplDepositSolQuoter, SplDepositStakeQuoter, SplDepositStakeSufAccs, SplSolSufAccs,
    SplWithdrawSolQuoter, SplWithdrawStakeQuoter, SplWithdrawStakeSufAccs,
};
use sanctum_spl_stake_pool_core::{
    StakePool, ValidatorList, ValidatorListHeader, ValidatorStakeInfo, SYSVAR_CLOCK,
};
use wasm_bindgen::JsError;

use crate::{
    interface::{get_account, get_account_data, AccountMap},
    pda::spl::find_validator_stake_account_pda_internal,
    update::{PoolUpdateType, Update},
};

#[derive(Clone, Debug, Default, PartialEq)]
pub struct SplStakePoolRouterOwned {
    pub stake_pool_addr: [u8; 32],
    pub stake_pool_program: [u8; 32],
    pub stake_pool: StakePool,
    pub validator_list: ValidatorListOwned,
    pub deposit_authority_program_address: [u8; 32],
    pub withdraw_authority_program_address: [u8; 32],
    pub reserve_stake_lamports: u64,
}

/// DepositSol + WithdrawSol common
impl SplStakePoolRouterOwned {
    pub fn sol_suf_accs(&self) -> SplSolSufAccs {
        SplSolSufAccs {
            stake_pool: &self.stake_pool,
            stake_pool_program: &self.stake_pool_program,
            stake_pool_addr: &self.stake_pool_addr,
            withdraw_authority_program_address: &self.withdraw_authority_program_address,
        }
    }
}

/// DepositSol
impl SplStakePoolRouterOwned {
    pub fn deposit_sol_quoter(&self, curr_epoch: u64) -> SplDepositSolQuoter {
        SplDepositSolQuoter {
            stake_pool: &self.stake_pool,
            curr_epoch,
        }
    }
}

/// WithdrawSol
impl SplStakePoolRouterOwned {
    pub fn withdraw_sol_quoter(&self, curr_epoch: u64) -> SplWithdrawSolQuoter {
        SplWithdrawSolQuoter {
            stake_pool: &self.stake_pool,
            reserve_stake_lamports: self.reserve_stake_lamports,
            curr_epoch,
        }
    }
}

/// DepositStake
impl SplStakePoolRouterOwned {
    pub fn deposit_stake_quoter(&self, curr_epoch: u64) -> SplDepositStakeQuoter {
        SplDepositStakeQuoter {
            stake_pool: &self.stake_pool,
            curr_epoch,
            validator_list: &self.validator_list.validators,
            default_stake_deposit_authority: &self.deposit_authority_program_address,
        }
    }

    pub fn deposit_stake_suf_accs(
        &self,
        vote_account: &[u8; 32],
    ) -> Option<SplDepositStakeSufAccs> {
        let validator_stake_info = self
            .validator_list
            .validators
            .iter()
            .find(|v| v.vote_account_address() == vote_account)?;
        Some(SplDepositStakeSufAccs {
            stake_pool_addr: &self.stake_pool_addr,
            stake_pool_program: &self.stake_pool_program,
            stake_pool: &self.stake_pool,
            validator_stake: find_validator_stake_account_pda_internal(
                &self.stake_pool_program,
                validator_stake_info.vote_account_address(),
                &self.stake_pool_addr,
                validator_stake_info.validator_seed_suffix(),
            )?
            .0,
            stake_deposit_authority: &self.deposit_authority_program_address,
            stake_withdraw_authority: &self.withdraw_authority_program_address,
        })
    }
}

/// WithdrawStake
impl SplStakePoolRouterOwned {
    pub fn withdraw_stake_quoter(&self, curr_epoch: u64) -> SplWithdrawStakeQuoter {
        SplWithdrawStakeQuoter {
            stake_pool: &self.stake_pool,
            curr_epoch,
            validator_list: &self.validator_list.validators,
        }
    }

    /// Returns `None` if vote acc not on validator list or validator stake acc PDA invalid
    pub fn withdraw_stake_suf_accs(
        &self,
        vote_account: &[u8; 32],
    ) -> Option<SplWithdrawStakeSufAccs> {
        let validator_stake_info = self
            .validator_list
            .validators
            .iter()
            .find(|v| v.vote_account_address() == vote_account)?;
        Some(SplWithdrawStakeSufAccs {
            stake_pool_addr: &self.stake_pool_addr,
            stake_pool_program: &self.stake_pool_program,
            stake_pool: &self.stake_pool,
            validator_stake: find_validator_stake_account_pda_internal(
                &self.stake_pool_program,
                validator_stake_info.vote_account_address(),
                &self.stake_pool_addr,
                validator_stake_info.validator_seed_suffix(),
            )?
            .0,
            stake_withdraw_authority: &self.withdraw_authority_program_address,
        })
    }
}

/// Update helpers
impl SplStakePoolRouterOwned {
    pub fn update_stake_pool(&mut self, stake_pool_data: &[u8]) -> Result<(), JsError> {
        self.stake_pool = StakePool::borsh_de(stake_pool_data)?;
        Ok(())
    }

    pub fn update_validator_list(&mut self, validator_list_data: &[u8]) -> Result<(), JsError> {
        let validator_list = ValidatorList::deserialize(validator_list_data)?;
        self.validator_list = ValidatorListOwned {
            header: validator_list.header,
            validators: validator_list.validators.to_vec(),
        };
        Ok(())
    }
}

impl Update for SplStakePoolRouterOwned {
    fn accounts_to_update(&self, ty: PoolUpdateType) -> impl Iterator<Item = [u8; 32]> {
        match ty {
            PoolUpdateType::DepositSol => {
                [Some(SYSVAR_CLOCK), Some(self.stake_pool_addr), None, None]
            }
            PoolUpdateType::WithdrawSol => [
                Some(SYSVAR_CLOCK),
                Some(self.stake_pool_addr),
                Some(self.stake_pool.reserve_stake),
                None,
            ],
            PoolUpdateType::DepositStake | PoolUpdateType::WithdrawStake => [
                Some(SYSVAR_CLOCK),
                Some(self.stake_pool_addr),
                Some(self.stake_pool.validator_list),
                None,
            ],
        }
        .into_iter()
        .flatten()
    }

    fn update(&mut self, ty: PoolUpdateType, accounts: &AccountMap) -> Result<(), JsError> {
        let stake_pool_data = get_account_data(accounts, self.stake_pool_addr)?;
        self.update_stake_pool(stake_pool_data)?;

        match ty {
            PoolUpdateType::DepositSol => (),
            PoolUpdateType::WithdrawSol => {
                self.reserve_stake_lamports =
                    get_account(accounts, self.stake_pool.reserve_stake)?.lamports;
            }
            PoolUpdateType::DepositStake | PoolUpdateType::WithdrawStake => {
                let validator_list_data =
                    get_account_data(accounts, self.stake_pool.validator_list)?;
                self.update_validator_list(validator_list_data)?;
            }
        }

        Ok(())
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct ValidatorListOwned {
    pub header: ValidatorListHeader,
    pub validators: Vec<ValidatorStakeInfo>,
}

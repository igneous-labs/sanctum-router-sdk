use sanctum_router_core::{
    SplDepositSolQuoter, SplSolSufAccs, SplStakePoolDepositStakeRouter, SplWithdrawSolQuoter,
};
use sanctum_spl_stake_pool_core::{
    StakePool, ValidatorList, ValidatorListHeader, ValidatorStakeInfo,
};
use wasm_bindgen::JsError;

use crate::{
    interface::{get_account, get_account_data, AccountMap},
    pda::spl::find_validator_stake_account_pda_internal,
    update::Update,
};

#[derive(Clone, Debug, Default, PartialEq)]
pub struct SplStakePoolRouterOwned {
    pub stake_pool_addr: [u8; 32],
    pub stake_pool_program: [u8; 32],
    pub stake_pool: StakePool,
    pub validator_list: ValidatorListOwned,
    pub curr_epoch: u64,
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
    pub fn deposit_sol_quoter(&self) -> SplDepositSolQuoter {
        SplDepositSolQuoter {
            stake_pool: &self.stake_pool,
            curr_epoch: self.curr_epoch,
        }
    }
}

/// WithdrawSol
impl SplStakePoolRouterOwned {
    pub fn withdraw_sol_quoter(&self) -> SplWithdrawSolQuoter {
        SplWithdrawSolQuoter {
            stake_pool: &self.stake_pool,
            reserve_stake_lamports: self.reserve_stake_lamports,
            curr_epoch: self.curr_epoch,
        }
    }
}

/// DepositStake
impl SplStakePoolRouterOwned {
    /// Sets validator stake according to validator stake info on this struct
    pub fn to_deposit_stake_router(
        &self,
        vote_account: &[u8; 32],
    ) -> Option<SplStakePoolDepositStakeRouter> {
        let validator_stake_info = self
            .validator_list
            .validators
            .iter()
            .find(|v| v.vote_account_address() == vote_account)?;

        Some(SplStakePoolDepositStakeRouter {
            stake_pool_addr: &self.stake_pool_addr,
            stake_pool_program: &self.stake_pool_program,
            stake_pool: &self.stake_pool,
            current_epoch: self.curr_epoch,
            withdraw_authority_program_address: &self.withdraw_authority_program_address,
            deposit_authority_program_address: &self.deposit_authority_program_address,
            validator_stake: find_validator_stake_account_pda_internal(
                &self.stake_pool_program,
                validator_stake_info.vote_account_address(),
                &self.stake_pool_addr,
                validator_stake_info.validator_seed_suffix(),
            )?
            .0,
            validator_stake_info,
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
    fn get_accounts_to_update(&self) -> impl Iterator<Item = [u8; 32]> {
        [
            self.stake_pool_addr,
            self.stake_pool.validator_list,
            self.stake_pool.reserve_stake,
        ]
        .into_iter()
    }

    fn update(&mut self, accounts: &AccountMap) -> Result<(), JsError> {
        let keys = [self.stake_pool_addr, self.stake_pool.validator_list];
        let [Ok(stake_pool_data), Ok(validator_list_data)] =
            keys.map(|pk| get_account_data(accounts, pk))
        else {
            return Err(JsError::new("Failed to fetch stake pool accounts"));
        };

        self.update_stake_pool(stake_pool_data)?;
        self.update_validator_list(validator_list_data)?;

        self.reserve_stake_lamports =
            get_account(accounts, self.stake_pool.reserve_stake)?.lamports;
        Ok(())
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct ValidatorListOwned {
    pub header: ValidatorListHeader,
    pub validators: Vec<ValidatorStakeInfo>,
}

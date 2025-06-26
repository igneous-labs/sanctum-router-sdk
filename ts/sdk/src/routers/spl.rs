use bs58_fixed_wasm::Bs58Array;
use sanctum_router_core::{
    SplDepositSolQuoter, SplDepositStakeQuoter, SplDepositStakeSufAccs, SplSolSufAccs,
    SplWithdrawSolQuoter, SplWithdrawStakeQuoter, SplWithdrawStakeSufAccs,
};
use sanctum_spl_stake_pool_core::{
    SplStakePoolError, StakePool, ValidatorList, ValidatorListHeader, ValidatorStakeInfo,
    SYSVAR_CLOCK,
};
use wasm_bindgen::JsError;

use crate::{
    err::{account_missing_err, invalid_pda_err, spl_err},
    init::{InitData, SplInitData},
    interface::{get_account, get_account_data, AccountMap},
    pda::spl::{
        find_deposit_auth_pda_internal, find_validator_stake_account_pda_internal,
        find_withdraw_auth_pda_internal,
    },
    update::PoolUpdateType,
};

#[derive(Clone, Debug, Default, PartialEq)]
pub struct SplStakePoolRouterOwned {
    pub stake_pool_program: [u8; 32],
    pub stake_pool_addr: [u8; 32],

    // Below 2 keys are duplicated in stake_pool data but
    // we set them at init time to make init() not require any
    // account data input and for update() to be immediately
    // ready after init, as opposed to needing to update()
    // twice in a row
    pub validator_list_addr: [u8; 32],
    pub reserve_stake_addr: [u8; 32],

    // PDAs
    pub deposit_authority_program_address: [u8; 32],
    pub withdraw_authority_program_address: [u8; 32],

    // Accounts
    pub stake_pool: Option<StakePool>,
    pub validator_list: Option<ValidatorListOwned>,
    pub reserve_stake_lamports: Option<u64>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct ValidatorListOwned {
    pub header: ValidatorListHeader,
    pub validators: Vec<ValidatorStakeInfo>,
}

/// Init
impl SplStakePoolRouterOwned {
    pub fn init(
        InitData::Spl(SplInitData {
            stake_pool_program_addr: Bs58Array(stake_pool_program_addr),
            stake_pool_addr: Bs58Array(stake_pool_addr),
            validator_list_addr: Bs58Array(validator_list_addr),
            reserve_stake_addr: Bs58Array(reserve_stake_addr),
        }): &InitData,
    ) -> Result<Self, JsError> {
        Ok(SplStakePoolRouterOwned {
            stake_pool_program: *stake_pool_program_addr,
            stake_pool_addr: *stake_pool_addr,
            validator_list_addr: *validator_list_addr,
            reserve_stake_addr: *reserve_stake_addr,
            deposit_authority_program_address: find_deposit_auth_pda_internal(
                stake_pool_program_addr,
                stake_pool_addr,
            )
            .ok_or_else(invalid_pda_err)?
            .0,
            withdraw_authority_program_address: find_withdraw_auth_pda_internal(
                stake_pool_program_addr,
                stake_pool_addr,
            )
            .ok_or_else(invalid_pda_err)?
            .0,
            stake_pool: Default::default(),
            validator_list: Default::default(),
            reserve_stake_lamports: Default::default(),
        })
    }
}

/// Getters
impl SplStakePoolRouterOwned {
    pub fn try_stake_pool(&self) -> Result<&StakePool, JsError> {
        self.stake_pool
            .as_ref()
            .ok_or_else(|| account_missing_err(&self.stake_pool_addr))
    }

    pub fn try_validator_list(&self) -> Result<&[ValidatorStakeInfo], JsError> {
        self.validator_list
            .as_ref()
            .ok_or_else(|| account_missing_err(&self.validator_list_addr))
            .map(|vl| vl.validators.as_slice())
    }

    pub fn try_reserve_stake_lamports(&self) -> Result<u64, JsError> {
        self.reserve_stake_lamports
            .ok_or_else(|| account_missing_err(&self.reserve_stake_addr))
    }
}

/// DepositSol + WithdrawSol common
impl SplStakePoolRouterOwned {
    pub fn sol_suf_accs(&self) -> Result<SplSolSufAccs, JsError> {
        Ok(SplSolSufAccs {
            stake_pool: self.try_stake_pool()?,
            stake_pool_program: &self.stake_pool_program,
            stake_pool_addr: &self.stake_pool_addr,
            withdraw_authority_program_address: &self.withdraw_authority_program_address,
        })
    }
}

/// DepositSol
impl SplStakePoolRouterOwned {
    pub fn deposit_sol_quoter(&self, curr_epoch: u64) -> Result<SplDepositSolQuoter, JsError> {
        Ok(SplDepositSolQuoter {
            stake_pool: self.try_stake_pool()?,
            curr_epoch,
        })
    }
}

/// WithdrawSol
impl SplStakePoolRouterOwned {
    pub fn withdraw_sol_quoter(&self, curr_epoch: u64) -> Result<SplWithdrawSolQuoter, JsError> {
        Ok(SplWithdrawSolQuoter {
            stake_pool: self.try_stake_pool()?,
            reserve_stake_lamports: self.try_reserve_stake_lamports()?,
            curr_epoch,
        })
    }
}

/// DepositStake
impl SplStakePoolRouterOwned {
    pub fn deposit_stake_quoter(&self, curr_epoch: u64) -> Result<SplDepositStakeQuoter, JsError> {
        Ok(SplDepositStakeQuoter {
            stake_pool: self.try_stake_pool()?,
            curr_epoch,
            validator_list: self.try_validator_list()?,
            default_stake_deposit_authority: &self.deposit_authority_program_address,
        })
    }

    pub fn deposit_stake_suf_accs(
        &self,
        vote_account: &[u8; 32],
    ) -> Result<SplDepositStakeSufAccs, JsError> {
        let validator_stake_info = self
            .try_validator_list()?
            .iter()
            .find(|v| v.vote_account_address() == vote_account)
            .ok_or_else(|| spl_err(SplStakePoolError::ValidatorNotFound))?;
        Ok(SplDepositStakeSufAccs {
            stake_pool_addr: &self.stake_pool_addr,
            stake_pool_program: &self.stake_pool_program,
            stake_pool: self.try_stake_pool()?,
            validator_stake: find_validator_stake_account_pda_internal(
                &self.stake_pool_program,
                validator_stake_info.vote_account_address(),
                &self.stake_pool_addr,
                validator_stake_info.validator_seed_suffix(),
            )
            .ok_or_else(invalid_pda_err)?
            .0,
            stake_deposit_authority: &self.deposit_authority_program_address,
            stake_withdraw_authority: &self.withdraw_authority_program_address,
        })
    }
}

/// WithdrawStake
impl SplStakePoolRouterOwned {
    pub fn withdraw_stake_quoter(
        &self,
        curr_epoch: u64,
    ) -> Result<SplWithdrawStakeQuoter, JsError> {
        Ok(SplWithdrawStakeQuoter {
            stake_pool: self.try_stake_pool()?,
            curr_epoch,
            validator_list: self.try_validator_list()?,
        })
    }

    /// Returns `None` if vote acc not on validator list or validator stake acc PDA invalid
    pub fn withdraw_stake_suf_accs(
        &self,
        vote_account: &[u8; 32],
    ) -> Result<SplWithdrawStakeSufAccs, JsError> {
        let validator_stake_info = self
            .try_validator_list()?
            .iter()
            .find(|v| v.vote_account_address() == vote_account)
            .ok_or_else(|| spl_err(SplStakePoolError::ValidatorNotFound))?;
        Ok(SplWithdrawStakeSufAccs {
            stake_pool_addr: &self.stake_pool_addr,
            stake_pool_program: &self.stake_pool_program,
            stake_pool: self.try_stake_pool()?,
            validator_stake: find_validator_stake_account_pda_internal(
                &self.stake_pool_program,
                validator_stake_info.vote_account_address(),
                &self.stake_pool_addr,
                validator_stake_info.validator_seed_suffix(),
            )
            .ok_or_else(invalid_pda_err)?
            .0,
            stake_withdraw_authority: &self.withdraw_authority_program_address,
        })
    }
}

/// Update
impl SplStakePoolRouterOwned {
    pub fn update_stake_pool(&mut self, stake_pool_data: &[u8]) -> Result<(), JsError> {
        self.stake_pool = Some(StakePool::borsh_de(stake_pool_data)?);
        Ok(())
    }

    pub fn update_validator_list(&mut self, validator_list_data: &[u8]) -> Result<(), JsError> {
        let validator_list = ValidatorList::deserialize(validator_list_data)?;
        self.validator_list = Some(ValidatorListOwned {
            header: validator_list.header,
            validators: validator_list.validators.to_vec(),
        });
        Ok(())
    }

    pub fn accounts_to_update(&self, ty: PoolUpdateType) -> impl Iterator<Item = [u8; 32]> {
        match ty {
            PoolUpdateType::DepositSol => {
                [Some(SYSVAR_CLOCK), Some(self.stake_pool_addr), None, None]
            }
            PoolUpdateType::WithdrawSol => [
                Some(SYSVAR_CLOCK),
                Some(self.stake_pool_addr),
                Some(self.reserve_stake_addr),
                None,
            ],
            PoolUpdateType::DepositStake | PoolUpdateType::WithdrawStake => [
                Some(SYSVAR_CLOCK),
                Some(self.stake_pool_addr),
                Some(self.validator_list_addr),
                None,
            ],
        }
        .into_iter()
        .flatten()
    }

    pub fn update(&mut self, ty: PoolUpdateType, accounts: &AccountMap) -> Result<(), JsError> {
        let stake_pool_data = get_account_data(accounts, self.stake_pool_addr)?;
        self.update_stake_pool(stake_pool_data)?;

        match ty {
            PoolUpdateType::DepositSol => Ok(()),
            PoolUpdateType::WithdrawSol => {
                self.reserve_stake_lamports =
                    Some(get_account(accounts, self.reserve_stake_addr)?.lamports);
                Ok(())
            }
            PoolUpdateType::DepositStake | PoolUpdateType::WithdrawStake => {
                let validator_list_data = get_account_data(accounts, self.validator_list_addr)?;
                self.update_validator_list(validator_list_data)
            }
        }
    }
}

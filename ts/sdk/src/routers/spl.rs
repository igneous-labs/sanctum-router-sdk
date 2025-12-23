use bs58_fixed_wasm::Bs58Array;
use sanctum_router_std::{
    sanctum_spl_stake_pool_core::{StakePool, ValidatorList, ValidatorStakeInfo, SYSVAR_CLOCK},
    SplDepositSolQuoter, SplDepositStakeQuoter, SplDepositStakeSufAccs, SplRouterDepositSol,
    SplRouterSol, SplSolSufAccs, SplWithdrawSolQuoter, SplWithdrawStakeQuoter,
    SplWithdrawStakeSufAccs,
};

use crate::{
    err::{account_missing_err, invalid_data_err, invalid_pda_err, SanctumRouterError},
    init::{InitData, SplInitData},
    interface::{get_account, AccountMap},
    pda::spl::{find_deposit_auth_pda_internal, find_withdraw_auth_pda_internal},
    update::PoolUpdateType,
};

pub type SplRouterStake = sanctum_router_std::SplRouterStake<
    Box<[ValidatorStakeInfo]>,
    fn(&[&[u8]], &[u8; 32]) -> Option<([u8; 32], u8)>,
>;

pub type SplRouter = sanctum_router_std::SplRouter<
    Box<[ValidatorStakeInfo]>,
    fn(&[&[u8]], &[u8; 32]) -> Option<([u8; 32], u8)>,
>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SplInit {
    pub stake_pool_program: [u8; 32],
    pub withdraw_authority_program_address: [u8; 32],
    pub stake_pool_addr: [u8; 32],
    pub validator_list_addr: [u8; 32],
    pub reserve_stake_addr: [u8; 32],
}

/// Notes
/// - `curr_epoch` field in this struct is not used, but patched with the shared one in
///   [`crate::router::SanctumRouter`] at quoting time
#[derive(Clone, Debug, PartialEq)]
pub enum SplRouterOwned {
    Init(SplInit),
    DepositSol(SplRouterDepositSol),
    Sol(SplRouterSol),
    Stake(SplRouterStake),
    Full(SplRouter),
}

macro_rules! cmn_field {
    ($field:ident, $T:ty) => {
        pub const fn $field(&self) -> $T {
            match self {
                Self::Init(SplInit { $field, .. })
                | Self::DepositSol(SplRouterDepositSol { $field, .. })
                | Self::Sol(SplRouterSol { $field, .. })
                | Self::Stake(SplRouterStake { $field, .. })
                | Self::Full(SplRouter { $field, .. }) => $field,
            }
        }
    };
}

macro_rules! sp_field {
    ($field:ident) => {
        Self::DepositSol(SplRouterDepositSol {
            stake_pool: StakePool { $field, .. },
            ..
        }) | Self::Sol(SplRouterSol {
            stake_pool: StakePool { $field, .. },
            ..
        }) | Self::Stake(SplRouterStake {
            stake_pool: StakePool { $field, .. },
            ..
        }) | Self::Full(SplRouter {
            stake_pool: StakePool { $field, .. },
            ..
        })
    };
}

/// Getters
impl SplRouterOwned {
    cmn_field!(stake_pool_program, &[u8; 32]);
    cmn_field!(withdraw_authority_program_address, &[u8; 32]);
    cmn_field!(stake_pool_addr, &[u8; 32]);

    pub const fn reserve_stake_addr(&self) -> &[u8; 32] {
        match self {
            Self::Init(SplInit {
                reserve_stake_addr, ..
            }) => reserve_stake_addr,
            sp_field!(reserve_stake) => reserve_stake,
        }
    }

    pub const fn validator_list_addr(&self) -> &[u8; 32] {
        match self {
            Self::Init(SplInit {
                validator_list_addr,
                ..
            }) => validator_list_addr,
            sp_field!(validator_list) => validator_list,
        }
    }

    pub fn try_stake_pool(&self) -> Result<&StakePool, SanctumRouterError> {
        match self {
            Self::Init(_) => Err(account_missing_err(self.validator_list_addr())),
            Self::DepositSol(SplRouterDepositSol { stake_pool, .. })
            | Self::Sol(SplRouterSol { stake_pool, .. })
            | Self::Stake(SplRouterStake { stake_pool, .. })
            | Self::Full(SplRouter { stake_pool, .. }) => Ok(stake_pool),
        }
    }

    pub fn try_validator_list(&self) -> Result<&[ValidatorStakeInfo], SanctumRouterError> {
        match self {
            Self::Init(_) | Self::DepositSol(_) | Self::Sol(_) => {
                Err(account_missing_err(self.validator_list_addr()))
            }
            Self::Stake(SplRouterStake { validator_list, .. })
            | Self::Full(SplRouter { validator_list, .. }) => Ok(validator_list),
        }
    }

    pub fn try_reserve_stake_lamports(&self) -> Result<u64, SanctumRouterError> {
        match self {
            Self::Init(_) | Self::DepositSol(_) | Self::Stake(_) => {
                Err(account_missing_err(self.reserve_stake_addr()))
            }
            Self::Sol(SplRouterSol {
                reserve_stake_lamports,
                ..
            })
            | Self::Full(SplRouter {
                reserve_stake_lamports,
                ..
            }) => Ok(*reserve_stake_lamports),
        }
    }

    const fn try_default_stake_deposit_authority(&self) -> Option<[u8; 32]> {
        match self {
            Self::Init(_) | Self::DepositSol(_) | Self::Sol(_) => None,
            Self::Stake(SplRouterStake {
                default_stake_deposit_authority,
                ..
            })
            | Self::Full(SplRouter {
                default_stake_deposit_authority,
                ..
            }) => Some(*default_stake_deposit_authority),
        }
    }
}

/// Init
impl SplRouterOwned {
    pub fn init(
        InitData::Spl(SplInitData {
            stake_pool_program_addr: Bs58Array(stake_pool_program_addr),
            stake_pool_addr: Bs58Array(stake_pool_addr),
            validator_list_addr: Bs58Array(validator_list_addr),
            reserve_stake_addr: Bs58Array(reserve_stake_addr),
        }): &InitData,
    ) -> Result<Self, SanctumRouterError> {
        Ok(SplRouterOwned::Init(SplInit {
            stake_pool_program: *stake_pool_program_addr,
            stake_pool_addr: *stake_pool_addr,
            validator_list_addr: *validator_list_addr,
            reserve_stake_addr: *reserve_stake_addr,
            withdraw_authority_program_address: find_withdraw_auth_pda_internal(
                stake_pool_program_addr,
                stake_pool_addr,
            )
            .ok_or_else(invalid_pda_err)?
            .0,
        }))
    }
}

/// DepositSol + WithdrawSol common
impl SplRouterOwned {
    pub fn sol_suf_accs(&self) -> Result<SplSolSufAccs<'_>, SanctumRouterError> {
        match self {
            Self::Init(_) => Err(account_missing_err(self.stake_pool_addr())),
            Self::DepositSol(s) => Ok(s.spl_sol_suf_accs()),
            Self::Sol(s) => Ok(s.spl_sol_suf_accs()),
            Self::Stake(s) => Ok(s.spl_sol_suf_accs()),
            Self::Full(s) => Ok(s.spl_sol_suf_accs()),
        }
    }
}

/// DepositSol
impl SplRouterOwned {
    pub fn deposit_sol_quoter(
        &self,
        curr_epoch: u64,
    ) -> Result<SplDepositSolQuoter<'_>, SanctumRouterError> {
        let quoter = match self {
            Self::Init(_) => return Err(account_missing_err(self.stake_pool_addr())),
            Self::DepositSol(s) => s.spl_deposit_sol_quoter(),
            Self::Sol(s) => s.spl_deposit_sol_quoter(),
            Self::Stake(s) => s.spl_deposit_sol_quoter(),
            Self::Full(s) => s.spl_deposit_sol_quoter(),
        };
        Ok(SplDepositSolQuoter {
            curr_epoch,
            ..quoter
        })
    }
}

/// WithdrawSol
impl SplRouterOwned {
    pub fn withdraw_sol_quoter(
        &self,
        curr_epoch: u64,
    ) -> Result<SplWithdrawSolQuoter<'_>, SanctumRouterError> {
        let quoter = match self {
            Self::Init(_) => return Err(account_missing_err(self.stake_pool_addr())),
            Self::DepositSol(_) | Self::Stake(_) => {
                return Err(account_missing_err(self.reserve_stake_addr()))
            }
            Self::Sol(s) => s.spl_withdraw_sol_quoter(),
            Self::Full(s) => s.spl_withdraw_sol_quoter(),
        };
        Ok(SplWithdrawSolQuoter {
            curr_epoch,
            ..quoter
        })
    }
}

/// DepositStake
impl SplRouterOwned {
    pub fn deposit_stake_quoter(
        &self,
        curr_epoch: u64,
    ) -> Result<SplDepositStakeQuoter<'_>, SanctumRouterError> {
        let quoter = match self {
            Self::Init(_) | Self::DepositSol(_) | Self::Sol(_) => {
                return Err(account_missing_err(self.validator_list_addr()))
            }
            Self::Stake(s) => s.spl_deposit_stake_quoter(),
            Self::Full(s) => s.spl_deposit_stake_quoter(),
        };
        Ok(SplDepositStakeQuoter {
            curr_epoch,
            ..quoter
        })
    }

    pub fn deposit_stake_suf_accs(
        &self,
        vote_account: &[u8; 32],
    ) -> Result<SplDepositStakeSufAccs<'_>, SanctumRouterError> {
        match self {
            Self::Init(_) | Self::DepositSol(_) | Self::Sol(_) => {
                return Err(account_missing_err(self.validator_list_addr()));
            }
            Self::Stake(s) => s.spl_deposit_stake_suf_accs(vote_account),
            Self::Full(s) => s.spl_deposit_stake_suf_accs(vote_account),
        }
        .ok_or_else(invalid_pda_err)
    }
}

/// WithdrawStake
impl SplRouterOwned {
    pub fn withdraw_stake_quoter(
        &self,
        curr_epoch: u64,
    ) -> Result<SplWithdrawStakeQuoter<'_>, SanctumRouterError> {
        let quoter = match self {
            Self::Init(_) | Self::DepositSol(_) | Self::Sol(_) => {
                return Err(account_missing_err(self.validator_list_addr()))
            }
            Self::Stake(s) => s.spl_withdraw_stake_quoter(),
            Self::Full(s) => s.spl_withdraw_stake_quoter(),
        };
        Ok(SplWithdrawStakeQuoter {
            curr_epoch,
            ..quoter
        })
    }

    /// Returns `None` if vote acc not on validator list or validator stake acc PDA invalid
    pub fn withdraw_stake_suf_accs(
        &self,
        vote_account: &[u8; 32],
    ) -> Result<SplWithdrawStakeSufAccs<'_>, SanctumRouterError> {
        match self {
            Self::Init(_) | Self::DepositSol(_) | Self::Sol(_) => {
                return Err(account_missing_err(self.validator_list_addr()));
            }
            Self::Stake(s) => s.spl_withdraw_stake_suf_accs(vote_account),
            Self::Full(s) => s.spl_withdraw_stake_suf_accs(vote_account),
        }
        .ok_or_else(invalid_pda_err)
    }
}

/// Update
impl SplRouterOwned {
    pub fn accounts_to_update(&self, ty: PoolUpdateType) -> impl Iterator<Item = [u8; 32]> {
        match ty {
            PoolUpdateType::DepositSol => [Some(SYSVAR_CLOCK), Some(*self.stake_pool_addr()), None],
            PoolUpdateType::WithdrawSol => [
                Some(SYSVAR_CLOCK),
                Some(*self.stake_pool_addr()),
                Some(*self.reserve_stake_addr()),
            ],
            PoolUpdateType::DepositStake | PoolUpdateType::WithdrawStake => [
                Some(SYSVAR_CLOCK),
                Some(*self.stake_pool_addr()),
                Some(*self.validator_list_addr()),
            ],
        }
        .into_iter()
        .flatten()
    }

    pub fn update(
        &mut self,
        ty: PoolUpdateType,
        accounts: &AccountMap,
    ) -> Result<(), SanctumRouterError> {
        let [s, v, r] = [
            self.stake_pool_addr(),
            self.validator_list_addr(),
            self.reserve_stake_addr(),
        ]
        .map(|a| get_account(accounts, *a));

        // stake pool must always be fetched
        let stake_pool =
            StakePool::borsh_de(s?.data.as_slice()).map_err(|_e| invalid_data_err())?;

        let reserve_stake_lamports_opt = match ty {
            // reserve must be fetched on withdrawSol
            PoolUpdateType::WithdrawSol => Some(r?.lamports),
            // else use the existing one if it exists
            PoolUpdateType::DepositSol
            | PoolUpdateType::DepositStake
            | PoolUpdateType::WithdrawStake => self.try_reserve_stake_lamports().ok(),
        };

        let [stake_pool_program, stake_pool_addr, withdraw_authority_program_address] = [
            Self::stake_pool_program,
            Self::stake_pool_addr,
            Self::withdraw_authority_program_address,
        ]
        .map(|get| *get(self));

        // dummy val
        let curr_epoch = 0;

        let default_stake_deposit_authority_opt = self.try_default_stake_deposit_authority();

        let fresh_val_list_res: Result<Box<[ValidatorStakeInfo]>, SanctumRouterError> = v
            .and_then(|v| ValidatorList::deserialize(&v.data).map_err(|_| invalid_data_err()))
            .map(|vlist| vlist.validators.into());

        let val_list_opt: Option<&mut Box<[ValidatorStakeInfo]>> = match ty {
            // val list must be fetched for these update types
            PoolUpdateType::DepositStake | PoolUpdateType::WithdrawStake => {
                Some(&mut fresh_val_list_res?)
            }
            // else use the existing one if it exists
            PoolUpdateType::DepositSol | PoolUpdateType::WithdrawSol => match self {
                Self::Init(_) | Self::DepositSol(_) | Self::Sol(_) => None,
                Self::Stake(SplRouterStake { validator_list, .. })
                | Self::Full(SplRouter { validator_list, .. }) => Some(validator_list),
            },
        };

        let new_state = match (reserve_stake_lamports_opt, val_list_opt) {
            (None, None) => Self::DepositSol(SplRouterDepositSol {
                stake_pool_program,
                stake_pool_addr,
                withdraw_authority_program_address,
                stake_pool,
                curr_epoch,
            }),
            (Some(reserve_stake_lamports), None) => Self::Sol(SplRouterSol {
                stake_pool_program,
                stake_pool_addr,
                withdraw_authority_program_address,
                stake_pool,
                curr_epoch,
                reserve_stake_lamports,
            }),
            (reserve_stake_lamports_opt, Some(val_list)) => {
                let default_stake_deposit_authority = default_stake_deposit_authority_opt
                    .or_else(|| {
                        find_deposit_auth_pda_internal(&stake_pool_program, &stake_pool_addr)
                            .map(|(addr, _)| addr)
                    })
                    .ok_or_else(invalid_pda_err)?;
                // we can take() validator_list here because we will infallibly
                // mutate self immediately after
                match reserve_stake_lamports_opt {
                    None => Self::Stake(SplRouterStake {
                        stake_pool_program,
                        stake_pool_addr,
                        withdraw_authority_program_address,
                        stake_pool,
                        curr_epoch,
                        default_stake_deposit_authority,
                        validator_list: core::mem::take(val_list),
                        find_pda: crate::pda::find_pda,
                    }),
                    Some(reserve_stake_lamports) => Self::Full(SplRouter {
                        stake_pool_program,
                        stake_pool_addr,
                        withdraw_authority_program_address,
                        stake_pool,
                        curr_epoch,
                        reserve_stake_lamports,
                        default_stake_deposit_authority,
                        validator_list: core::mem::take(val_list),
                        find_pda: crate::pda::find_pda,
                    }),
                }
            }
        };

        *self = new_state;

        Ok(())
    }
}

use sanctum_router_std::{
    sanctum_marinade_liquid_staking_core::{
        self, State as MarinadeState, ValidatorList, ValidatorRecord, LIQ_POOL_MSOL_LEG_PUBKEY,
        MSOL_MINT_ADDR, STATE_PUBKEY, VALIDATOR_LIST_PUBKEY,
    },
    MarinadeDepositSolQuoter, MarinadeDepositSolSufAccs, MarinadeDepositStakeQuoter,
    MarinadeDepositStakeSufAccs, MarinadeRouterSol,
};

use crate::{
    err::{
        account_missing_err, invalid_data_err, invalid_pda_err, unsupported_update_err,
        SanctumRouterError,
    },
    interface::{get_account_data, AccountMap},
    update::PoolUpdateType,
};

pub type MarinadeRouter = sanctum_router_std::MarinadeRouter<
    Box<[ValidatorRecord]>,
    fn(&[&[u8]], &[u8; 32]) -> Option<([u8; 32], u8)>,
>;

#[derive(Clone, Debug, Default, PartialEq)]
pub enum MarinadeRouterOwned {
    #[default]
    Uninit,
    Sol(MarinadeRouterSol),
    Full(MarinadeRouter),
}

/// DepositSol
impl MarinadeRouterOwned {
    pub fn deposit_sol_quoter(&self) -> Result<MarinadeDepositSolQuoter<'_>, SanctumRouterError> {
        match self {
            Self::Uninit => Err(account_missing_err(&STATE_PUBKEY)),
            Self::Sol(s) => Ok(s.marinade_deposit_sol_quoter()),
            Self::Full(s) => Ok(s.marinade_deposit_sol_quoter()),
        }
    }

    pub fn deposit_sol_suf_accs(
        &self,
    ) -> Result<MarinadeDepositSolSufAccs<'_>, SanctumRouterError> {
        match self {
            Self::Uninit => Err(account_missing_err(&STATE_PUBKEY)),
            Self::Sol(s) => Ok(s.marinade_deposit_sol_suf_accs()),
            Self::Full(s) => Ok(s.marinade_deposit_sol_suf_accs()),
        }
    }
}

/// DepositStake
impl MarinadeRouterOwned {
    pub fn deposit_stake_quoter(
        &self,
    ) -> Result<MarinadeDepositStakeQuoter<'_>, SanctumRouterError> {
        match self {
            Self::Uninit => Err(account_missing_err(&STATE_PUBKEY)),
            Self::Sol(_) => Err(account_missing_err(&VALIDATOR_LIST_PUBKEY)),
            Self::Full(s) => Ok(s.marinade_deposit_stake_quoter()),
        }
    }

    pub fn deposit_stake_suf_accs(
        &self,
        vote_account: &[u8; 32],
    ) -> Result<MarinadeDepositStakeSufAccs<'_>, SanctumRouterError> {
        match self {
            Self::Uninit => Err(account_missing_err(&STATE_PUBKEY)),
            Self::Sol(_) => Err(account_missing_err(&VALIDATOR_LIST_PUBKEY)),
            Self::Full(s) => s
                .marinade_deposit_stake_suf_accs(vote_account)
                .ok_or_else(invalid_pda_err),
        }
    }
}

/// Update
impl MarinadeRouterOwned {
    pub fn accounts_to_update(ty: PoolUpdateType) -> impl Iterator<Item = [u8; 32]> {
        match ty {
            PoolUpdateType::DepositSol => {
                [Some(STATE_PUBKEY), Some(LIQ_POOL_MSOL_LEG_PUBKEY), None]
            }
            PoolUpdateType::DepositStake => [
                STATE_PUBKEY,
                LIQ_POOL_MSOL_LEG_PUBKEY,
                VALIDATOR_LIST_PUBKEY,
            ]
            .map(Some),
            _ => [None; 3],
        }
        .into_iter()
        .flatten()
    }

    pub fn update(
        &mut self,
        ty: PoolUpdateType,
        accounts: &AccountMap,
    ) -> Result<(), SanctumRouterError> {
        match ty {
            PoolUpdateType::DepositSol | PoolUpdateType::DepositStake => {
                let [s, m, validator_records_data_opt] = [
                    // these 2 accs are always fetched regardless of update type
                    sanctum_marinade_liquid_staking_core::STATE_PUBKEY,
                    sanctum_marinade_liquid_staking_core::LIQ_POOL_MSOL_LEG_PUBKEY,
                    // val list only required for DepositStake
                    sanctum_marinade_liquid_staking_core::VALIDATOR_LIST_PUBKEY,
                ]
                .map(|k| get_account_data(accounts, k));
                let state = MarinadeState::borsh_de(s?).map_err(|_e| invalid_data_err())?;
                let msol_leg_balance = try_token_acc_amt(m?)?;

                let new_state = match validator_records_data_opt {
                    Ok(validator_records_data) => {
                        let vlist = ValidatorList::try_from_acc_data(
                            validator_records_data,
                            state.validator_system.validator_list.len() as usize,
                        )
                        .ok_or_else(invalid_data_err)?;
                        Self::Full(MarinadeRouter {
                            state,
                            msol_leg_balance,
                            validator_records: vlist.0.into(),
                            find_pda: crate::pda::find_pda,
                        })
                    }
                    Err(_not_fetched) => Self::Sol(MarinadeRouterSol {
                        state,
                        msol_leg_balance,
                    }),
                };

                *self = new_state;

                Ok(())
            }
            PoolUpdateType::WithdrawSol | PoolUpdateType::WithdrawStake => {
                Err(unsupported_update_err(ty, &MSOL_MINT_ADDR))
            }
        }
    }
}

fn try_token_acc_amt(d: &[u8]) -> Result<u64, SanctumRouterError> {
    Ok(u64::from_le_bytes(
        *d.get(..72)
            .and_then(|s| s.last_chunk())
            .ok_or_else(invalid_data_err)?,
    ))
}

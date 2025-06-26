use sanctum_marinade_liquid_staking_core::{
    State as MarinadeState, ValidatorList, ValidatorRecord, LIQ_POOL_MSOL_LEG_PUBKEY,
    MSOL_MINT_ADDR, STATE_PUBKEY, VALIDATOR_LIST_PUBKEY,
};
use sanctum_router_core::{
    MarinadeDepositSolQuoter, MarinadeDepositSolSufAccs, MarinadeDepositStakeQuoter,
    MarinadeDepositStakeSufAccs,
};

use crate::{
    err::{
        account_missing_err, invalid_data_err, invalid_pda_err, unsupported_update_err,
        SanctumRouterError,
    },
    interface::{get_account_data, AccountMap},
    pda::marinade::find_marinade_duplication_flag_pda_internal,
    update::PoolUpdateType,
};

#[derive(Clone, Debug, Default, PartialEq)]
pub struct MarinadeRouterOwned {
    pub state: Option<MarinadeState>,
    pub validator_records: Option<Vec<ValidatorRecord>>,
    pub msol_leg_balance: Option<u64>,
}

/// Getters
impl MarinadeRouterOwned {
    pub fn try_state(&self) -> Result<&MarinadeState, SanctumRouterError> {
        self.state
            .as_ref()
            .ok_or_else(|| account_missing_err(&STATE_PUBKEY))
    }

    pub fn try_validator_records(&self) -> Result<&[ValidatorRecord], SanctumRouterError> {
        self.validator_records
            .as_ref()
            .ok_or_else(|| account_missing_err(&VALIDATOR_LIST_PUBKEY))
            .map(|v| v.as_slice())
    }

    pub fn try_msol_leg_balance(&self) -> Result<u64, SanctumRouterError> {
        self.msol_leg_balance
            .ok_or_else(|| account_missing_err(&LIQ_POOL_MSOL_LEG_PUBKEY))
    }
}

/// DepositSol
impl MarinadeRouterOwned {
    pub fn deposit_sol_quoter(&self) -> Result<MarinadeDepositSolQuoter, SanctumRouterError> {
        Ok(MarinadeDepositSolQuoter {
            state: self.try_state()?,
            msol_leg_balance: self.try_msol_leg_balance()?,
        })
    }

    pub fn deposit_sol_suf_accs(&self) -> Result<MarinadeDepositSolSufAccs, SanctumRouterError> {
        self.try_state().map(MarinadeDepositSolSufAccs::from_state)
    }
}

/// DepositStake
impl MarinadeRouterOwned {
    pub fn deposit_stake_quoter(&self) -> Result<MarinadeDepositStakeQuoter, SanctumRouterError> {
        Ok(MarinadeDepositStakeQuoter {
            state: self.try_state()?,
            msol_leg_balance: self.try_msol_leg_balance()?,
            validator_records: self.try_validator_records()?,
        })
    }

    pub fn deposit_stake_suf_accs(
        &self,
        vote_account: &[u8; 32],
    ) -> Result<MarinadeDepositStakeSufAccs, SanctumRouterError> {
        Ok(MarinadeDepositStakeSufAccs {
            state: self.try_state()?,
            duplication_flag: find_marinade_duplication_flag_pda_internal(vote_account)
                .ok_or_else(invalid_pda_err)?
                .0,
        })
    }
}

/// Update
impl MarinadeRouterOwned {
    pub fn update_state(&mut self, data: &[u8]) -> Result<(), SanctumRouterError> {
        self.state = Some(MarinadeState::borsh_de(data).map_err(|_e| invalid_data_err())?);
        Ok(())
    }

    pub fn update_validator_records(
        &mut self,
        validator_list_data: &[u8],
        count: usize,
    ) -> Result<(), SanctumRouterError> {
        let validator_list = ValidatorList::try_from_acc_data(validator_list_data, count)
            .ok_or_else(invalid_data_err)?;

        self.validator_records = Some(validator_list.0.to_vec());
        Ok(())
    }

    pub fn update_msol_leg_balance(
        &mut self,
        msol_leg_data: &[u8],
    ) -> Result<(), SanctumRouterError> {
        self.msol_leg_balance = Some(try_token_acc_amt(msol_leg_data)?);
        Ok(())
    }

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
                let [s, m] = [
                    sanctum_marinade_liquid_staking_core::STATE_PUBKEY,
                    sanctum_marinade_liquid_staking_core::LIQ_POOL_MSOL_LEG_PUBKEY,
                ]
                .map(|k| get_account_data(accounts, k));
                let state_data = s?;
                let msol_leg_data = m?;

                self.update_state(state_data)?;
                self.update_msol_leg_balance(msol_leg_data)?;

                if matches!(ty, PoolUpdateType::DepositStake) {
                    let validator_records_data = get_account_data(
                        accounts,
                        sanctum_marinade_liquid_staking_core::VALIDATOR_LIST_PUBKEY,
                    )?;
                    self.update_validator_records(
                        validator_records_data,
                        // unwrap-safety: state was just updated above
                        self.try_state()
                            .unwrap()
                            .validator_system
                            .validator_list
                            .len() as usize,
                    )?;
                }

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

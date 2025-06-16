use sanctum_marinade_liquid_staking_core::{
    State as MarinadeState, ValidatorList, ValidatorRecord, MSOL_MINT_ADDR,
};
use sanctum_router_core::{
    MarinadeDepositSolQuoter, MarinadeDepositSolSufAccs, MarinadeDepositStakeQuoter,
    MarinadeDepositStakeSufAccs,
};
use wasm_bindgen::JsError;

use crate::{
    err::{invalid_data_err, unsupported_update},
    interface::{get_account_data, AccountMap},
    pda::marinade::find_marinade_duplication_flag_pda_internal,
    update::{PoolUpdateType, Update},
};

#[derive(Clone, Debug, PartialEq)]
pub struct MarinadeRouterOwned {
    pub state: MarinadeState,
    pub validator_records: Vec<ValidatorRecord>,
    pub msol_leg_balance: u64,
}

/// Init
impl MarinadeRouterOwned {
    pub const fn init_accounts() -> [[u8; 32]; 3] {
        [
            sanctum_marinade_liquid_staking_core::STATE_PUBKEY,
            sanctum_marinade_liquid_staking_core::LIQ_POOL_MSOL_LEG_PUBKEY,
            sanctum_marinade_liquid_staking_core::VALIDATOR_LIST_PUBKEY,
        ]
    }

    pub fn init(accounts: &AccountMap) -> Result<Self, JsError> {
        let [s, m, v] = Self::init_accounts().map(|k| get_account_data(accounts, k));
        let state_data = s?;
        let msol_leg_data = m?;
        let validator_records_data = v?;

        // TODO: impl Default for MarinadeState in
        // sanctum_marinade_liquid_staking_core
        let mut res = Self {
            state: MarinadeState::borsh_de(state_data)?,
            validator_records: Default::default(),
            msol_leg_balance: Default::default(),
        };
        res.update_msol_leg_balance(msol_leg_data)?;
        res.update_validator_records(
            validator_records_data,
            res.state.validator_system.validator_list.len() as usize,
        )?;

        Ok(res)
    }
}

/// DepositSol
impl MarinadeRouterOwned {
    pub fn deposit_sol_quoter(&self) -> MarinadeDepositSolQuoter {
        MarinadeDepositSolQuoter {
            state: &self.state,
            msol_leg_balance: self.msol_leg_balance,
        }
    }

    pub fn deposit_sol_suf_accs(&self) -> MarinadeDepositSolSufAccs {
        MarinadeDepositSolSufAccs::from_state(&self.state)
    }
}

/// DepositStake
impl MarinadeRouterOwned {
    pub fn deposit_stake_quoter(&self) -> MarinadeDepositStakeQuoter {
        MarinadeDepositStakeQuoter {
            state: &self.state,
            msol_leg_balance: self.msol_leg_balance,
            validator_records: &self.validator_records,
        }
    }

    pub fn deposit_stake_suf_accs(
        &self,
        vote_account: &[u8; 32],
    ) -> Option<MarinadeDepositStakeSufAccs> {
        Some(MarinadeDepositStakeSufAccs {
            state: &self.state,
            duplication_flag: find_marinade_duplication_flag_pda_internal(vote_account)?.0,
        })
    }
}

/// Update helpers
impl MarinadeRouterOwned {
    pub fn update_state(&mut self, data: &[u8]) -> Result<(), JsError> {
        self.state = MarinadeState::borsh_de(data)?;
        Ok(())
    }

    pub fn update_validator_records(
        &mut self,
        validator_list_data: &[u8],
        count: usize,
    ) -> Result<(), JsError> {
        let validator_list = ValidatorList::try_from_acc_data(validator_list_data, count)
            .ok_or_else(invalid_data_err)?;

        self.validator_records = validator_list.0.to_vec();
        Ok(())
    }

    pub fn update_msol_leg_balance(&mut self, msol_leg_data: &[u8]) -> Result<(), JsError> {
        self.msol_leg_balance = try_token_acc_amt(msol_leg_data)?;
        Ok(())
    }
}

impl Update for MarinadeRouterOwned {
    fn accounts_to_update(&self, ty: PoolUpdateType) -> impl Iterator<Item = [u8; 32]> {
        match ty {
            PoolUpdateType::DepositSol => [
                Some(sanctum_marinade_liquid_staking_core::STATE_PUBKEY),
                Some(sanctum_marinade_liquid_staking_core::LIQ_POOL_MSOL_LEG_PUBKEY),
                None,
            ],
            PoolUpdateType::DepositStake => [
                sanctum_marinade_liquid_staking_core::STATE_PUBKEY,
                sanctum_marinade_liquid_staking_core::LIQ_POOL_MSOL_LEG_PUBKEY,
                sanctum_marinade_liquid_staking_core::VALIDATOR_LIST_PUBKEY,
            ]
            .map(Some),
            _ => [None; 3],
        }
        .into_iter()
        .flatten()
    }

    fn update(&mut self, ty: PoolUpdateType, accounts: &AccountMap) -> Result<(), JsError> {
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
                        self.state.validator_system.validator_list.len() as usize,
                    )?;
                }

                Ok(())
            }
            PoolUpdateType::WithdrawSol | PoolUpdateType::WithdrawStake => {
                Err(unsupported_update(ty, &MSOL_MINT_ADDR))
            }
        }
    }
}

fn try_token_acc_amt(d: &[u8]) -> Result<u64, JsError> {
    Ok(u64::from_le_bytes(
        *d.get(..72)
            .and_then(|s| s.last_chunk())
            .ok_or_else(invalid_data_err)?,
    ))
}

use sanctum_marinade_liquid_staking_core::{
    State as MarinadeState, ValidatorList, ValidatorRecord,
};
use sanctum_router_core::{MarinadeSolQuoter, MarinadeStakeRouter};
use wasm_bindgen::JsError;

use crate::{
    err::invalid_data_err,
    interface::{get_account_data, AccountMap},
    pda::marinade::find_marinade_duplication_flag_pda_internal,
    update::Update,
};

#[derive(Clone, Debug, PartialEq)]
pub struct MarinadeRouterOwned {
    pub state: MarinadeState,
    pub validator_records: Vec<ValidatorRecord>,
    pub msol_leg_balance: u64,
}

impl MarinadeRouterOwned {
    pub fn to_deposit_sol_router(&self) -> MarinadeSolQuoter {
        MarinadeSolQuoter {
            state: &self.state,
            msol_leg_balance: self.msol_leg_balance,
        }
    }

    pub fn to_deposit_stake_router(&self, vote_account: &[u8; 32]) -> Option<MarinadeStakeRouter> {
        Some(MarinadeStakeRouter {
            state: &self.state,
            msol_leg_balance: self.msol_leg_balance,
            duplication_flag: find_marinade_duplication_flag_pda_internal(vote_account)?.0,
        })
    }

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
            .ok_or(invalid_data_err())?;

        self.validator_records = validator_list.0.to_vec();
        Ok(())
    }
}

impl Update for MarinadeRouterOwned {
    fn get_accounts_to_update(&self) -> impl Iterator<Item = [u8; 32]> {
        [
            sanctum_marinade_liquid_staking_core::STATE_PUBKEY,
            self.state.validator_system.validator_list.account,
            sanctum_marinade_liquid_staking_core::LIQ_POOL_MSOL_LEG_PUBKEY,
        ]
        .into_iter()
    }

    fn update(&mut self, accounts: &AccountMap) -> Result<(), JsError> {
        let [Ok(state_data), Ok(validator_records_data), Ok(msol_leg_data)] = [
            sanctum_marinade_liquid_staking_core::STATE_PUBKEY,
            sanctum_marinade_liquid_staking_core::VALIDATOR_LIST_PUBKEY,
            sanctum_marinade_liquid_staking_core::LIQ_POOL_MSOL_LEG_PUBKEY,
        ]
        .map(|pk| get_account_data(accounts, pk)) else {
            return Err(JsError::new("Failed to fetch marinade accounts"));
        };

        self.update_state(state_data)?;
        self.update_validator_records(
            validator_records_data,
            self.state.validator_system.validator_list.len() as usize,
        )?;
        // This is a token account, reading `amount`
        self.msol_leg_balance = u64::from_le_bytes(
            *msol_leg_data
                .get(..72)
                .and_then(|s| s.last_chunk::<8>())
                .ok_or_else(invalid_data_err)?,
        );
        Ok(())
    }
}

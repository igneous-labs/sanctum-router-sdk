use sanctum_router_core::{LidoWithdrawStakeQuoter, LidoWithdrawStakeSufAccs};
use solido_legacy_core::{Lido, ListHeader, Validator, ValidatorList};
use wasm_bindgen::JsError;

use crate::{
    interface::{get_account_data, AccountMap},
    pda::lido::find_lido_validator_stake_account_pda_internal,
    update::Update,
};

#[derive(Clone, Debug, PartialEq)]
pub struct LidoRouterOwned {
    pub state: Lido,
    pub validator_list: LidoValidatorListOwned,
}

/// WithdrawStake
impl LidoRouterOwned {
    /// Lido only allows withdrawing from max stake validator
    pub fn withdraw_stake_quoter(&self, curr_epoch: u64) -> Option<LidoWithdrawStakeQuoter> {
        LidoWithdrawStakeQuoter::new(&self.state, &self.validator_list.validators, curr_epoch)
    }

    /// Lido only allows withdrawing from max stake validator
    ///
    /// Returns `None` if data missing or validator stake acc PDA invalid
    pub fn withdraw_stake_suf_accs(&self) -> Option<LidoWithdrawStakeSufAccs> {
        let max_validator = self
            .validator_list
            .validators
            .iter()
            .max_by_key(|v| v.effective_stake_balance())?;
        let largest_stake_vote = max_validator.vote_account_address();
        Some(LidoWithdrawStakeSufAccs {
            validator_list_addr: &self.state.validator_list,
            largest_stake_vote,
            stake_to_split: find_lido_validator_stake_account_pda_internal(
                largest_stake_vote,
                max_validator.stake_seeds().begin(),
            )?
            .0,
        })
    }
}

/// Update helpers
impl LidoRouterOwned {
    pub fn update_state(&mut self, state_data: &[u8]) -> Result<(), JsError> {
        self.state = Lido::borsh_de(state_data)?;
        Ok(())
    }

    pub fn update_validator_list(&mut self, validator_list_data: &[u8]) -> Result<(), JsError> {
        let validator_list = ValidatorList::deserialize(validator_list_data)?;
        self.validator_list = LidoValidatorListOwned {
            header: validator_list.header,
            validators: validator_list.entries.to_vec(),
        };
        Ok(())
    }
}

impl Update for LidoRouterOwned {
    fn get_accounts_to_update(&self) -> impl Iterator<Item = [u8; 32]> {
        [
            solido_legacy_core::LIDO_STATE_ADDR,
            solido_legacy_core::VALIDATOR_LIST_ADDR,
        ]
        .into_iter()
    }

    fn update(&mut self, accounts: &AccountMap) -> Result<(), JsError> {
        let [Ok(state_data), Ok(validator_list_data)] = [
            solido_legacy_core::LIDO_STATE_ADDR,
            solido_legacy_core::VALIDATOR_LIST_ADDR,
        ]
        .map(|k| get_account_data(accounts, k)) else {
            return Err(JsError::new("Failed to fetch lido accounts"));
        };

        self.update_state(state_data)?;
        self.update_validator_list(validator_list_data)?;

        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Default)]
pub struct LidoValidatorListOwned {
    pub header: ListHeader,
    pub validators: Vec<Validator>,
}

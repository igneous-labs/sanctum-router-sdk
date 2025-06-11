use sanctum_router_core::LidoWithdrawStakeRouter;
use solido_legacy_core::{Lido, ListHeader, Validator, ValidatorList};
use wasm_bindgen::JsError;

use crate::{
    interface::{get_account_data, AccountMap},
    pda::lido::find_lido_validator_stake_account_pda_internal,
    router::Update,
};

#[derive(Clone, Debug, PartialEq)]
pub struct LidoRouterOwned {
    pub state: Lido,
    pub validator_list: LidoValidatorListOwned,
    pub curr_epoch: u64,
}

impl LidoRouterOwned {
    /// Lido only allows withdrawing from max stake validator
    pub fn to_withdraw_stake_router<'a>(
        &'a self,
        vote: Option<&'a [u8; 32]>,
    ) -> Option<LidoWithdrawStakeRouter<'a>> {
        let max_validator = self
            .validator_list
            .validators
            .iter()
            .max_by_key(|v| v.effective_stake_balance())?;

        let vote_account = vote.map_or_else(
            || Some(max_validator.vote_account_address()),
            |v| (v == max_validator.vote_account_address()).then_some(v),
        )?;

        Some(LidoWithdrawStakeRouter {
            state: &self.state,
            voter: vote_account,
            stake_to_split: find_lido_validator_stake_account_pda_internal(
                max_validator.vote_account_address(),
                max_validator.stake_seeds().begin(),
            )?
            .0,
            curr_epoch: self.curr_epoch,
            validator_effective_stake_balance: max_validator.effective_stake_balance(),
        })
    }

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

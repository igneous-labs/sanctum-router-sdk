use sanctum_router_core::{LidoWithdrawStakeQuoter, LidoWithdrawStakeSufAccs};
use solido_legacy_core::{
    Lido, ListHeader, Validator, ValidatorList, STSOL_MINT_ADDR, SYSVAR_CLOCK,
};
use wasm_bindgen::JsError;

use crate::{
    err::unsupported_update,
    interface::{get_account_data, AccountMap},
    pda::lido::find_lido_validator_stake_account_pda_internal,
    update::{PoolUpdateType, Update},
};

#[derive(Clone, Debug, Default, PartialEq)]
pub struct LidoRouterOwned {
    pub state: Lido,
    pub validator_list: LidoValidatorListOwned,
}

/// Init
impl LidoRouterOwned {
    pub const fn init_accounts() -> [[u8; 32]; 2] {
        [
            solido_legacy_core::LIDO_STATE_ADDR,
            solido_legacy_core::VALIDATOR_LIST_ADDR,
        ]
    }

    pub fn init(accounts: &AccountMap) -> Result<Self, JsError> {
        let [s, v] = Self::init_accounts().map(|k| get_account_data(accounts, k));
        let state_data = s?;
        let validator_list_data = v?;

        let state = Lido::borsh_de(state_data)?;
        let ValidatorList { header, entries } = ValidatorList::deserialize(validator_list_data)?;
        let validator_list = LidoValidatorListOwned {
            header,
            validators: entries.to_vec(),
        };

        Ok(Self {
            state,
            validator_list,
        })
    }
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

impl Update for LidoRouterOwned {
    fn accounts_to_update(&self, ty: PoolUpdateType) -> impl Iterator<Item = [u8; 32]> {
        match ty {
            PoolUpdateType::WithdrawStake => [
                solido_legacy_core::LIDO_STATE_ADDR,
                solido_legacy_core::VALIDATOR_LIST_ADDR,
                SYSVAR_CLOCK,
            ]
            .map(Some),
            _ => [None; 3],
        }
        .into_iter()
        .flatten()
    }

    fn update(&mut self, ty: PoolUpdateType, accounts: &AccountMap) -> Result<(), JsError> {
        match ty {
            PoolUpdateType::WithdrawStake => {
                *self = Self::init(accounts)?;
                Ok(())
            }
            _ => Err(unsupported_update(ty, &STSOL_MINT_ADDR)),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Default)]
pub struct LidoValidatorListOwned {
    pub header: ListHeader,
    pub validators: Vec<Validator>,
}

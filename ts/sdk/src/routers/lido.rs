use sanctum_router_core::{LidoWithdrawStakeQuoter, LidoWithdrawStakeSufAccs};
use solido_legacy_core::{
    Lido, ListHeader, Validator, ValidatorList, STSOL_MINT_ADDR, SYSVAR_CLOCK,
};
use wasm_bindgen::JsError;

use crate::{
    err::{account_missing_err, invalid_data_err, invalid_pda_err, unsupported_update},
    interface::{get_account_data, AccountMap},
    pda::lido::find_lido_validator_stake_account_pda_internal,
    update::PoolUpdateType,
};

#[derive(Clone, Debug, Default, PartialEq)]
pub struct LidoRouterOwned(pub Option<LidoRouterInner>);

#[derive(Clone, Debug, PartialEq)]
pub struct LidoRouterInner {
    pub state: Lido,
    pub validator_list: LidoValidatorListOwned,
}

#[derive(Clone, Debug, PartialEq)]
pub struct LidoValidatorListOwned {
    pub header: ListHeader,
    pub validators: Vec<Validator>,
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

        Ok(Self(Some(LidoRouterInner {
            state,
            validator_list,
        })))
    }
}

/// Getters
impl LidoRouterOwned {
    pub fn try_inner(&self) -> Result<&LidoRouterInner, JsError> {
        self.0
            .as_ref()
            .ok_or_else(|| account_missing_err(&solido_legacy_core::LIDO_STATE_ADDR))
    }
}

/// WithdrawStake
impl LidoRouterOwned {
    /// Lido only allows withdrawing from max stake validator
    pub fn withdraw_stake_quoter(
        &self,
        curr_epoch: u64,
    ) -> Result<LidoWithdrawStakeQuoter, JsError> {
        let inner = self.try_inner()?;
        LidoWithdrawStakeQuoter::new(&inner.state, &inner.validator_list.validators, curr_epoch)
            .ok_or_else(invalid_data_err)
    }

    /// Lido only allows withdrawing from max stake validator
    ///
    /// Returns `None` if data missing or validator stake acc PDA invalid
    pub fn withdraw_stake_suf_accs(&self) -> Result<LidoWithdrawStakeSufAccs, JsError> {
        let inner = self.try_inner()?;
        let max_validator = inner
            .validator_list
            .validators
            .iter()
            .max_by_key(|v| v.effective_stake_balance())
            .ok_or_else(invalid_data_err)?;
        let largest_stake_vote = max_validator.vote_account_address();
        Ok(LidoWithdrawStakeSufAccs {
            validator_list_addr: &inner.state.validator_list,
            largest_stake_vote,
            stake_to_split: find_lido_validator_stake_account_pda_internal(
                largest_stake_vote,
                max_validator.stake_seeds().begin(),
            )
            .ok_or_else(invalid_pda_err)?
            .0,
        })
    }
}

/// Update
impl LidoRouterOwned {
    pub fn accounts_to_update(ty: PoolUpdateType) -> impl Iterator<Item = [u8; 32]> {
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

    pub fn update(&mut self, ty: PoolUpdateType, accounts: &AccountMap) -> Result<(), JsError> {
        match ty {
            PoolUpdateType::WithdrawStake => {
                *self = Self::init(accounts)?;
                Ok(())
            }
            _ => Err(unsupported_update(ty, &STSOL_MINT_ADDR)),
        }
    }
}

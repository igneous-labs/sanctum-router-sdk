use sanctum_router_std::{
    solido_legacy_core::{self, Lido, ValidatorList, STSOL_MINT_ADDR, SYSVAR_CLOCK},
    LidoWithdrawStakeQuoter, LidoWithdrawStakeSufAccs,
};

use crate::{
    err::{account_missing_err, invalid_data_err, unsupported_update_err, SanctumRouterError},
    interface::{get_account_data, AccountMap},
    pda::find_pda,
    update::PoolUpdateType,
};

pub type LidoRouter =
    sanctum_router_std::LidoRouter<fn(&[&[u8]], &[u8; 32]) -> Option<([u8; 32], u8)>>;

/// Notes
/// - `curr_epoch` field in this struct is not used, but patched with the shared one in
///   [`crate::router::SanctumRouter`] at quoting time
#[derive(Clone, Debug, Default, PartialEq)]
pub struct LidoRouterOwned(pub Option<LidoRouter>);

/// Init
impl LidoRouterOwned {
    pub const fn init_accounts() -> [[u8; 32]; 2] {
        [
            solido_legacy_core::LIDO_STATE_ADDR,
            solido_legacy_core::VALIDATOR_LIST_ADDR,
        ]
    }

    pub fn init(accounts: &AccountMap) -> Result<Self, SanctumRouterError> {
        let [s, v] = Self::init_accounts().map(|k| get_account_data(accounts, k));
        let state_data = s?;
        let validator_list_data = v?;

        let state = Lido::borsh_de(state_data).map_err(|_e| invalid_data_err())?;
        let ValidatorList { entries, .. } =
            ValidatorList::deserialize(validator_list_data).map_err(|_e| invalid_data_err())?;

        Ok(Self(Some(LidoRouter::new(&state, entries, 0, find_pda))))
    }
}

/// Getters
impl LidoRouterOwned {
    pub fn try_inner(&self) -> Result<&LidoRouter, SanctumRouterError> {
        self.0
            .as_ref()
            .ok_or_else(|| account_missing_err(&solido_legacy_core::LIDO_STATE_ADDR))
    }
}

/// WithdrawStake
impl LidoRouterOwned {
    pub fn withdraw_stake_quoter(
        &self,
        curr_epoch: u64,
    ) -> Result<LidoWithdrawStakeQuoter<'_>, SanctumRouterError> {
        let quoter = self.try_inner()?.lido_withdraw_stake_quoter();
        Ok(LidoWithdrawStakeQuoter {
            curr_epoch,
            ..quoter
        })
    }

    pub fn withdraw_stake_suf_accs(
        &self,
    ) -> Result<LidoWithdrawStakeSufAccs<'_>, SanctumRouterError> {
        self.try_inner().and_then(|inner| {
            inner
                .lido_withdraw_stake_suf_accs()
                .ok_or_else(invalid_data_err)
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

    pub fn update(
        &mut self,
        ty: PoolUpdateType,
        accounts: &AccountMap,
    ) -> Result<(), SanctumRouterError> {
        match ty {
            PoolUpdateType::WithdrawStake => {
                *self = Self::init(accounts)?;
                Ok(())
            }
            _ => Err(unsupported_update_err(ty, &STSOL_MINT_ADDR)),
        }
    }
}

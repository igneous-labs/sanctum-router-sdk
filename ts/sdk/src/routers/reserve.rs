use sanctum_router_std::{
    sanctum_reserve_core::{self, Fee, FeeEnum, Pool, PoolUnstakeParams, ProtocolFee},
    ReserveDepositStakeQuoter, ReserveDepositStakeSufAccs, NATIVE_MINT,
};

use crate::{
    err::{
        account_missing_err, invalid_data_err, invalid_pda_err, unsupported_update_err,
        SanctumRouterError,
    },
    interface::{get_account, get_account_data, AccountMap},
    pda::find_pda,
    update::PoolUpdateType,
};

pub type ReserveRouter =
    sanctum_router_std::ReserveRouter<fn(&[&[u8]], &[u8; 32]) -> Option<([u8; 32], u8)>>;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct ReserveRouterOwned(pub Option<ReserveRouter>);

/// Init
impl ReserveRouterOwned {
    pub const fn init_accounts() -> [[u8; 32]; 4] {
        [
            sanctum_reserve_core::POOL,
            sanctum_reserve_core::FEE,
            sanctum_reserve_core::PROTOCOL_FEE,
            sanctum_reserve_core::POOL_SOL_RESERVES,
        ]
    }

    pub fn init(accounts: &AccountMap) -> Result<Self, SanctumRouterError> {
        let [p, f, pf] = [
            sanctum_reserve_core::POOL,
            sanctum_reserve_core::FEE,
            sanctum_reserve_core::PROTOCOL_FEE,
        ]
        .map(|pk| get_account_data(accounts, pk));
        let pool_data = p?;
        let fee_data = f?;
        let protocol_fee_data = pf?;

        let pool = Pool::anchor_de(pool_data).map_err(|_e| invalid_data_err())?;
        let fee = Fee::anchor_de(fee_data).map_err(|_e| invalid_data_err())?;
        let protocol_fee =
            ProtocolFee::anchor_de(protocol_fee_data).map_err(|_e| invalid_data_err())?;
        let pool_sol_reserves =
            get_account(accounts, sanctum_reserve_core::POOL_SOL_RESERVES)?.lamports;

        Ok(Self(Some(ReserveRouter::new(
            fee,
            &protocol_fee,
            &pool,
            pool_sol_reserves,
            find_pda,
        ))))
    }
}

/// Getters
impl ReserveRouterOwned {
    pub fn try_inner(&self) -> Result<&ReserveRouter, SanctumRouterError> {
        self.0
            .as_ref()
            .ok_or_else(|| account_missing_err(&sanctum_reserve_core::POOL))
    }
}

/// DepositStake
impl ReserveRouterOwned {
    pub fn deposit_stake_quoter(
        &self,
    ) -> Result<ReserveDepositStakeQuoter<'_>, SanctumRouterError> {
        self.try_inner()
            .map(|inner| inner.reserve_deposit_stake_quoter())
    }

    /// Returns `None` if stake acc record PDA invalid
    pub fn deposit_stake_suf_accs(
        &self,
        stake_account_addr: &[u8; 32],
    ) -> Result<ReserveDepositStakeSufAccs, SanctumRouterError> {
        self.try_inner().and_then(|inner| {
            inner
                .reserve_deposit_stake_suf_accs(stake_account_addr)
                .ok_or_else(invalid_pda_err)
        })
    }
}

/// Prefund
impl ReserveRouterOwned {
    pub fn prefund_params(&self) -> Result<(PoolUnstakeParams, &FeeEnum), SanctumRouterError> {
        let inner = self.try_inner()?;
        Ok((
            PoolUnstakeParams {
                pool_incoming_stake: inner.pool_incoming_stake,
                sol_reserves_lamports: inner.pool_sol_reserves,
            },
            &inner.fee_account,
        ))
    }
}

/// Update
impl ReserveRouterOwned {
    pub fn accounts_to_update(ty: PoolUpdateType) -> impl Iterator<Item = [u8; 32]> {
        match ty {
            PoolUpdateType::DepositStake => Self::init_accounts().map(Some),
            _ => [None; 4],
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
            PoolUpdateType::DepositStake => {
                *self = Self::init(accounts)?;
                Ok(())
            }
            _ => Err(unsupported_update_err(ty, &NATIVE_MINT)),
        }
    }
}

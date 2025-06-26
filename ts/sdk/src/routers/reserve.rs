use sanctum_reserve_core::{Fee, FeeEnum, Pool, PoolBalance, ProtocolFee};
use sanctum_router_core::{ReserveDepositStakeQuoter, ReserveDepositStakeSufAccs, NATIVE_MINT};

use crate::{
    err::{
        account_missing_err, invalid_data_err, invalid_pda_err, unsupported_update_err,
        SanctumRouterError,
    },
    interface::{get_account, get_account_data, AccountMap},
    pda::reserve::find_reserve_stake_account_record_pda_internal,
    update::PoolUpdateType,
};

#[derive(Clone, Debug, Default, PartialEq)]
pub struct ReserveRouterOwned(pub Option<ReserveRouterInner>);

#[derive(Clone, Debug, PartialEq)]
pub struct ReserveRouterInner {
    pub pool: Pool,
    pub fee_account: Fee,
    pub protocol_fee_account: ProtocolFee,
    pub pool_sol_reserves: u64,
}

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
        let fee_account = Fee::anchor_de(fee_data).map_err(|_e| invalid_data_err())?;
        let protocol_fee_account =
            ProtocolFee::anchor_de(protocol_fee_data).map_err(|_e| invalid_data_err())?;
        let pool_sol_reserves =
            get_account(accounts, sanctum_reserve_core::POOL_SOL_RESERVES)?.lamports;

        Ok(Self(Some(ReserveRouterInner {
            pool,
            fee_account,
            protocol_fee_account,
            pool_sol_reserves,
        })))
    }
}

/// Getters
impl ReserveRouterOwned {
    pub fn try_inner(&self) -> Result<&ReserveRouterInner, SanctumRouterError> {
        self.0
            .as_ref()
            .ok_or_else(|| account_missing_err(&sanctum_reserve_core::POOL))
    }
}

/// DepositStake
impl ReserveRouterOwned {
    pub fn deposit_stake_quoter(&self) -> Result<ReserveDepositStakeQuoter, SanctumRouterError> {
        let inner = self.try_inner()?;
        Ok(ReserveDepositStakeQuoter {
            pool_incoming_stake: inner.pool.incoming_stake,
            fee_account: &inner.fee_account.0,
            protocol_fee_account: &inner.protocol_fee_account,
            pool_sol_reserves: inner.pool_sol_reserves,
        })
    }

    /// Returns `None` if stake acc record PDA invalid
    pub fn deposit_stake_suf_accs(
        &self,
        stake_account_addr: &[u8; 32],
    ) -> Result<ReserveDepositStakeSufAccs, SanctumRouterError> {
        Ok(ReserveDepositStakeSufAccs {
            stake_acc_record_addr: find_reserve_stake_account_record_pda_internal(
                stake_account_addr,
            )
            .ok_or_else(invalid_pda_err)?
            .0,
        })
    }
}

/// Prefund
impl ReserveRouterOwned {
    pub fn prefund_params(&self) -> Result<(PoolBalance, &FeeEnum), SanctumRouterError> {
        let inner = self.try_inner()?;
        Ok((
            PoolBalance {
                pool_incoming_stake: inner.pool.incoming_stake,
                sol_reserves_lamports: inner.pool_sol_reserves,
            },
            &inner.fee_account.0,
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

use sanctum_reserve_core::{Fee, FeeEnum, Pool, PoolBalance, ProtocolFee};
use sanctum_router_core::{ReserveDepositStakeQuoter, ReserveDepositStakeSufAccs, NATIVE_MINT};
use wasm_bindgen::JsError;

use crate::{
    err::unsupported_update,
    interface::{get_account, get_account_data, AccountMap},
    pda::reserve::find_reserve_stake_account_record_pda_internal,
    update::{PoolUpdateType, Update},
};

#[derive(Clone, Debug, PartialEq)]
pub struct ReserveRouterOwned {
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

    pub fn init(accounts: &AccountMap) -> Result<Self, JsError> {
        let [p, f, pf] = [
            sanctum_reserve_core::POOL,
            sanctum_reserve_core::FEE,
            sanctum_reserve_core::PROTOCOL_FEE,
        ]
        .map(|pk| get_account_data(accounts, pk));
        let pool_data = p?;
        let fee_data = f?;
        let protocol_fee_data = pf?;

        let pool = Pool::anchor_de(pool_data)?;
        let fee_account = Fee::anchor_de(fee_data)?;
        let protocol_fee_account = ProtocolFee::anchor_de(protocol_fee_data)?;
        let pool_sol_reserves =
            get_account(accounts, sanctum_reserve_core::POOL_SOL_RESERVES)?.lamports;

        Ok(Self {
            pool,
            fee_account,
            protocol_fee_account,
            pool_sol_reserves,
        })
    }
}

/// DepositStake
impl ReserveRouterOwned {
    pub fn deposit_stake_quoter(&self) -> ReserveDepositStakeQuoter {
        ReserveDepositStakeQuoter {
            pool_incoming_stake: self.pool.incoming_stake,
            fee_account: &self.fee_account.0,
            protocol_fee_account: &self.protocol_fee_account,
            pool_sol_reserves: self.pool_sol_reserves,
        }
    }

    /// Returns `None` if stake acc record PDA invalid
    pub fn deposit_stake_suf_accs(
        &self,
        stake_account_addr: &[u8; 32],
    ) -> Option<ReserveDepositStakeSufAccs> {
        Some(ReserveDepositStakeSufAccs {
            stake_acc_record_addr: find_reserve_stake_account_record_pda_internal(
                stake_account_addr,
            )?
            .0,
        })
    }
}

/// Prefund
impl ReserveRouterOwned {
    pub const fn prefund_params(&self) -> (PoolBalance, &FeeEnum) {
        (
            PoolBalance {
                pool_incoming_stake: self.pool.incoming_stake,
                sol_reserves_lamports: self.pool_sol_reserves,
            },
            &self.fee_account.0,
        )
    }
}

impl Update for ReserveRouterOwned {
    fn accounts_to_update(&self, ty: PoolUpdateType) -> impl Iterator<Item = [u8; 32]> {
        match ty {
            PoolUpdateType::DepositStake => Self::init_accounts().map(Some),
            _ => [None; 4],
        }
        .into_iter()
        .flatten()
    }

    fn update(&mut self, ty: PoolUpdateType, accounts: &AccountMap) -> Result<(), JsError> {
        match ty {
            PoolUpdateType::DepositStake => {
                *self = Self::init(accounts)?;
                Ok(())
            }
            _ => Err(unsupported_update(ty, &NATIVE_MINT)),
        }
    }
}

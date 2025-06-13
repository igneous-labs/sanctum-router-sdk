use sanctum_reserve_core::{Fee, FeeEnum, Pool, PoolBalance, ProtocolFee};
use sanctum_router_core::{ReserveDepositStakeQuoter, ReserveDepositStakeSufAccs};
use wasm_bindgen::JsError;

use crate::{
    interface::{get_account, get_account_data, AccountMap},
    pda::reserve::find_reserve_stake_account_record_pda_internal,
    update::Update,
};

#[derive(Clone, Debug, PartialEq)]
pub struct ReserveRouterOwned {
    pub pool: Pool,
    pub fee_account: Fee,
    pub protocol_fee_account: ProtocolFee,
    pub pool_sol_reserves: u64,
}

/// DepositStake
impl ReserveRouterOwned {
    pub fn deposit_stake_quoter(&self) -> ReserveDepositStakeQuoter {
        ReserveDepositStakeQuoter {
            pool: &self.pool,
            fee_account: &self.fee_account,
            protocol_fee_account: &self.protocol_fee_account,
            pool_sol_reserves: self.pool_sol_reserves,
        }
    }

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

/// Update helpers
impl ReserveRouterOwned {
    pub fn update_pool(&mut self, pool_data: &[u8]) -> Result<(), JsError> {
        self.pool = Pool::anchor_de(pool_data)?;
        Ok(())
    }

    pub fn update_fee(&mut self, fee_account_data: &[u8]) -> Result<(), JsError> {
        self.fee_account = Fee::anchor_de(fee_account_data)?;
        Ok(())
    }

    pub fn update_protocol_fee(&mut self, protocol_fee_account_data: &[u8]) -> Result<(), JsError> {
        self.protocol_fee_account = ProtocolFee::anchor_de(protocol_fee_account_data)?;
        Ok(())
    }

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
    fn get_accounts_to_update(&self) -> impl Iterator<Item = [u8; 32]> {
        [
            sanctum_reserve_core::POOL,
            sanctum_reserve_core::FEE,
            sanctum_reserve_core::PROTOCOL_FEE,
            sanctum_reserve_core::POOL_SOL_RESERVES,
        ]
        .into_iter()
    }

    fn update(&mut self, accounts: &AccountMap) -> Result<(), JsError> {
        let [Ok(pool_data), Ok(fee_data), Ok(protocol_fee_data)] = [
            sanctum_reserve_core::POOL,
            sanctum_reserve_core::FEE,
            sanctum_reserve_core::PROTOCOL_FEE,
        ]
        .map(|pk| get_account_data(accounts, pk)) else {
            return Err(JsError::new("Failed to fetch reserve accounts"));
        };

        self.update_pool(pool_data)?;
        self.update_fee(fee_data)?;
        self.update_protocol_fee(protocol_fee_data)?;

        self.pool_sol_reserves =
            get_account(accounts, sanctum_reserve_core::POOL_SOL_RESERVES)?.lamports;

        Ok(())
    }
}

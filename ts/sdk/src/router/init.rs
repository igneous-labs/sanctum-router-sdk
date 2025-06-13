use sanctum_marinade_liquid_staking_core::State as MarinadeState;
use sanctum_reserve_core::{Fee, Pool, ProtocolFee};
use sanctum_spl_stake_pool_core::StakePool;
use serde::{Deserialize, Serialize};
use solido_legacy_core::Lido;
use tsify_next::Tsify;
use wasm_bindgen::prelude::*;

use crate::{
    err::{account_missing_err, invalid_pda_err},
    interface::{get_account_data, AccountMap, B58PK},
    pda::spl::{find_deposit_auth_pda_internal, find_withdraw_auth_pda_internal},
    router::{SanctumRouter, SanctumRouterHandle},
    routers::{
        LidoRouterOwned, LidoValidatorListOwned, MarinadeRouterOwned, ReserveRouterOwned,
        SplStakePoolRouterOwned,
    },
};

#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub struct SplPoolAccounts {
    pub pool: B58PK,
    pub validator_list: B58PK,
}

/// Returns the accounts that need to be fetched to initialize the router.
#[wasm_bindgen(js_name = getInitAccounts)]
pub fn get_init_accounts(spl_lsts: Vec<SplPoolAccounts>) -> Box<[B58PK]> {
    spl_lsts
        .iter()
        .flat_map(|accounts| [accounts.pool, accounts.validator_list])
        .chain([
            B58PK::new(solido_legacy_core::LIDO_STATE_ADDR),
            B58PK::new(solido_legacy_core::VALIDATOR_LIST_ADDR),
        ])
        .chain([
            B58PK::new(sanctum_reserve_core::POOL),
            B58PK::new(sanctum_reserve_core::FEE),
            B58PK::new(sanctum_reserve_core::PROTOCOL_FEE),
        ])
        .chain([
            B58PK::new(sanctum_marinade_liquid_staking_core::STATE_PUBKEY),
            B58PK::new(sanctum_marinade_liquid_staking_core::VALIDATOR_LIST_PUBKEY),
        ])
        .collect()
}

/// Creates a new router from the fetched init accounts.
#[wasm_bindgen(js_name = fromFetchedAccounts)]
pub fn from_fetched_accounts(
    spl_lsts: Vec<SplPoolAccounts>,
    accounts: AccountMap,
    curr_epoch: u64,
) -> Result<SanctumRouterHandle, JsError> {
    // Lido
    let [Ok(state_data), Ok(validator_list_data)] = [
        solido_legacy_core::LIDO_STATE_ADDR,
        solido_legacy_core::VALIDATOR_LIST_ADDR,
    ]
    .map(|k| get_account_data(&accounts, k)) else {
        return Err(JsError::new("Failed to fetch lido accounts"));
    };

    let mut lido_router = LidoRouterOwned {
        state: Lido::borsh_de(state_data)?,
        validator_list: LidoValidatorListOwned::default(),
        curr_epoch,
    };

    lido_router.update_validator_list(validator_list_data)?;

    // Marinade
    let [Ok(state_data), Ok(validator_records_data)] = [
        sanctum_marinade_liquid_staking_core::STATE_PUBKEY,
        sanctum_marinade_liquid_staking_core::VALIDATOR_LIST_PUBKEY,
    ]
    .map(|pk| get_account_data(&accounts, pk)) else {
        return Err(JsError::new("Failed to fetch marinade accounts"));
    };

    let mut marinade_router = MarinadeRouterOwned {
        state: MarinadeState::borsh_de(state_data)?,
        validator_records: vec![],
        // Set on `update()`
        msol_leg_balance: 0,
    };

    marinade_router.update_validator_records(
        validator_records_data,
        marinade_router.state.validator_system.validator_list.len() as usize,
    )?;

    // Reserve
    let [Ok(pool_data), Ok(fee_data), Ok(protocol_fee_data)] = [
        sanctum_reserve_core::POOL,
        sanctum_reserve_core::FEE,
        sanctum_reserve_core::PROTOCOL_FEE,
    ]
    .map(|pk| get_account_data(&accounts, pk)) else {
        return Err(JsError::new("Failed to fetch reserve accounts"));
    };

    let reserve_router = ReserveRouterOwned {
        pool: Pool::anchor_de(pool_data)?,
        fee_account: Fee::anchor_de(fee_data)?,
        protocol_fee_account: ProtocolFee::anchor_de(protocol_fee_data)?,
        // Set on `update()`
        pool_sol_reserves: 0,
    };

    // SPL
    let spl_routers = spl_lsts
        .iter()
        .map(|lst| {
            let pool_account = accounts
                .0
                .get(&lst.pool)
                .ok_or(account_missing_err(&lst.pool.0))?;
            let stake_pool_addr = lst.pool.0;
            let program_addr = pool_account.owner.0;
            let pool_data = StakePool::borsh_de(&*pool_account.data)?;

            let mut router = SplStakePoolRouterOwned {
                stake_pool_addr,
                stake_pool_program: program_addr,
                stake_pool: pool_data,
                deposit_authority_program_address: find_deposit_auth_pda_internal(
                    &program_addr,
                    &stake_pool_addr,
                )
                .ok_or_else(invalid_pda_err)?
                .0,
                withdraw_authority_program_address: find_withdraw_auth_pda_internal(
                    &program_addr,
                    &stake_pool_addr,
                )
                .ok_or_else(invalid_pda_err)?
                .0,
                curr_epoch,
                ..Default::default()
            };

            let validator_list_data = get_account_data(&accounts, lst.validator_list.0)?;

            router.update_validator_list(validator_list_data)?;
            Ok(router)
        })
        .collect::<Result<Vec<_>, JsError>>()?;

    Ok(SanctumRouterHandle(SanctumRouter {
        spl_routers,
        lido_router,
        marinade_router,
        reserve_router,
    }))
}

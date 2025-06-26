use std::collections::HashSet;

use sanctum_router_core::SYSVAR_CLOCK;
use wasm_bindgen::prelude::*;

use crate::{
    err::router_missing_err,
    interface::{get_account_data, AccountMap, B58PK},
    router::{clock::try_clock_acc_data_epoch, SanctumRouterHandle},
    routers::{LidoRouterOwned, MarinadeRouterOwned, ReserveRouterOwned},
    update::{PoolUpdate, SwapMints},
};

/// Returns the accounts needed to update specific pools for given swap routes.
///
/// Dedups returned pubkey list; all pubkeys in returned list guaranteed to be unique.
#[wasm_bindgen(js_name = accountsToUpdate)]
pub fn accounts_to_update(
    this: &SanctumRouterHandle,
    // Clippy complains, needed for wasm_bindgen
    #[allow(clippy::boxed_local)] swap_mints: Box<[SwapMints]>,
) -> Result<Box<[B58PK]>, JsError> {
    // collect into HashSet to dedup PoolUpdates
    let pool_updates: HashSet<PoolUpdate> = swap_mints
        .iter()
        .flat_map(|sm| sm.into_pool_updates())
        .collect();

    let mut accounts = Vec::new();

    for PoolUpdate { mint, ty } in pool_updates.into_iter() {
        match mint {
            sanctum_router_core::NATIVE_MINT => {
                accounts.extend(ReserveRouterOwned::accounts_to_update(ty).map(B58PK::new));
            }
            sanctum_marinade_liquid_staking_core::MSOL_MINT_ADDR => {
                accounts.extend(MarinadeRouterOwned::accounts_to_update(ty).map(B58PK::new));
            }
            solido_legacy_core::STSOL_MINT_ADDR => {
                accounts.extend(LidoRouterOwned::accounts_to_update(ty).map(B58PK::new));
            }
            mint => accounts.extend(
                this.0
                    .try_find_spl_by_mint(&mint)?
                    .accounts_to_update(ty)
                    .map(B58PK::new),
            ),
        }
    }

    accounts.sort();
    accounts.dedup();
    Ok(accounts.into_boxed_slice())
}

/// Updates specific pools for given swap routes
#[wasm_bindgen(js_name = update)]
pub fn update(
    this: &mut SanctumRouterHandle,
    // Clippy complains, needed for wasm_bindgen
    #[allow(clippy::boxed_local)] swap_mints: Box<[SwapMints]>,
    accounts: &AccountMap,
) -> Result<(), JsError> {
    // collect into HashSet to dedup PoolUpdates
    let pool_updates: HashSet<PoolUpdate> = swap_mints
        .iter()
        .flat_map(|sm| sm.into_pool_updates())
        .collect();

    // Use this state flag instead of just doing
    // update if clock found in AccountMap
    // because we want to fail if clock is supposed to be updated
    // but wasn't fetched
    let mut require_clock_update = false;

    for PoolUpdate { mint, ty } in pool_updates.into_iter() {
        match mint {
            sanctum_router_core::NATIVE_MINT => {
                this.0.reserve_router.update(ty, accounts)?;
            }
            sanctum_marinade_liquid_staking_core::MSOL_MINT_ADDR => {
                this.0.marinade_router.update(ty, accounts)?;
            }
            solido_legacy_core::STSOL_MINT_ADDR => {
                this.0.lido_router.update(ty, accounts)?;
                require_clock_update = true;
            }
            mint => {
                this.0
                    .spl_routers
                    .get_mut(&mint)
                    .ok_or_else(|| router_missing_err(&mint))?
                    .update(ty, accounts)?;
                require_clock_update = true;
            }
        }
    }

    if require_clock_update {
        let curr_epoch =
            get_account_data(accounts, SYSVAR_CLOCK).and_then(try_clock_acc_data_epoch)?;
        this.0.curr_epoch = Some(curr_epoch);
    }

    Ok(())
}

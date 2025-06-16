use sanctum_router_core::SYSVAR_CLOCK;
use wasm_bindgen::prelude::*;

use crate::{
    err::router_missing_err,
    interface::{get_account_data, AccountMap, B58PK},
    router::{clock::try_clock_acc_data_epoch, SanctumRouterHandle},
    update::Update,
};

/// Returns the accounts needed to update a specific routers according to the mint addresses.
///
/// Dedups returned pubkey list; all pubkeys in returned list guaranteed to be unique.
#[wasm_bindgen(js_name = getAccountsToUpdate)]
pub fn get_accounts_to_update(
    this: &SanctumRouterHandle,
    // Clippy complains, needed for wasm_bindgen
    #[allow(clippy::boxed_local)] mints: Box<[B58PK]>,
) -> Result<Box<[B58PK]>, JsError> {
    let mut accounts = Vec::new();

    for mint in mints.iter() {
        match mint.0 {
            sanctum_router_core::NATIVE_MINT => {
                accounts.extend(
                    this.0
                        .reserve_router
                        .get_accounts_to_update()
                        .map(B58PK::new),
                );
            }
            sanctum_marinade_liquid_staking_core::MSOL_MINT_ADDR => {
                accounts.extend(
                    this.0
                        .marinade_router
                        .get_accounts_to_update()
                        .map(B58PK::new),
                );
            }
            solido_legacy_core::STSOL_MINT_ADDR => {
                accounts.extend(this.0.lido_router.get_accounts_to_update().map(B58PK::new));
            }
            mint => accounts.extend(
                this.0
                    .try_find_spl_by_mint(&mint)?
                    .get_accounts_to_update()
                    .map(B58PK::new),
            ),
        }
    }

    accounts.sort();
    accounts.dedup();
    Ok(accounts.into_boxed_slice())
}

/// Updates a specific routers according to the mint addresses.
#[wasm_bindgen(js_name = update)]
pub fn update(
    this: &mut SanctumRouterHandle,
    // Clippy complains, needed for wasm_bindgen
    #[allow(clippy::boxed_local)] mints: Box<[B58PK]>,
    accounts: &AccountMap,
) -> Result<(), JsError> {
    // Use this state flag instead of just doing
    // update if clock found in AccountMap
    // because we want to fail if clock is supposed to be updated
    // but wasn't fetched
    let mut require_clock_update = false;

    for mint in mints.iter() {
        match mint.0 {
            sanctum_router_core::NATIVE_MINT => {
                this.0.reserve_router.update(accounts)?;
            }
            sanctum_marinade_liquid_staking_core::MSOL_MINT_ADDR => {
                this.0.marinade_router.update(accounts)?;
            }
            solido_legacy_core::STSOL_MINT_ADDR => {
                this.0.lido_router.update(accounts)?;
                require_clock_update = true;
            }
            mint => {
                this.0
                    .spl_routers
                    .iter_mut()
                    .find(|r| r.stake_pool.pool_mint == mint)
                    .ok_or_else(router_missing_err)?
                    .update(accounts)?;
                require_clock_update = true;
            }
        }
    }

    if require_clock_update {
        let curr_epoch =
            get_account_data(accounts, SYSVAR_CLOCK).and_then(try_clock_acc_data_epoch)?;
        this.0.curr_epoch = curr_epoch;
    }

    Ok(())
}

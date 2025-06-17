use std::iter::once;

use sanctum_router_core::SYSVAR_CLOCK;
use wasm_bindgen::prelude::*;

use crate::{
    interface::{get_account_data, AccountMap, B58PK},
    router::{clock::try_clock_acc_data_epoch, SanctumRouter, SanctumRouterHandle},
    routers::{
        LidoRouterOwned, MarinadeRouterOwned, ReserveRouterOwned, SplPoolAccounts,
        SplStakePoolRouterOwned,
    },
};

/// Returns the accounts that need to be fetched to initialize the router.
#[wasm_bindgen(js_name = initAccounts)]
pub fn init_accounts(
    // Clippy complains, needed for wasm_bindgen
    #[allow(clippy::boxed_local)] spl_lsts: Box<[SplPoolAccounts]>,
) -> Box<[B58PK]> {
    spl_lsts
        .iter()
        .flat_map(|accounts| [accounts.pool, accounts.validator_list])
        .chain(LidoRouterOwned::init_accounts().map(B58PK::new))
        .chain(ReserveRouterOwned::init_accounts().map(B58PK::new))
        .chain(MarinadeRouterOwned::init_accounts().map(B58PK::new))
        .chain(once(B58PK::new(SYSVAR_CLOCK))) // for current epoch
        .collect()
}

/// Creates a new router from the fetched init accounts.
#[wasm_bindgen(js_name = init)]
pub fn init(
    // Clippy complains, needed for wasm_bindgen
    #[allow(clippy::boxed_local)] spl_lsts: Box<[SplPoolAccounts]>,
    init_accounts: &AccountMap,
) -> Result<SanctumRouterHandle, JsError> {
    let curr_epoch =
        get_account_data(init_accounts, SYSVAR_CLOCK).and_then(try_clock_acc_data_epoch)?;
    let lido_router = LidoRouterOwned::init(init_accounts)?;
    let marinade_router = MarinadeRouterOwned::init(init_accounts)?;
    let reserve_router = ReserveRouterOwned::init(init_accounts)?;
    let spl_routers = spl_lsts
        .iter()
        .map(|lst| SplStakePoolRouterOwned::init(lst, init_accounts))
        .collect::<Result<Vec<_>, JsError>>()?;

    Ok(SanctumRouterHandle(SanctumRouter {
        curr_epoch,
        spl_routers,
        lido_router,
        marinade_router,
        reserve_router,
    }))
}

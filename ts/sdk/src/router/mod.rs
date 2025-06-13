// since this should be the top-level module with only #[wasm_bindgen] exports,
// all its modules can be private

use wasm_bindgen::prelude::*;

use crate::routers::{MarinadeRouterOwned, ReserveRouterOwned, SplStakePoolRouterOwned};

mod deposit_sol;
mod deposit_stake;
mod init;
mod token_quote;
mod update;
mod withdraw_sol;

/// The main top level router type that is an aggregation of all underlying stake pools
#[wasm_bindgen]
pub struct SanctumRouterHandle(pub(crate) SanctumRouter);

#[derive(Clone, Debug)]
pub struct SanctumRouter {
    pub spl_routers: Vec<SplStakePoolRouterOwned>,
    pub marinade_router: MarinadeRouterOwned,
    pub reserve_router: ReserveRouterOwned,
}

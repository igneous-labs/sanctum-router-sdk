// since this should be the top-level module with only #[wasm_bindgen] exports,
// all its modules can be private

use wasm_bindgen::prelude::*;

use crate::{
    err::router_missing_err,
    routers::{LidoRouterOwned, MarinadeRouterOwned, ReserveRouterOwned, SplStakePoolRouterOwned},
};

mod deposit_sol;
mod deposit_stake;
mod init;
mod swap_via_stake;
mod token_pair;
mod update;
mod withdraw_sol;
mod withdraw_stake;

/// The main top level router type that is an aggregation of all underlying stake pools
#[wasm_bindgen]
pub struct SanctumRouterHandle(pub(crate) SanctumRouter);

#[derive(Clone, Debug)]
pub struct SanctumRouter {
    pub spl_routers: Vec<SplStakePoolRouterOwned>,
    pub lido_router: LidoRouterOwned,
    pub marinade_router: MarinadeRouterOwned,
    pub reserve_router: ReserveRouterOwned,
}

impl SanctumRouter {
    pub(crate) fn find_spl_by_mint(&self, mint: &[u8; 32]) -> Option<&SplStakePoolRouterOwned> {
        self.spl_routers
            .iter()
            .find(|r| r.stake_pool.pool_mint == *mint)
    }

    pub(crate) fn try_find_spl_by_mint(
        &self,
        mint: &[u8; 32],
    ) -> Result<&SplStakePoolRouterOwned, JsError> {
        self.find_spl_by_mint(mint).ok_or_else(router_missing_err)
    }
}

// since this should be the top-level module with only #[wasm_bindgen] exports,
// all its modules can be private

use std::collections::HashMap;

use sanctum_router_core::SYSVAR_CLOCK;
use wasm_bindgen::prelude::*;

use crate::{
    err::{account_missing_err, router_missing_err, SanctumRouterError},
    routers::{LidoRouterOwned, MarinadeRouterOwned, ReserveRouterOwned, SplStakePoolRouterOwned},
};

mod clock;
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

#[derive(Clone, Debug, Default)]
pub struct SanctumRouter {
    pub lido_router: LidoRouterOwned,
    pub marinade_router: MarinadeRouterOwned,
    pub reserve_router: ReserveRouterOwned,

    /// Fetched from sysvar clock
    pub curr_epoch: Option<u64>,

    /// Key is LST mint
    pub spl_routers: HashMap<[u8; 32], SplStakePoolRouterOwned>,
}

impl SanctumRouter {
    pub(crate) fn find_spl_by_mint(&self, mint: &[u8; 32]) -> Option<&SplStakePoolRouterOwned> {
        self.spl_routers.get(mint)
    }

    pub(crate) fn try_find_spl_by_mint(
        &self,
        mint: &[u8; 32],
    ) -> Result<&SplStakePoolRouterOwned, SanctumRouterError> {
        self.find_spl_by_mint(mint)
            .ok_or_else(|| router_missing_err(mint))
    }

    pub(crate) fn try_curr_epoch(&self) -> Result<u64, SanctumRouterError> {
        self.curr_epoch
            .ok_or_else(|| account_missing_err(&SYSVAR_CLOCK))
    }
}

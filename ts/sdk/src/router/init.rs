use std::collections::hash_map::Entry;

use bs58_fixed_wasm::Bs58Array;
use sanctum_marinade_liquid_staking_core::MSOL_MINT_ADDR;
use sanctum_router_core::NATIVE_MINT;
use serde::{Deserialize, Serialize};
use solido_legacy_core::STSOL_MINT_ADDR;
use tsify_next::Tsify;
use wasm_bindgen::prelude::*;

use crate::{
    err::invalid_data_err,
    init::InitData,
    interface::B58PK,
    router::{SanctumRouter, SanctumRouterHandle},
    routers::SplStakePoolRouterOwned,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi, large_number_types_as_bigints)]
#[serde(rename_all = "camelCase")]
pub struct InitMint {
    pub mint: B58PK,

    /// Must be provided for SPL pools,
    /// omitted for everything else
    #[tsify(optional)]
    pub init: Option<InitData>,
}

/// Initialize for specific mints.
///
/// The mint must still be updated before it can be used.
///
/// Calling this again for the same mint will **NOT** result in
/// any changes or reinitialization.
#[wasm_bindgen(js_name = init)]
pub fn init(
    SanctumRouterHandle(this): &mut SanctumRouterHandle,
    // Clippy complains, needed for wasm_bindgen
    #[allow(clippy::boxed_local)] init_mints: Box<[InitMint]>,
) -> Result<(), JsError> {
    init_mints.iter().try_for_each(
        |InitMint {
             mint: Bs58Array(mint),
             init,
         }| {
            match *mint {
                // no-op for everything other than spl
                NATIVE_MINT | MSOL_MINT_ADDR | STSOL_MINT_ADDR => Ok(()),
                spl_mint => {
                    let init_data = init.ok_or_else(invalid_data_err)?;
                    match this.spl_routers.entry(spl_mint) {
                        Entry::Occupied(_already_init) => Ok(()),
                        Entry::Vacant(v) => {
                            v.insert(SplStakePoolRouterOwned::init(&init_data)?);
                            Ok(())
                        }
                    }
                }
            }
        },
    )
}

/// Creates a new empty router that needs to have individual mints
/// init and updated for the specific swap
/// before it can start operating for it.
#[wasm_bindgen(js_name = newSanctumRouter)]
pub fn new_sanctum_router() -> Result<SanctumRouterHandle, JsError> {
    Ok(SanctumRouterHandle(SanctumRouter::default()))
}

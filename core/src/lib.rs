// Implementation notes:
//
// - Do not derive tsify for packed structs or structs that contain `[u8; 32]`
//   since we want nice types available on ts side as opposed to just `number[]`.
//   Instead, create a tsify-able newtype in the wasm sdk crate and define conversions to/from

#![cfg_attr(all(not(test), not(feature = "std")), no_std)]

// crate re-exports
pub use sanctum_fee_ratio;
pub use sanctum_marinade_liquid_staking_core;
pub use sanctum_reserve_core;
pub use sanctum_spl_stake_pool_core;
pub use sanctum_u64_ratio;
pub use solido_legacy_core;

mod consts;
mod instructions;
mod internal_utils;
mod pda;
mod quote;
mod routers;
mod traits;

pub use consts::*;
pub use instructions::*;
pub use pda::*;
pub use quote::*;
pub use routers::*;
pub use traits::*;

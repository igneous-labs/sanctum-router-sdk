mod deposit_sol;
mod deposit_stake;

pub use deposit_sol::*;
pub use deposit_stake::*;

use sanctum_marinade_liquid_staking_core::State as MarinadeState;

#[derive(Debug, Clone, Copy)]
pub struct MarinadeQuoter<'a> {
    pub state: &'a MarinadeState,
    pub msol_leg_balance: u64,
}

mod deposit_sol;
mod deposit_stake;

pub use deposit_sol::*;
pub use deposit_stake::*;

use sanctum_marinade_liquid_staking_core::State as MarinadeState;

pub struct MarinadeSolRouter<'a> {
    pub state: &'a MarinadeState,
    pub msol_leg_balance: u64,
}

pub struct MarinadeStakeRouter<'a> {
    pub state: &'a MarinadeState,
    pub msol_leg_balance: u64,
    pub duplication_flag: [u8; 32],
}

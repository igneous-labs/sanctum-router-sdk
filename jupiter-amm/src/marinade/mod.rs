use sanctum_marinade_liquid_staking_core::State as MarinadeState;

mod amm;
mod base;

#[derive(Debug, Clone)]
pub struct MarinadeSolAmm {
    pub state: MarinadeState,
    pub msol_leg_balance: u64,
}

impl Default for MarinadeSolAmm {
    fn default() -> Self {
        Self {
            state: MarinadeState::DEFAULT,
            msol_leg_balance: 0,
        }
    }
}

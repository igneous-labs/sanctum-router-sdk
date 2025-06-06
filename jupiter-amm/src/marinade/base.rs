use sanctum_marinade_liquid_staking_core::State as MarinadeState;
use sanctum_router_core::MarinadeSolRouter;
use solana_sdk::pubkey::Pubkey;

use crate::StakePoolBase;

use super::MarinadeSolAmm;

impl StakePoolBase for MarinadeSolAmm {
    /// Keyed account: MarinadeState
    fn from_keyed_account(
        keyed_account: &jupiter_amm_interface::KeyedAccount,
        _amm_context: &jupiter_amm_interface::AmmContext,
    ) -> anyhow::Result<Self>
    where
        Self: Sized,
    {
        let state = MarinadeState::borsh_de(keyed_account.account.data.as_slice())?;

        Ok(Self {
            state,
            ..Default::default()
        })
    }

    fn program_id(&self) -> solana_sdk::pubkey::Pubkey {
        Pubkey::from(sanctum_marinade_liquid_staking_core::MARINADE_STAKING_PROGRAM)
    }

    fn main_state_key(&self) -> solana_sdk::pubkey::Pubkey {
        Pubkey::from(sanctum_marinade_liquid_staking_core::STATE_PUBKEY)
    }

    fn staked_sol_mint(&self) -> solana_sdk::pubkey::Pubkey {
        Pubkey::from(self.state.msol_mint)
    }

    fn get_accounts_to_update(&self) -> Vec<solana_sdk::pubkey::Pubkey> {
        vec![
            Pubkey::from(sanctum_marinade_liquid_staking_core::STATE_PUBKEY),
            Pubkey::from(sanctum_marinade_liquid_staking_core::LIQ_POOL_MSOL_LEG_PUBKEY),
        ]
    }

    fn update(&mut self, account_map: &jupiter_amm_interface::AccountMap) -> anyhow::Result<()> {
        let state_pk = Pubkey::from(sanctum_marinade_liquid_staking_core::STATE_PUBKEY);
        let state = MarinadeState::borsh_de(
            account_map
                .get(&state_pk)
                .ok_or_else(|| anyhow::anyhow!("{} not in AccountMap", state_pk))?
                .data
                .as_slice(),
        )?;
        self.state = state;

        self.msol_leg_balance = u64::from_le_bytes(
            account_map
                .get(&Pubkey::from(
                    sanctum_marinade_liquid_staking_core::LIQ_POOL_MSOL_LEG_PUBKEY,
                ))
                .ok_or_else(|| anyhow::anyhow!("{} not in AccountMap", state_pk))?
                .data[64..72]
                .try_into()
                .map_err(|_| anyhow::anyhow!("Can't parse MSOL leg token account balance"))?,
        );

        Ok(())
    }

    fn to_router(&self) -> impl sanctum_router_core::DepositSol {
        MarinadeSolRouter {
            state: &self.state,
            msol_leg_balance: self.msol_leg_balance,
        }
    }
}

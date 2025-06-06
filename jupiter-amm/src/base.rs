use jupiter_amm_interface::{AccountMap, AmmContext, KeyedAccount};
use sanctum_router_core::DepositSol;
use solana_sdk::pubkey::Pubkey;

pub trait StakePoolBase {
    fn from_keyed_account(
        keyed_account: &KeyedAccount,
        amm_context: &AmmContext,
    ) -> anyhow::Result<Self>
    where
        Self: Sized;

    fn program_id(&self) -> Pubkey;

    // Only exists for JUP identification purposes
    fn main_state_key(&self) -> Pubkey;

    fn staked_sol_mint(&self) -> Pubkey;

    fn get_accounts_to_update(&self) -> Vec<Pubkey>;

    fn update(&mut self, account_map: &AccountMap) -> anyhow::Result<()>;

    fn to_router(&self) -> impl DepositSol;
}

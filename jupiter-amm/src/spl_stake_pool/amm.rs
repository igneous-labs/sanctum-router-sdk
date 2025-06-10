use jupiter_amm_interface::{Amm, Swap};
use rust_decimal::{
    prelude::{FromPrimitive, Zero},
    Decimal,
};
use sanctum_router_core::{DepositSol, SANCTUM_ROUTER_PROGRAM};

use solana_sdk::instruction::AccountMeta;
use solana_sdk::pubkey::Pubkey;

use crate::{
    keys_signer_writer_to_account_metas, metas::stake_wrapped_sol_prefix_metas, StakePoolBase,
    NATIVE_MINT, TEMPORARY_JUP_AMM_LABEL,
};

use super::SplStakePoolSolAmm;

impl Amm for SplStakePoolSolAmm {
    fn from_keyed_account(
        keyed_account: &jupiter_amm_interface::KeyedAccount,
        amm_context: &jupiter_amm_interface::AmmContext,
    ) -> anyhow::Result<Self>
    where
        Self: Sized,
    {
        StakePoolBase::from_keyed_account(keyed_account, amm_context)
    }

    fn label(&self) -> String {
        TEMPORARY_JUP_AMM_LABEL.to_string()
    }

    fn program_id(&self) -> Pubkey {
        StakePoolBase::program_id(self)
    }

    fn key(&self) -> Pubkey {
        self.main_state_key()
    }

    fn get_reserve_mints(&self) -> Vec<Pubkey> {
        vec![NATIVE_MINT, self.staked_sol_mint()]
    }

    fn get_accounts_to_update(&self) -> Vec<Pubkey> {
        StakePoolBase::get_accounts_to_update(self)
    }

    fn update(&mut self, account_map: &jupiter_amm_interface::AccountMap) -> anyhow::Result<()> {
        StakePoolBase::update(self, account_map)
    }

    fn quote(
        &self,
        quote_params: &jupiter_amm_interface::QuoteParams,
    ) -> anyhow::Result<jupiter_amm_interface::Quote> {
        if quote_params.input_mint == NATIVE_MINT
            && quote_params.output_mint == self.staked_sol_mint()
        {
            let quote = self
                .to_router()
                .get_deposit_sol_quote(quote_params.amount)
                .ok_or_else(|| anyhow::anyhow!("Failed to get quote for {}", self.label()))?;

            Ok(jupiter_amm_interface::Quote {
                in_amount: quote.inp,
                out_amount: quote.out,
                fee_amount: quote.fee,
                fee_mint: Pubkey::from(self.stake_pool.pool_mint),
                fee_pct: Decimal::from_f64((quote.fee as f64) / (quote.fee + quote.out) as f64)
                    .unwrap_or_else(Decimal::zero),
                ..Default::default()
            })
        } else {
            todo!()
        }
    }

    fn get_swap_and_account_metas(
        &self,
        swap_params: &jupiter_amm_interface::SwapParams,
    ) -> anyhow::Result<jupiter_amm_interface::SwapAndAccountMetas> {
        let mut account_metas = vec![AccountMeta {
            pubkey: Pubkey::from(SANCTUM_ROUTER_PROGRAM),
            is_signer: false,
            is_writable: false,
        }];

        if swap_params.source_mint == NATIVE_MINT
            && swap_params.destination_mint == self.staked_sol_mint()
        {
            account_metas.extend_from_slice(&stake_wrapped_sol_prefix_metas(swap_params));

            let suffix_accounts = self.to_router().suffix_accounts();
            let suffix_is_signer = self.to_router().suffix_is_signer();
            let suffix_is_writable = self.to_router().suffix_is_writable();
            account_metas.extend(keys_signer_writer_to_account_metas(
                suffix_accounts.as_ref(),
                suffix_is_signer.as_ref(),
                suffix_is_writable.as_ref(),
            ));
        } else {
            // Add checking for withdrawsol mints and return error otherwise
            todo!()
        }

        Ok(jupiter_amm_interface::SwapAndAccountMetas {
            swap: Swap::StakeDexStakeWrappedSol,
            account_metas,
        })
    }

    fn clone_amm(&self) -> Box<dyn Amm + Send + Sync> {
        Box::new(self.clone())
    }
}

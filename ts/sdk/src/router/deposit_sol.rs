use sanctum_router_core::{
    DepositSolQuoter, DepositSolSufAccs, StakeWrappedSolIxData, StakeWrappedSolPrefixKeysOwned,
    WithRouterFee, NATIVE_MINT, SANCTUM_ROUTER_PROGRAM, STAKE_WRAPPED_SOL_PREFIX_ACCS_LEN,
    STAKE_WRAPPED_SOL_PREFIX_IS_SIGNER, STAKE_WRAPPED_SOL_PREFIX_IS_WRITER, TOKEN_PROGRAM,
};
use serde::{Deserialize, Serialize};
use serde_bytes::ByteBuf;
use tsify_next::Tsify;
use wasm_bindgen::prelude::*;

use crate::{
    err::{generic_err, invalid_pda_err},
    interface::{keys_signer_writer_to_account_metas, AccountMeta, Instruction, B58PK},
    pda::router::find_fee_token_account_pda_internal,
    router::{token_pair::TokenQuoteWithRouterFee, SanctumRouterHandle},
};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi, large_number_types_as_bigints)]
#[serde(rename_all = "camelCase")]
pub struct DepositSolQuoteParams {
    /// Input lamport amount
    pub amt: u64,

    /// Output mint
    pub out: B58PK,
}

/// Requires `update()` to be called before calling this function
#[wasm_bindgen(js_name = quoteDepositSol)]
pub fn quote_deposit_sol(
    this: &SanctumRouterHandle,
    params: DepositSolQuoteParams,
) -> Result<TokenQuoteWithRouterFee, JsError> {
    let out_mint = params.out.0;
    match out_mint {
        sanctum_marinade_liquid_staking_core::MSOL_MINT_ADDR => this
            .0
            .marinade_router
            .deposit_sol_quoter()
            .quote_deposit_sol(params.amt)
            .map_err(generic_err),
        mint => this
            .0
            .try_find_spl_by_mint(&mint)?
            .deposit_sol_quoter()
            .quote_deposit_sol(params.amt)
            .map_err(generic_err),
    }
    .map(|q| TokenQuoteWithRouterFee(WithRouterFee::zero(q)))
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi, large_number_types_as_bigints)]
#[serde(rename_all = "camelCase")]
pub struct DepositSolSwapParams {
    /// Input lamport amount
    pub amt: u64,

    /// Output mint
    pub out: B58PK,

    /// Input token account to transfer `amt` tokens from
    pub signer_inp: B58PK,

    /// Output token account to receive tokens to
    pub signer_out: B58PK,

    /// Signing authority of `self.signer_inp`; user making the swap.
    pub signer: B58PK,
}

/// Requires `update()` to be called before calling this function
#[wasm_bindgen(js_name = depositSolIx)]
pub fn deposit_sol_ix(
    this: &SanctumRouterHandle,
    params: DepositSolSwapParams,
) -> Result<Instruction, JsError> {
    let out_mint = params.out.0;
    let (prefix_metas, data) = deposit_sol_prefix_metas_and_data(&params)?;

    let metas: Box<[AccountMeta]> = match out_mint {
        sanctum_marinade_liquid_staking_core::MSOL_MINT_ADDR => {
            let router = this.0.marinade_router.deposit_sol_suf_accs();

            let suffix_accounts = keys_signer_writer_to_account_metas(
                &router.suffix_accounts().as_borrowed().0,
                &router.suffix_is_signer().0,
                &router.suffix_is_writable().0,
            );

            [prefix_metas.as_ref(), suffix_accounts.as_ref()]
                .concat()
                .into()
        }
        mint => {
            let router = this.0.try_find_spl_by_mint(&mint)?.sol_suf_accs();

            let suffix_accounts = keys_signer_writer_to_account_metas(
                &router.suffix_accounts().as_borrowed().0,
                &router.suffix_is_signer().0,
                &router.suffix_is_writable().0,
            );

            [prefix_metas.as_ref(), suffix_accounts.as_ref()]
                .concat()
                .into()
        }
    };

    let ix = Instruction {
        program_address: B58PK::new(SANCTUM_ROUTER_PROGRAM),
        accounts: metas,
        data: ByteBuf::from(data.to_buf()),
    };

    Ok(ix)
}

fn deposit_sol_prefix_metas_and_data(
    swap_params: &DepositSolSwapParams,
) -> Result<
    (
        [AccountMeta; STAKE_WRAPPED_SOL_PREFIX_ACCS_LEN],
        StakeWrappedSolIxData,
    ),
    JsError,
> {
    let metas = keys_signer_writer_to_account_metas(
        &StakeWrappedSolPrefixKeysOwned::default()
            .with_consts()
            .with_user(swap_params.signer.0)
            .with_wsol_mint(NATIVE_MINT)
            .with_out_mint(swap_params.out.0)
            .with_inp_wsol(swap_params.signer_inp.0)
            .with_out_token(swap_params.signer_out.0)
            .with_token_program(TOKEN_PROGRAM)
            .with_out_fee_token(
                find_fee_token_account_pda_internal(&swap_params.out.0)
                    .ok_or_else(invalid_pda_err)?
                    .0,
            )
            .as_borrowed()
            .0,
        &STAKE_WRAPPED_SOL_PREFIX_IS_SIGNER.0,
        &STAKE_WRAPPED_SOL_PREFIX_IS_WRITER.0,
    );

    let data = StakeWrappedSolIxData::new(swap_params.amt);

    Ok((metas, data))
}

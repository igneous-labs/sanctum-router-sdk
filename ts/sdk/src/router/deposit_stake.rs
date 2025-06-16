use sanctum_router_core::{
    ActiveStakeParams, DepositStakeIxAccsBuilder, DepositStakeIxData, DepositStakeQuoter,
    DepositStakeSufAccs, StakeAccountLamports, WithRouterFee, DEPOSIT_STAKE_IX_ACCS_LEN,
    DEPOSIT_STAKE_IX_IS_SIGNER, DEPOSIT_STAKE_IX_IS_WRITER_NON_WSOL_OUT,
    DEPOSIT_STAKE_IX_IS_WRITER_WSOL_OUT, NATIVE_MINT, SANCTUM_ROUTER_PROGRAM,
};
use serde::{Deserialize, Serialize};
use serde_bytes::ByteBuf;
use tsify_next::Tsify;
use wasm_bindgen::prelude::*;

use crate::{
    err::{generic_err, invalid_pda_err, router_missing_err},
    interface::{keys_signer_writer_to_account_metas, AccountMeta, Instruction, B58PK},
    pda::router::find_fee_token_account_pda_internal,
    router::SanctumRouterHandle,
};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi, large_number_types_as_bigints)]
#[serde(rename_all = "camelCase")]
pub struct DepositStakeQuoteParams {
    /// Validator vote account the stake account to be deposited is delegated to
    pub vote: B58PK,

    /// Balance of the stake account to be deposited
    pub inp: StakeAccountLamports,

    /// Output mint
    pub out: B58PK,
}

impl DepositStakeQuoteParams {
    pub fn to_active_stake_params(self) -> ActiveStakeParams {
        ActiveStakeParams {
            vote: self.vote.0,
            lamports: self.inp,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi, large_number_types_as_bigints)]
#[serde(rename_all = "camelCase")]
pub struct DepositStakeQuote {
    /// Validator vote account `inp` is delegated to
    pub vote: B58PK,

    /// Stake to be deposited
    pub inp: StakeAccountLamports,

    /// Output tokens, after subtracting fees
    pub out: u64,

    /// In terms of output tokens
    pub fee: u64,
}

// need to use a simple newtype here instead of type alias
// otherwise wasm_bindgen shits itself with missing generics
#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi, large_number_types_as_bigints)]
#[serde(rename_all = "camelCase")]
pub struct DepositStakeQuoteWithRouterFee(pub(crate) WithRouterFee<DepositStakeQuote>);

/// Requires `update()` to be called before calling this function
#[wasm_bindgen(js_name = quoteDepositStake)]
pub fn quote_deposit_stake(
    this: &SanctumRouterHandle,
    params: DepositStakeQuoteParams,
) -> Result<DepositStakeQuoteWithRouterFee, JsError> {
    let active_stake_params = params.to_active_stake_params();
    let out_mint = params.out.0;
    match out_mint {
        sanctum_router_core::NATIVE_MINT => this
            .0
            .reserve_router
            .deposit_stake_quoter()
            .quote_deposit_stake(active_stake_params)
            .map_err(generic_err),
        sanctum_marinade_liquid_staking_core::MSOL_MINT_ADDR => this
            .0
            .marinade_router
            .deposit_stake_quoter()
            .quote_deposit_stake(active_stake_params)
            .map_err(generic_err),
        mint => {
            let router = this.0.try_find_spl_by_mint(&mint)?;
            router
                .deposit_stake_quoter(this.0.curr_epoch)
                .quote_deposit_stake(active_stake_params)
                .map_err(generic_err)
        }
    }
    .map(|q| {
        conv_quote(if params.out.0 != sanctum_router_core::NATIVE_MINT {
            q.with_router_fee()
        } else {
            WithRouterFee::zero(q)
        })
    })
}

#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi, large_number_types_as_bigints)]
#[serde(rename_all = "camelCase")]
pub struct DepositStakeSwapParams {
    /// Vote account `self.signer_inp` stake account is delegated to
    pub inp: B58PK,

    /// Output mint
    pub out: B58PK,

    /// Stake account to deposit
    pub signer_inp: B58PK,

    /// Output token account to receive tokens to
    pub signer_out: B58PK,

    /// Signing authority of `self.signer_inp`; user making the swap.
    pub signer: B58PK,
}

/// Requires `update()` to be called before calling this function
///
/// @param {SanctumRouterHandle} _this
/// @param {DepositStakeSwapParams} params
#[wasm_bindgen(js_name = depositStakeIx)]
pub fn deposit_stake_ix(
    this: &SanctumRouterHandle,
    params: DepositStakeSwapParams,
) -> Result<Instruction, JsError> {
    let out_mint = params.out.0;
    let vote_account = params.inp.0;
    let stake_account = params.signer_inp.0;
    let (prefix_metas, data) = deposit_stake_prefix_metas_and_data(&params)?;

    let metas: Box<[AccountMeta]> = match out_mint {
        sanctum_router_core::NATIVE_MINT => {
            let router = this
                .0
                .reserve_router
                .deposit_stake_suf_accs(&stake_account)
                .ok_or_else(router_missing_err)?;

            let suffix_accounts = keys_signer_writer_to_account_metas(
                &router.suffix_accounts().as_borrowed().0,
                &router.suffix_is_signer().0,
                &router.suffix_is_writable().0,
            );

            [prefix_metas.as_ref(), suffix_accounts.as_ref()]
                .concat()
                .into()
        }
        sanctum_marinade_liquid_staking_core::MSOL_MINT_ADDR => {
            let router = this
                .0
                .marinade_router
                .deposit_stake_suf_accs(&vote_account)
                .ok_or_else(router_missing_err)?;

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
            let router = this
                .0
                .try_find_spl_by_mint(&mint)?
                .deposit_stake_suf_accs(&vote_account)
                .ok_or_else(router_missing_err)?;

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

fn conv_quote(
    WithRouterFee {
        quote: sanctum_router_core::DepositStakeQuote { inp, out, fee },
        router_fee,
    }: WithRouterFee<sanctum_router_core::DepositStakeQuote>,
) -> DepositStakeQuoteWithRouterFee {
    DepositStakeQuoteWithRouterFee(WithRouterFee {
        quote: DepositStakeQuote {
            inp: inp.lamports,
            vote: B58PK::new(inp.vote),
            out,
            fee,
        },
        router_fee,
    })
}

fn deposit_stake_prefix_metas_and_data(
    swap_params: &DepositStakeSwapParams,
) -> Result<([AccountMeta; DEPOSIT_STAKE_IX_ACCS_LEN], DepositStakeIxData), JsError> {
    let metas = keys_signer_writer_to_account_metas(
        &DepositStakeIxAccsBuilder::start()
            .with_user(swap_params.signer.0)
            .with_out_token(swap_params.signer_out.0)
            .with_out_fee_token(
                find_fee_token_account_pda_internal(&swap_params.out.0)
                    .ok_or_else(invalid_pda_err)?
                    .0,
            )
            .with_out_mint(swap_params.out.0)
            .with_inp_stake(swap_params.signer_inp.0)
            .build()
            .as_borrowed()
            .0,
        &DEPOSIT_STAKE_IX_IS_SIGNER.0,
        &if swap_params.out.0 == NATIVE_MINT {
            DEPOSIT_STAKE_IX_IS_WRITER_WSOL_OUT
        } else {
            DEPOSIT_STAKE_IX_IS_WRITER_NON_WSOL_OUT
        }
        .0,
    );

    let data = DepositStakeIxData::new();

    Ok((metas, data))
}

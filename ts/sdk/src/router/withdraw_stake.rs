use sanctum_router_core::{
    ActiveStakeParams, Prefund, PrefundWithdrawStakeIxData, PrefundWithdrawStakePrefixAccsBuilder,
    StakeAccountLamports, WithdrawStakeQuoter, WithdrawStakeSufAccs, PREFUNDER,
    PREFUND_WITHDRAW_STAKE_PREFIX_ACCS_LEN, PREFUND_WITHDRAW_STAKE_PREFIX_IS_SIGNER,
    PREFUND_WITHDRAW_STAKE_PREFIX_IS_WRITER, SANCTUM_ROUTER_PROGRAM, STAKE_PROGRAM, SYSTEM_PROGRAM,
    SYSVAR_CLOCK,
};
use serde::{Deserialize, Serialize};
use serde_bytes::ByteBuf;
use solido_legacy_core::LidoError;
use tsify_next::Tsify;
use wasm_bindgen::prelude::*;

use crate::{
    err::{account_missing_err, generic_err, invalid_pda_err, router_missing_err},
    interface::{keys_signer_writer_to_account_metas, AccountMeta, Instruction, B58PK},
    pda::{
        reserve::find_reserve_stake_account_record_pda_internal,
        router::{create_slumdog_stake_internal, find_bridge_stake_acc_internal},
    },
    router::SanctumRouterHandle,
};

#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi, large_number_types_as_bigints)]
#[serde(rename_all = "camelCase")]
pub struct WithdrawStakeQuoteParams {
    /// Input LST amount
    pub amt: u64,

    /// Input mint
    pub inp: B58PK,

    /// Desired vote account of output stake account.
    /// If null, then any vote account of any validator in the stake pool
    /// may be used
    #[tsify(optional)]
    pub out: Option<B58PK>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi, large_number_types_as_bigints)]
#[serde(rename_all = "camelCase")]
pub struct WithdrawStakeQuote {
    /// Validator vote account output stake acc will be delegated to
    pub vote: B58PK,

    /// input tokens
    pub inp: u64,

    /// Output stake account balances, after subtracting fees
    pub out: StakeAccountLamports,

    /// In terms of input tokens, charged by the stake pool
    pub fee: u64,
}

// need to use a simple newtype here instead of type alias
// otherwise wasm_bindgen shits itself with missing generics
#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi, large_number_types_as_bigints)]
#[serde(rename_all = "camelCase")]
pub struct PrefundWithdrawStakeQuote(pub(crate) Prefund<WithdrawStakeQuote>);

/// Requires `update()` to be called before calling this function
#[wasm_bindgen(js_name = quotePrefundWithdrawStake)]
pub fn quote_prefund_withdraw_stake(
    this: &SanctumRouterHandle,
    params: WithdrawStakeQuoteParams,
) -> Result<PrefundWithdrawStakeQuote, JsError> {
    let inp_mint = params.inp.0;
    let out_vote = params.out.map(|pk| pk.0);
    let out_vote = out_vote.as_ref();
    let (reserves_balance, reserves_fee) = this.0.reserve_router.prefund_params();
    let quote = match inp_mint {
        solido_legacy_core::STSOL_MINT_ADDR => this
            .0
            .lido_router
            .withdraw_stake_quoter()
            .ok_or_else(|| account_missing_err(&solido_legacy_core::VALIDATOR_LIST_ADDR))?
            .quote_prefund_withdraw_stake(params.amt, out_vote, &reserves_balance, reserves_fee)
            .map_err(generic_err),
        mint => {
            let router = this.0.try_find_spl_by_mint(&mint)?;
            router
                .withdraw_stake_quoter()
                .quote_prefund_withdraw_stake(params.amt, out_vote, &reserves_balance, reserves_fee)
                .map_err(generic_err)
        }
    }?;
    Ok(conv_prefund_quote(quote))
}

#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi, large_number_types_as_bigints)]
#[serde(rename_all = "camelCase")]
pub struct WithdrawStakeSwapParams {
    /// Input LST amount
    pub amt: u64,

    /// Input mint
    pub inp: B58PK,

    /// Vote account the withdrawn stake account will be delegated to
    pub out: B58PK,

    /// Input token account to transfer `amt` tokens from
    pub signer_inp: B58PK,

    /// Bridge stake seed of the stake account to withdraw
    pub bridge_stake_seed: u32,

    /// Signing authority of `self.signer_inp`; user making the swap.
    pub signer: B58PK,
}

/// Requires `update()` to be called before calling this function
#[wasm_bindgen(js_name = prefundWithdrawStakeIx)]
pub fn prefund_withdraw_stake_ix(
    this: &SanctumRouterHandle,
    params: WithdrawStakeSwapParams,
) -> Result<Instruction, JsError> {
    let inp_mint = params.inp.0;
    let vote = params.out.0;
    let (prefix_metas, data) = prefund_withdraw_stake_prefix_metas_and_data(&params)?;

    let metas: Box<[AccountMeta]> = match inp_mint {
        solido_legacy_core::STSOL_MINT_ADDR => {
            let router = this
                .0
                .lido_router
                .withdraw_stake_suf_accs()
                .ok_or_else(router_missing_err)?;

            if *router.largest_stake_vote != vote {
                return Err(LidoError::ValidatorWithMoreStakeExists.into());
            }

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
                .withdraw_stake_suf_accs(&vote)
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

fn conv_prefund_quote(
    Prefund {
        quote:
            sanctum_router_core::WithdrawStakeQuote {
                inp,
                out: ActiveStakeParams { vote, lamports },
                fee,
            },
        prefund_fee,
    }: Prefund<sanctum_router_core::WithdrawStakeQuote>,
) -> PrefundWithdrawStakeQuote {
    PrefundWithdrawStakeQuote(Prefund {
        quote: WithdrawStakeQuote {
            inp,
            vote: B58PK::new(vote),
            out: lamports,
            fee,
        },
        prefund_fee,
    })
}

fn prefund_withdraw_stake_prefix_metas_and_data(
    swap_params: &WithdrawStakeSwapParams,
) -> Result<
    (
        [AccountMeta; PREFUND_WITHDRAW_STAKE_PREFIX_ACCS_LEN],
        PrefundWithdrawStakeIxData,
    ),
    JsError,
> {
    let (bridge_stake, _bump) =
        find_bridge_stake_acc_internal(&swap_params.signer.0, swap_params.bridge_stake_seed)
            .ok_or_else(invalid_pda_err)?;
    let slumdog_stake = create_slumdog_stake_internal(&bridge_stake);
    let (slumdog_stake_acc_record, _bump) =
        find_reserve_stake_account_record_pda_internal(&slumdog_stake)
            .ok_or_else(invalid_pda_err)?;
    let metas = keys_signer_writer_to_account_metas(
        &PrefundWithdrawStakePrefixAccsBuilder::start()
            .with_user(swap_params.signer.0)
            .with_bridge_stake(bridge_stake)
            .with_slumdog_stake(slumdog_stake)
            .with_slumdog_stake_acc_record(slumdog_stake_acc_record)
            .with_inp_mint(swap_params.inp.0)
            .with_inp_token(swap_params.signer_inp.0)
            .with_clock(SYSVAR_CLOCK)
            .with_prefunder(PREFUNDER)
            .with_stake_program(STAKE_PROGRAM)
            .with_system_program(SYSTEM_PROGRAM)
            .with_unstake_program(sanctum_reserve_core::UNSTAKE_PROGRAM)
            .with_unstake_pool(sanctum_reserve_core::POOL)
            .with_unstake_fee(sanctum_reserve_core::FEE)
            .with_unstake_pool_sol_reserves(sanctum_reserve_core::POOL_SOL_RESERVES)
            .with_unstake_protocol_fee(sanctum_reserve_core::PROTOCOL_FEE)
            .with_unstake_protocol_fee_dest(sanctum_reserve_core::PROTOCOL_FEE_VAULT)
            .build()
            .as_borrowed()
            .0,
        &PREFUND_WITHDRAW_STAKE_PREFIX_IS_SIGNER.0,
        &PREFUND_WITHDRAW_STAKE_PREFIX_IS_WRITER.0,
    );

    let data = PrefundWithdrawStakeIxData::new(swap_params.amt, swap_params.bridge_stake_seed);

    Ok((metas, data))
}

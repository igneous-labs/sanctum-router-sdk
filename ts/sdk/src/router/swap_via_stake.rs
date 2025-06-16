use bs58_fixed_wasm::Bs58Array;
use sanctum_marinade_liquid_staking_core::MSOL_MINT_ADDR;
use sanctum_router_core::{
    quote_prefund_swap_via_stake as core_quote, DepositStakeQuote, DepositStakeSufAccs, Prefund,
    PrefundSwapViaStakeIxData, PrefundSwapViaStakePrefixAccsBuilder, SplWithdrawStakeValQuoter,
    WithRouterFee, WithdrawStakeQuote, WithdrawStakeSufAccs, NATIVE_MINT, PREFUNDER,
    PREFUND_SWAP_VIA_STAKE_PREFIX_ACCS_LEN, PREFUND_SWAP_VIA_STAKE_PREFIX_IS_SIGNER,
    PREFUND_SWAP_VIA_STAKE_PREFIX_IS_WRITER_NON_WSOL_OUT,
    PREFUND_SWAP_VIA_STAKE_PREFIX_IS_WRITER_WSOL_OUT, SANCTUM_ROUTER_PROGRAM, STAKE_PROGRAM,
    SYSTEM_PROGRAM, SYSVAR_CLOCK,
};
use serde::{Deserialize, Serialize};
use serde_bytes::ByteBuf;
use solido_legacy_core::STSOL_MINT_ADDR;
use tsify_next::Tsify;
use wasm_bindgen::prelude::*;

use crate::{
    err::{generic_err, invalid_data_err, invalid_pda_err, router_missing_err},
    interface::{keys_signer_writer_to_account_metas, AccountMeta, Instruction, B58PK},
    pda::{
        reserve::find_reserve_stake_account_record_pda_internal,
        router::{
            create_slumdog_stake_internal, find_bridge_stake_acc_internal,
            find_fee_token_account_pda_internal,
        },
    },
    router::{token_pair::TokenQuoteParams, SanctumRouter, SanctumRouterHandle},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Tsify)]
#[serde(rename_all = "camelCase")]
#[tsify(into_wasm_abi, from_wasm_abi, large_number_types_as_bigints)]
pub struct SwapViaStakeQuote {
    /// Input tokens that will leave the user's wallet
    pub inp: u64,

    /// The amount of tokens received, after fees
    pub out: u64,

    /// Fee charged on withdraw stake leg, in terms of input tokens
    pub inp_fee: u64,

    /// Fee charged on deposit stake leg, in terms of output tokens
    pub out_fee: u64,
}

// need to use a simple newtype here instead of type alias
// otherwise wasm_bindgen shits itself with missing generics
// TODO: this type name is very long but keeps consistency with naming conventions
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi, large_number_types_as_bigints)]
#[serde(rename_all = "camelCase")]
pub struct PrefundSwapViaStakeQuoteWithRouterFee(
    pub(crate) Prefund<WithRouterFee<SwapViaStakeQuote>>,
);

/// Requires `update()` to be called before calling this function
#[wasm_bindgen(js_name = quotePrefundSwapViaStake)]
pub fn quote_prefund_swap_via_stake(
    this: &SanctumRouterHandle,
    TokenQuoteParams {
        amt,
        inp: Bs58Array(inp_mint),
        out: Bs58Array(out_mint),
    }: TokenQuoteParams,
) -> Result<PrefundSwapViaStakeQuoteWithRouterFee, JsError> {
    quote_prefund_swap_via_stake_inner(&this.0, amt, &inp_mint, &out_mint)
        .map(|(wsq, dsq)| map_quote(&out_mint, wsq, dsq))
}

#[inline] // inlining reduces binary size slightly
fn map_quote(
    out_mint: &[u8; 32],
    Prefund {
        quote: WithdrawStakeQuote {
            inp, fee: inp_fee, ..
        },
        prefund_fee,
    }: Prefund<WithdrawStakeQuote>,
    dsq: DepositStakeQuote,
) -> PrefundSwapViaStakeQuoteWithRouterFee {
    let WithRouterFee {
        quote: DepositStakeQuote {
            out, fee: out_fee, ..
        },
        router_fee,
    } = if *out_mint != sanctum_router_core::NATIVE_MINT {
        dsq.with_router_fee()
    } else {
        WithRouterFee::zero(dsq)
    };
    PrefundSwapViaStakeQuoteWithRouterFee(Prefund {
        quote: WithRouterFee {
            quote: SwapViaStakeQuote {
                inp,
                out,
                inp_fee,
                out_fee,
            },
            router_fee,
        },
        prefund_fee,
    })
}

// Used by both quote and ix
//
// TODO: this is potentially a very expensive N^2 operation since we need to
// iterate through the validators of the pool we're withdrawing stake from and then
// the pool we're depositing stake into. Optimize if needed.
fn quote_prefund_swap_via_stake_inner(
    this: &SanctumRouter,
    amt: u64,
    inp_mint: &[u8; 32],
    out_mint: &[u8; 32],
) -> Result<(Prefund<WithdrawStakeQuote>, DepositStakeQuote), JsError> {
    let (reserves_balance, reserves_fee) = this.reserve_router.prefund_params();

    // TODO: if we used dyn or some other means we could reduce
    // number of total match arms (Withdraw + Deposit) from n^2 to n
    // but for now we have this macro in its place to reduce redundancy instead
    macro_rules! match_deposit_stake {
        ($w_itr:expr) => {
            match *out_mint {
                NATIVE_MINT => {
                    let d = this.reserve_router.deposit_stake_quoter().after_prefund()?;
                    core_quote($w_itr, d, amt, &reserves_balance, reserves_fee).map_err(generic_err)
                }
                MSOL_MINT_ADDR => {
                    let d = this.marinade_router.deposit_stake_quoter();
                    core_quote($w_itr, d, amt, &reserves_balance, reserves_fee).map_err(generic_err)
                }
                out => {
                    let d = this.try_find_spl_by_mint(&out)?.deposit_stake_quoter();
                    core_quote($w_itr, d, amt, &reserves_balance, reserves_fee).map_err(generic_err)
                }
            }
        };
    }

    match *inp_mint {
        STSOL_MINT_ADDR => {
            let w_itr = this.lido_router.withdraw_stake_quoter();
            match_deposit_stake!(w_itr)
        }
        inp => {
            let router = this.try_find_spl_by_mint(&inp)?;
            let w_itr = SplWithdrawStakeValQuoter::all(
                &router.stake_pool,
                &router.validator_list.validators,
                router.curr_epoch,
            )?;
            match_deposit_stake!(w_itr)
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi, large_number_types_as_bigints)]
#[serde(rename_all = "camelCase")]
pub struct SwapViaStakeSwapParams {
    /// Input LST amount
    pub amt: u64,

    /// Input mint
    pub inp: B58PK,

    /// Output mint
    pub out: B58PK,

    /// Input token account to transfer `amt` tokens from
    pub signer_inp: B58PK,

    /// Output token account to receive tokens to
    pub signer_out: B58PK,

    /// Bridge stake seed of the intermediate bridge stake account
    pub bridge_stake_seed: u32,

    /// Signing authority of `self.signer_inp`; user making the swap.
    pub signer: B58PK,
}

/// Requires `update()` to be called before calling this function
///
/// @param {SanctumRouterHandle} _this
/// @param {SwapViaStakeSwapParams} params
#[wasm_bindgen(js_name = prefundSwapViaStakeIx)]
pub fn prefund_swap_via_stake_ix(
    this: &SanctumRouterHandle,
    params: SwapViaStakeSwapParams,
) -> Result<Instruction, JsError> {
    let inp_mint = params.inp.0;
    let out_mint = params.out.0;

    let (prefix_metas, data, bridge_stake) = prefund_swap_via_stake_prefix(&params)?;
    let (_wsq, dsq) =
        quote_prefund_swap_via_stake_inner(&this.0, params.amt, &inp_mint, &out_mint)?;
    let vote = dsq.inp.vote;

    let mut metas = Vec::from(prefix_metas);

    match inp_mint {
        STSOL_MINT_ADDR => {
            let w_router = this
                .0
                .lido_router
                .withdraw_stake_suf_accs()
                .ok_or_else(invalid_data_err)?;
            let w_suf = keys_signer_writer_to_account_metas(
                &w_router.suffix_accounts().as_borrowed().0,
                &w_router.suffix_is_signer().0,
                &w_router.suffix_is_writable().0,
            );
            metas.extend(w_suf);
        }
        inp => {
            let w_router = this
                .0
                .try_find_spl_by_mint(&inp)?
                .withdraw_stake_suf_accs(&vote)
                .ok_or_else(invalid_data_err)?;
            let w_suf = keys_signer_writer_to_account_metas(
                &w_router.suffix_accounts().as_borrowed().0,
                &w_router.suffix_is_signer().0,
                &w_router.suffix_is_writable().0,
            );
            metas.extend(w_suf);
        }
    };

    match out_mint {
        NATIVE_MINT => {
            let d_router = this
                .0
                .reserve_router
                .deposit_stake_suf_accs(&bridge_stake)
                .ok_or_else(invalid_pda_err)?;
            let d_suf = keys_signer_writer_to_account_metas(
                &d_router.suffix_accounts().as_borrowed().0,
                &d_router.suffix_is_signer().0,
                &d_router.suffix_is_writable().0,
            );
            metas.extend(d_suf);
        }
        MSOL_MINT_ADDR => {
            let d_router = this
                .0
                .marinade_router
                .deposit_stake_suf_accs(&vote)
                .ok_or_else(invalid_pda_err)?;
            let d_suf = keys_signer_writer_to_account_metas(
                &d_router.suffix_accounts().as_borrowed().0,
                &d_router.suffix_is_signer().0,
                &d_router.suffix_is_writable().0,
            );
            metas.extend(d_suf);
        }
        out => {
            let d_router = this
                .0
                .try_find_spl_by_mint(&out)?
                .deposit_stake_suf_accs(&vote)
                .ok_or_else(router_missing_err)?;
            let d_suf = keys_signer_writer_to_account_metas(
                &d_router.suffix_accounts().as_borrowed().0,
                &d_router.suffix_is_signer().0,
                &d_router.suffix_is_writable().0,
            );
            metas.extend(d_suf);
        }
    };

    let ix = Instruction {
        program_address: B58PK::new(SANCTUM_ROUTER_PROGRAM),
        accounts: metas.into(),
        data: ByteBuf::from(data.to_buf()),
    };

    Ok(ix)
}

/// Returns `(meta, ix_data, bridge_stake_addr)`
fn prefund_swap_via_stake_prefix(
    swap_params: &SwapViaStakeSwapParams,
) -> Result<
    (
        [AccountMeta; PREFUND_SWAP_VIA_STAKE_PREFIX_ACCS_LEN],
        PrefundSwapViaStakeIxData,
        [u8; 32],
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
        &PrefundSwapViaStakePrefixAccsBuilder::start()
            .with_user(swap_params.signer.0)
            .with_inp_token(swap_params.signer_inp.0)
            .with_out_token(swap_params.signer_out.0)
            .with_out_fee_token(
                find_fee_token_account_pda_internal(&swap_params.out.0)
                    .ok_or_else(invalid_pda_err)?
                    .0,
            )
            .with_inp_mint(swap_params.inp.0)
            .with_out_mint(swap_params.out.0)
            .with_prefunder(PREFUNDER)
            .with_bridge_stake(bridge_stake)
            .with_slumdog_stake(slumdog_stake)
            .with_slumdog_stake_acc_record(slumdog_stake_acc_record)
            .with_unstake_program(sanctum_reserve_core::UNSTAKE_PROGRAM)
            .with_unstake_pool(sanctum_reserve_core::POOL)
            .with_unstake_fee(sanctum_reserve_core::FEE)
            .with_unstake_pool_sol_reserves(sanctum_reserve_core::POOL_SOL_RESERVES)
            .with_unstake_protocol_fee(sanctum_reserve_core::PROTOCOL_FEE)
            .with_unstake_protocol_fee_dest(sanctum_reserve_core::PROTOCOL_FEE_VAULT)
            .with_clock(SYSVAR_CLOCK)
            .with_stake_program(STAKE_PROGRAM)
            .with_system_program(SYSTEM_PROGRAM)
            .build()
            .as_borrowed()
            .0,
        &PREFUND_SWAP_VIA_STAKE_PREFIX_IS_SIGNER.0,
        &if swap_params.out.0 == NATIVE_MINT {
            PREFUND_SWAP_VIA_STAKE_PREFIX_IS_WRITER_WSOL_OUT
        } else {
            PREFUND_SWAP_VIA_STAKE_PREFIX_IS_WRITER_NON_WSOL_OUT
        }
        .0,
    );

    let data = PrefundSwapViaStakeIxData::new(swap_params.amt, swap_params.bridge_stake_seed);

    Ok((metas, data, bridge_stake))
}

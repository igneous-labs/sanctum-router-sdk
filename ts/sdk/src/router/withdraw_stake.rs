use sanctum_router_core::{
    ActiveStakeParams, Prefund, WithdrawStakeQuoter, WithdrawStakeSufAccs, SANCTUM_ROUTER_PROGRAM,
};
use solido_legacy_core::LidoError;
use wasm_bindgen::prelude::*;

use crate::{
    err::{account_missing_err, generic_err, router_missing_err},
    instructions::get_prefund_withdraw_stake_prefix_metas_and_data,
    interface::{
        keys_signer_writer_to_account_metas, AccountMeta, Instruction, PrefundWithdrawStakeQuote,
        WithdrawStakeQuote, WithdrawStakeQuoteParams, WithdrawStakeSwapParams, B58PK,
    },
    router::SanctumRouterHandle,
};

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

/// Requires `update()` to be called before calling this function
#[wasm_bindgen(js_name = quotePrefundWithdrawStake)]
pub fn quote_prefund_withdraw_stake(
    this: &SanctumRouterHandle,
    params: WithdrawStakeQuoteParams,
) -> Result<PrefundWithdrawStakeQuote, JsError> {
    let out_vote = params.out_vote.map(|pk| pk.0);
    let out_vote = out_vote.as_ref();
    let (reserves_balance, reserves_fee) = this.0.reserve_router.prefund_params();
    let quote = match params.inp_mint.0 {
        solido_legacy_core::STSOL_MINT_ADDR => this
            .0
            .lido_router
            .withdraw_stake_quoter()
            .ok_or_else(|| account_missing_err(&solido_legacy_core::VALIDATOR_LIST_ADDR))?
            .quote_prefund_withdraw_stake(params.amt, out_vote, &reserves_balance, reserves_fee)
            .map_err(generic_err),
        mint => {
            let router = this
                .0
                .find_spl_by_mint(&mint)
                .ok_or_else(router_missing_err)?;
            router
                .withdraw_stake_quoter()
                .quote_prefund_withdraw_stake(params.amt, out_vote, &reserves_balance, reserves_fee)
                .map_err(generic_err)
        }
    }?;
    Ok(conv_prefund_quote(quote))
}

/// Requires `update()` to be called before calling this function
#[wasm_bindgen(js_name = prefundWithdrawStakeIx)]
pub fn prefund_withdraw_stake_ix(
    this: &SanctumRouterHandle,
    params: WithdrawStakeSwapParams,
) -> Result<Instruction, JsError> {
    let inp_mint = params.inp.0;
    let vote = params.out.0;
    let (prefix_metas, data) = get_prefund_withdraw_stake_prefix_metas_and_data(&params)?;

    let metas: Box<[AccountMeta]> = match inp_mint {
        solido_legacy_core::STSOL_MINT_ADDR => {
            let router = this
                .0
                .lido_router
                .withdraw_stake_suf_accs()
                .ok_or(router_missing_err())?;

            if *router.largest_stake_vote != vote {
                return Err(generic_err(LidoError::ValidatorWithMoreStakeExists));
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
                .find_spl_by_mint(&mint)
                .ok_or(router_missing_err())?
                .withdraw_stake_suf_accs(&vote)
                .ok_or(router_missing_err())?;

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
        data: Box::new(data.to_buf()),
    };

    Ok(ix)
}

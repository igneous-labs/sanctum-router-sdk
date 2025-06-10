use sanctum_marinade_liquid_staking_core::State as MarinadeState;
use sanctum_reserve_core::{Fee, Pool, ProtocolFee};
use sanctum_router_core::{
    DepositSol, DepositStake, DepositStakeQuote, TokenQuote, WithRouterFee, WithdrawSol,
    SANCTUM_ROUTER_PROGRAM,
};
use sanctum_spl_stake_pool_core::StakePool;
use serde::{Deserialize, Serialize};
use tsify_next::Tsify;
use wasm_bindgen::{prelude::wasm_bindgen, JsError};

use crate::{
    err::{account_missing_err, invalid_pda_err, router_missing_err},
    instructions::{
        get_deposit_sol_prefix_metas_and_data, get_deposit_stake_prefix_metas_and_data,
        get_withdraw_wrapped_sol_prefix_metas_and_data,
    },
    interface::{
        get_account_data, keys_signer_writer_to_account_metas, AccountMap, AccountMeta,
        DepositStakeQuoteParams, DepositStakeSwapParams, Instruction, SplPoolAccounts,
        TokenQuoteParams, TokenSwapParams, B58PK,
    },
    pda::spl::{find_deposit_auth_pda_internal, find_withdraw_auth_pda_internal},
    router::Update,
};

mod marinade;
mod reserve;
mod spl;

pub use marinade::*;
pub use reserve::*;
pub use spl::*;

#[wasm_bindgen]
pub struct SanctumRouterHandle(pub(crate) SanctumRouter);

/// Returns the accounts that need to be initialized for the router.
#[wasm_bindgen(js_name = getInitAccounts)]
pub fn get_init_accounts(spl_lsts: Vec<SplPoolAccounts>) -> Box<[B58PK]> {
    spl_lsts
        .iter()
        .flat_map(|accounts| [accounts.pool, accounts.validator_list])
        // TODO: Add lido
        .chain([
            B58PK::new(sanctum_reserve_core::POOL),
            B58PK::new(sanctum_reserve_core::FEE),
            B58PK::new(sanctum_reserve_core::PROTOCOL_FEE),
        ])
        .chain([
            B58PK::new(sanctum_marinade_liquid_staking_core::STATE_PUBKEY),
            B58PK::new(sanctum_marinade_liquid_staking_core::VALIDATOR_LIST_PUBKEY),
        ])
        .collect()
}

/// Creates a new router from the fetched accounts.
#[wasm_bindgen(js_name = fromFetchedAccounts)]
pub fn from_fetched_accounts(
    spl_lsts: Vec<SplPoolAccounts>,
    accounts: AccountMap,
    curr_epoch: u64,
) -> Result<SanctumRouterHandle, JsError> {
    // Marinade
    let [Ok(state_data), Ok(validator_records_data)] = [
        sanctum_marinade_liquid_staking_core::STATE_PUBKEY,
        sanctum_marinade_liquid_staking_core::VALIDATOR_LIST_PUBKEY,
    ]
    .map(|pk| get_account_data(&accounts, pk)) else {
        return Err(JsError::new("Failed to fetch marinade accounts"));
    };

    let mut marinade_router = MarinadeRouterOwned {
        state: MarinadeState::borsh_de(state_data)?,
        validator_records: vec![],
        // Set on `update()`
        msol_leg_balance: 0,
    };

    marinade_router.update_validator_records(
        validator_records_data,
        marinade_router.state.validator_system.validator_list.len() as usize,
    )?;

    // Reserve
    let [Ok(pool_data), Ok(fee_data), Ok(protocol_fee_data)] = [
        sanctum_reserve_core::POOL,
        sanctum_reserve_core::FEE,
        sanctum_reserve_core::PROTOCOL_FEE,
    ]
    .map(|pk| get_account_data(&accounts, pk)) else {
        return Err(JsError::new("Failed to fetch reserve accounts"));
    };

    let reserve_router = ReserveRouterOwned {
        pool: Pool::anchor_de(pool_data)?,
        fee_account: Fee::anchor_de(fee_data)?,
        protocol_fee_account: ProtocolFee::anchor_de(protocol_fee_data)?,
        // Set on `update()`
        pool_sol_reserves: 0,
    };

    // SPL
    let spl_routers = spl_lsts
        .iter()
        .map(|lst| {
            let pool_account = accounts
                .0
                .get(&lst.pool)
                .ok_or(account_missing_err(&lst.pool.0))?;
            let stake_pool_addr = lst.pool.0;
            let program_addr = pool_account.owner.0;
            let pool_data = StakePool::borsh_de(&*pool_account.data)?;

            let mut router = SplStakePoolRouterOwned {
                stake_pool_addr,
                stake_pool_program: program_addr,
                stake_pool: pool_data,
                deposit_authority_program_address: find_deposit_auth_pda_internal(
                    &program_addr,
                    &stake_pool_addr,
                )
                .ok_or(invalid_pda_err())?
                .0,
                withdraw_authority_program_address: find_withdraw_auth_pda_internal(
                    &program_addr,
                    &stake_pool_addr,
                )
                .ok_or(invalid_pda_err())?
                .0,
                curr_epoch,
                ..Default::default()
            };

            let validator_list_data = get_account_data(&accounts, lst.validator_list.0)?;

            router.update_validator_list(validator_list_data)?;
            Ok(router)
        })
        .collect::<Result<Vec<_>, JsError>>()?;

    Ok(SanctumRouterHandle(SanctumRouter {
        spl_routers,
        marinade_router,
        reserve_router,
    }))
}

/// Returns the accounts needed to update a specific routers according to the mint addresses.
#[wasm_bindgen(js_name = getAccountsToUpdate)]
pub fn get_accounts_to_update(
    this: &SanctumRouterHandle,
    // Clippy complains, needed for wasm_bindgen
    #[allow(clippy::boxed_local)] mints: Box<[B58PK]>,
) -> Result<Box<[B58PK]>, JsError> {
    let mut accounts = Vec::new();

    for mint in mints.iter() {
        match mint.0 {
            sanctum_router_core::NATIVE_MINT => {
                accounts.extend(
                    this.0
                        .reserve_router
                        .get_accounts_to_update()
                        .map(B58PK::new),
                );
            }
            sanctum_marinade_liquid_staking_core::MSOL_MINT_ADDR => {
                accounts.extend(
                    this.0
                        .marinade_router
                        .get_accounts_to_update()
                        .map(B58PK::new),
                );
            }
            mint => accounts.extend(
                this.0
                    .spl_routers
                    .iter()
                    .find(|r| r.stake_pool.pool_mint == mint)
                    .ok_or(router_missing_err())?
                    .get_accounts_to_update()
                    .map(B58PK::new),
            ),
        }
    }

    Ok(accounts.into_boxed_slice())
}

/// Updates a specific routers according to the mint addresses.
#[wasm_bindgen(js_name = update)]
pub fn update(
    this: &mut SanctumRouterHandle,
    // Clippy complains, needed for wasm_bindgen
    #[allow(clippy::boxed_local)] mints: Box<[B58PK]>,
    accounts: AccountMap,
) -> Result<(), JsError> {
    for mint in mints.iter() {
        match mint.0 {
            sanctum_router_core::NATIVE_MINT => {
                this.0.reserve_router.update(&accounts)?;
            }
            sanctum_marinade_liquid_staking_core::MSOL_MINT_ADDR => {
                this.0.marinade_router.update(&accounts)?;
            }
            mint => {
                this.0
                    .spl_routers
                    .iter_mut()
                    .find(|r| r.stake_pool.pool_mint == mint)
                    .ok_or(router_missing_err())?
                    .update(&accounts)?;
            }
        }
    }

    Ok(())
}

// need to use a simple newtype here instead of type alias
// otherwise wasm_bindgen shits itself with missing generics
#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi, large_number_types_as_bigints)]
#[serde(rename_all = "camelCase")]
pub struct TokenQuoteWithRouterFee(WithRouterFee<TokenQuote>);

/// Requires `update()` to be called before calling this function
#[wasm_bindgen(js_name = getDepositSolQuote)]
pub fn get_deposit_sol_quote(
    this: &SanctumRouterHandle,
    params: TokenQuoteParams,
) -> Option<TokenQuoteWithRouterFee> {
    match params.out_mint.0 {
        sanctum_marinade_liquid_staking_core::MSOL_MINT_ADDR => this
            .0
            .marinade_router
            .to_deposit_sol_router()
            .get_deposit_sol_quote(params.amt),
        mint => this
            .0
            .spl_routers
            .iter()
            .find(|r| r.stake_pool.pool_mint == mint)?
            .to_deposit_sol_router()
            .get_deposit_sol_quote(params.amt),
    }
    .map(|q| TokenQuoteWithRouterFee(WithRouterFee::zero(q)))
}

/// Requires `update()` to be called before calling this function
#[wasm_bindgen(js_name = getDepositSolIx)]
pub fn get_deposit_sol_ix(
    this: &SanctumRouterHandle,
    params: TokenSwapParams,
) -> Result<Instruction, JsError> {
    let out_mint = params.out.0;
    let (prefix_metas, data) = get_deposit_sol_prefix_metas_and_data(params)?;

    let metas: Box<[AccountMeta]> = match out_mint {
        sanctum_marinade_liquid_staking_core::MSOL_MINT_ADDR => {
            let router = this.0.marinade_router.to_deposit_sol_router();

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
                .spl_routers
                .iter()
                .find(|r| r.stake_pool.pool_mint == mint)
                .ok_or(router_missing_err())?
                .to_deposit_sol_router();

            let suffix_accounts = keys_signer_writer_to_account_metas(
                &DepositSol::suffix_accounts(&router).as_borrowed().0,
                &DepositSol::suffix_is_signer(&router).0,
                &DepositSol::suffix_is_writable(&router).0,
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

/// Requires `update()` to be called before calling this function
#[wasm_bindgen(js_name = getWithdrawSolQuote)]
pub fn get_withdraw_sol_quote(
    this: &SanctumRouterHandle,
    params: TokenQuoteParams,
) -> Option<TokenQuoteWithRouterFee> {
    this.0
        .spl_routers
        .iter()
        .find(|r| r.stake_pool.pool_mint == params.inp_mint.0)?
        .to_withdraw_sol_router()
        .get_withdraw_sol_quote(params.amt)
        .map(|q| TokenQuoteWithRouterFee(q.withdraw_sol_with_router_fee()))
}

/// Requires `update()` to be called before calling this function
#[wasm_bindgen(js_name = getWithdrawSolIx)]
pub fn get_withdraw_sol_ix(
    this: &SanctumRouterHandle,
    params: TokenSwapParams,
) -> Result<Instruction, JsError> {
    let router = this
        .0
        .spl_routers
        .iter()
        .find(|r| r.stake_pool.pool_mint == params.inp.0)
        .ok_or(router_missing_err())?
        .to_withdraw_sol_router();

    let (prefix_metas, data) = get_withdraw_wrapped_sol_prefix_metas_and_data(params)?;

    let suffix_accounts = keys_signer_writer_to_account_metas(
        &WithdrawSol::suffix_accounts(&router).as_borrowed().0,
        &WithdrawSol::suffix_is_signer(&router).0,
        &WithdrawSol::suffix_is_writable(&router).0,
    );

    Ok(Instruction {
        program_address: B58PK::new(SANCTUM_ROUTER_PROGRAM),
        accounts: [prefix_metas.as_ref(), suffix_accounts.as_ref()]
            .concat()
            .into(),
        data: Box::new(data.to_buf()),
    })
}

// need to use a simple newtype here instead of type alias
// otherwise wasm_bindgen shits itself with missing generics
#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi, large_number_types_as_bigints)]
#[serde(rename_all = "camelCase")]
pub struct DepositStakeQuoteWithRouterFee(WithRouterFee<DepositStakeQuote>);

/// Requires `update()` to be called before calling this function
#[wasm_bindgen(js_name = getDepositStakeQuote)]
pub fn get_deposit_stake_quote(
    this: &mut SanctumRouterHandle,
    params: DepositStakeQuoteParams,
) -> Option<DepositStakeQuoteWithRouterFee> {
    match params.out_mint.0 {
        sanctum_router_core::NATIVE_MINT => this
            .0
            .reserve_router
            // StakeAccRecord not relevant for quoting
            .to_deposit_stake_router(&[0; 32])?
            .get_deposit_stake_quote(params.inp_stake),
        sanctum_marinade_liquid_staking_core::MSOL_MINT_ADDR => this
            .0
            .marinade_router
            .to_deposit_stake_router(&params.validator_vote.0)?
            .get_deposit_stake_quote(params.inp_stake),
        mint => {
            let router = this
                .0
                .spl_routers
                .iter()
                .find(|r| r.stake_pool.pool_mint == mint)?;

            router
                .to_deposit_stake_router(&params.validator_vote.0)?
                .get_deposit_stake_quote(params.inp_stake)
        }
    }
    .map(|q| {
        DepositStakeQuoteWithRouterFee(if params.out_mint.0 != sanctum_router_core::NATIVE_MINT {
            q.with_router_fee()
        } else {
            WithRouterFee::zero(q)
        })
    })
}

/// Requires `update()` to be called before calling this function
/// Stake account to deposit should be set on `params.signerInp`
/// Vote account of the stake account to deposit should be set on `params.inp`
#[wasm_bindgen(js_name = getDepositStakeIx)]
pub fn get_deposit_stake_ix(
    this: &mut SanctumRouterHandle,
    params: DepositStakeSwapParams,
) -> Result<Instruction, JsError> {
    let out_mint = params.out.0;
    let vote_account = params.inp.0;
    let stake_account = params.signer_inp.0;
    let (prefix_metas, data) = get_deposit_stake_prefix_metas_and_data(params)?;

    let metas: Box<[AccountMeta]> = match out_mint {
        sanctum_router_core::NATIVE_MINT => {
            let router = this
                .0
                .reserve_router
                .to_deposit_stake_router(&stake_account)
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
        sanctum_marinade_liquid_staking_core::MSOL_MINT_ADDR => {
            let router = this
                .0
                .marinade_router
                .to_deposit_stake_router(&vote_account)
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
        mint => {
            let router = this
                .0
                .spl_routers
                .iter()
                .find(|r| r.stake_pool.pool_mint == mint)
                .ok_or(router_missing_err())?
                .to_deposit_stake_router(&vote_account)
                .ok_or(router_missing_err())?;

            let suffix_accounts = keys_signer_writer_to_account_metas(
                &DepositStake::suffix_accounts(&router).as_borrowed().0,
                &DepositStake::suffix_is_signer(&router).0,
                &DepositStake::suffix_is_writable(&router).0,
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
#[derive(Clone, Debug)]
pub struct SanctumRouter {
    pub spl_routers: Vec<SplStakePoolRouterOwned>,
    pub marinade_router: MarinadeRouterOwned,
    pub reserve_router: ReserveRouterOwned,
}

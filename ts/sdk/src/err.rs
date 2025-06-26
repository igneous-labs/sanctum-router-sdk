use sanctum_marinade_liquid_staking_core::MarinadeError;
use sanctum_reserve_core::ReserveError;
use sanctum_router_core::{PrefundSwapViaStakeQuoteErr, PrefundWithdrawStakeQuoteErr};
use sanctum_spl_stake_pool_core::SplStakePoolError;
use serde::{Deserialize, Serialize};
use solido_legacy_core::LidoError;
use tsify_next::Tsify;
use wasm_bindgen::prelude::*;

use crate::{interface::Bs58PkString, update::PoolUpdateType};

/// All {@link Error} objects thrown by SDK functions will start with
/// `{SanctumRouterErr}:`, so that the `SanctumRouterErr` error code can be
/// extracted by splitting on the first colon `:`
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[allow(clippy::enum_variant_names)] // we want all the ts consts to have `Err` suffix
pub enum SanctumRouterErr {
    AccountMissingErr,
    InvalidPdaErr,
    InvalidDataErr,
    RouterMissingErr,
    UnsupportedUpdateErr,
    UserErr,
    PoolErr,
    InternalErr,
}

/// Top level error, all fallible functions should
/// have this as Result's err type to throw the appropriate `JsError`
#[derive(Debug)]
pub struct SanctumRouterError {
    pub code: SanctumRouterErr,

    pub cause: Option<String>,
}

impl From<SanctumRouterError> for JsValue {
    fn from(SanctumRouterError { code, cause }: SanctumRouterError) -> Self {
        let suf = cause.unwrap_or_default();
        JsError::new(&format!("{code:?}{ERR_CODE_MSG_SEP}{suf}")).into()
    }
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct AllSanctumRouterErrs(#[tsify(type = "SanctumRouterErr[]")] pub [SanctumRouterErr; 8]);

/// Returns the array of all possible {@link SanctumRouterErr}s
#[wasm_bindgen(js_name = allSanctumRouterErrs)]
pub fn all_sanctum_router_errs() -> AllSanctumRouterErrs {
    use SanctumRouterErr::*;

    AllSanctumRouterErrs([
        AccountMissingErr,
        InvalidPdaErr,
        InvalidDataErr,
        RouterMissingErr,
        UnsupportedUpdateErr,
        UserErr,
        PoolErr,
        InternalErr,
    ])
}

const ERR_CODE_MSG_SEP: &str = ":";

pub fn invalid_pda_err() -> SanctumRouterError {
    SanctumRouterError {
        code: SanctumRouterErr::InvalidPdaErr,
        cause: None,
    }
}

// TODO: maybe add more details here
pub fn invalid_data_err() -> SanctumRouterError {
    SanctumRouterError {
        code: SanctumRouterErr::InvalidDataErr,
        cause: None,
    }
}

pub fn router_missing_err(mint: &[u8; 32]) -> SanctumRouterError {
    let b58mintstr = Bs58PkString::encode(mint);
    SanctumRouterError {
        code: SanctumRouterErr::RouterMissingErr,
        cause: Some(format!("router missing for mint {b58mintstr}")),
    }
}

pub fn account_missing_err(pubkey: &[u8; 32]) -> SanctumRouterError {
    let b58pkstr = Bs58PkString::encode(pubkey);
    SanctumRouterError {
        code: SanctumRouterErr::AccountMissingErr,
        cause: Some(format!("{b58pkstr} missing from AccountMap")),
    }
}

pub fn marinade_err(e: MarinadeError) -> SanctumRouterError {
    const MARINADE_ERR_PREFIX: &str = "MarinadeError::";

    let (code, cause) = match e {
        MarinadeError::DepositAmountIsTooLow
        | MarinadeError::TooLowDelegationInDepositingStake
        | MarinadeError::WithdrawStakeLamportsIsTooLow
        | MarinadeError::SelectedStakeAccountHasNotEnoughFunds
        | MarinadeError::StakeAccountRemainderTooLow
        | MarinadeError::WrongValidatorAccountOrIndex => (
            SanctumRouterErr::UserErr,
            format!("{MARINADE_ERR_PREFIX}{e}"),
        ),
        MarinadeError::ProgramIsPaused
        | MarinadeError::StakingIsCapped
        | MarinadeError::WithdrawStakeAccountIsNotEnabled
        | MarinadeError::StakeAccountIsEmergencyUnstaking => (
            SanctumRouterErr::PoolErr,
            format!("{MARINADE_ERR_PREFIX}{e}"),
        ),
        MarinadeError::CalculationFailure => (
            SanctumRouterErr::InternalErr,
            format!("{MARINADE_ERR_PREFIX}{e}"),
        ),
    };
    SanctumRouterError {
        code,
        cause: Some(cause),
    }
}

pub fn spl_err(e: SplStakePoolError) -> SanctumRouterError {
    const SPL_ERR_PREFIX: &str = "SplStakePoolError::";

    let (code, cause) = match e {
        SplStakePoolError::IncorrectDepositVoteAddress
        | SplStakePoolError::IncorrectWithdrawVoteAddress
        | SplStakePoolError::InvalidSolDepositAuthority
        | SplStakePoolError::InvalidStakeDepositAuthority
        | SplStakePoolError::ValidatorNotFound => {
            (SanctumRouterErr::UserErr, format!("{SPL_ERR_PREFIX}{e}"))
        }
        SplStakePoolError::InvalidState
        | SplStakePoolError::StakeListAndPoolOutOfDate
        | SplStakePoolError::SolWithdrawalTooLarge
        | SplStakePoolError::StakeLamportsNotEqualToMinimum => {
            (SanctumRouterErr::PoolErr, format!("{SPL_ERR_PREFIX}{e}"))
        }
        SplStakePoolError::CalculationFailure => (
            SanctumRouterErr::InternalErr,
            format!("{SPL_ERR_PREFIX}{e}"),
        ),
    };

    SanctumRouterError {
        code,
        cause: Some(cause),
    }
}

pub fn lido_err(e: LidoError) -> SanctumRouterError {
    const LIDO_ERR_PREFIX: &str = "LidoError::";

    let (code, cause) = match e {
        LidoError::ValidatorWithMoreStakeExists => {
            (SanctumRouterErr::UserErr, format!("{LIDO_ERR_PREFIX}{e}"))
        }
        LidoError::InvalidAmount | LidoError::ExchangeRateNotUpdatedInThisEpoch => {
            (SanctumRouterErr::PoolErr, format!("{LIDO_ERR_PREFIX}{e}"))
        }
        LidoError::CalculationFailure => (
            SanctumRouterErr::InternalErr,
            format!("{LIDO_ERR_PREFIX}{e}"),
        ),
    };

    SanctumRouterError {
        code,
        cause: Some(cause),
    }
}

pub fn reserve_err(e: ReserveError) -> SanctumRouterError {
    const RESERVE_ERR_PREFIX: &str = "ReserveError::";

    let (code, cause) = match e {
        ReserveError::NotEnoughLiquidity => (
            SanctumRouterErr::PoolErr,
            format!("{RESERVE_ERR_PREFIX}{e}"),
        ),
        ReserveError::InternalError => (
            SanctumRouterErr::InternalErr,
            format!("{RESERVE_ERR_PREFIX}{e}"),
        ),
    };

    SanctumRouterError {
        code,
        cause: Some(cause),
    }
}

pub fn prefund_wsq_err<E>(
    e: PrefundWithdrawStakeQuoteErr<E>,
    handle_pool: fn(E) -> SanctumRouterError,
) -> SanctumRouterError {
    match e {
        PrefundWithdrawStakeQuoteErr::Reserve(e) => reserve_err(e),
        PrefundWithdrawStakeQuoteErr::Pool(e) => handle_pool(e),
    }
}

pub fn prefund_svsq_err<W, D>(
    e: PrefundSwapViaStakeQuoteErr<W, D>,
    handle_w: fn(W) -> SanctumRouterError,
    handle_d: fn(D) -> SanctumRouterError,
) -> SanctumRouterError {
    match e {
        PrefundSwapViaStakeQuoteErr::NoMatch => SanctumRouterError {
            code: SanctumRouterErr::PoolErr,
            cause: Some("NoMatch".to_owned()),
        },
        PrefundSwapViaStakeQuoteErr::Reserve(e) => reserve_err(e),
        PrefundSwapViaStakeQuoteErr::WithdrawStake(e) => handle_w(e),
        PrefundSwapViaStakeQuoteErr::DepositStake(e) => handle_d(e),
    }
}

pub fn unsupported_update_err(ty: PoolUpdateType, mint: &[u8; 32]) -> SanctumRouterError {
    let b58mintstr = Bs58PkString::encode(mint);
    SanctumRouterError {
        code: SanctumRouterErr::UnsupportedUpdateErr,
        cause: Some(format!("{ty:?} not supported by pool of mint {b58mintstr}")),
    }
}

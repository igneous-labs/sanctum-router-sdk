use const_format::formatcp;
use sanctum_marinade_liquid_staking_core::MarinadeError;
use sanctum_reserve_core::ReserveError;
use sanctum_router_core::{PrefundSwapViaStakeQuoteErr, PrefundWithdrawStakeQuoteErr};
use sanctum_spl_stake_pool_core::SplStakePoolError;
use solido_legacy_core::LidoError;
use wasm_bindgen::{intern, prelude::*};

use crate::{interface::Bs58PkString, update::PoolUpdateType};

macro_rules! def_errconst {
    ($NAME:ident) => {
        #[allow(unused)]
        pub const $NAME: &str = stringify!($NAME);

        // isolate the export in a module
        // so we can use the same ident for the const
        // because you cant create new idents in macro_rules!
        #[allow(non_snake_case, unused)]
        mod $NAME {
            use super::$NAME as name;

            #[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
            const _EXPORT: &str = const_format::formatcp!(r#"export const {name} = "{name}";"#);
        }
    };
}

// No trailing commas allowed
macro_rules! def_errconsts {
    // base-case: (msut be placed above recursive-case for match priority)
    //
    // - expand individual using def_errconst!()
    // - appends `typeof $NAME` to ts_union expr, but omits trailing ` | `, since this is the last one,
    //   then puts the `ts_union` expr into the proper `typescript_custom_section` export
    (@ts_union $ts_union:expr; $NAME:ident) => {
        def_errconst!($NAME);

        const _FINAL_ERR_TS_UNION: &str = concat!($ts_union, "typeof ", stringify!($NAME));

        #[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
        const _ERR_TS_UNION: &str = const_format::formatcp!(r#"
/**
 * All {{@link Error}} objects thrown by SDK functions will start with
 * `{{SanctumRouterErr}}:`, so that the `SanctumRouterErr` error code can be
 * extracted by splitting on the first colon `:`
 */
export type SanctumRouterErr = {_FINAL_ERR_TS_UNION};
"#);
    };

    // recursive-case:
    //
    // - expand individual using def_errconst!()
    // - appends `typeof $NAME | ` to ts_union expr
    (@ts_union $ts_union:expr; $NAME:ident $(, $($tail:tt)*)?) => {
        def_errconst!($NAME);

        def_errconsts!(@ts_union concat!($ts_union, "typeof ", stringify!($NAME), " | ");  $($($tail)*)?);
    };

    // start
    ($($tail:tt)*) => { def_errconsts!(@ts_union ""; $($tail)*); };
}

def_errconsts!(
    ACCOUNT_MISSING_ERR,
    INVALID_PDA_ERR,
    INVALID_DATA_ERR,
    ROUTER_MISSING_ERR,
    UNSUPPORTED_UPDATE_ERR,
    USER_ERR,
    POOL_ERR,
    INTERNAL_ERR
);

const ERR_CODE_MSG_SEP: &str = ":";

pub fn invalid_pda_err() -> JsError {
    JsError::new(intern(formatcp!("{INVALID_PDA_ERR}{ERR_CODE_MSG_SEP}")))
}

pub fn invalid_data_err() -> JsError {
    JsError::new(intern(formatcp!("{INVALID_DATA_ERR}{ERR_CODE_MSG_SEP}")))
}

// TODO: mint as arg
pub fn router_missing_err() -> JsError {
    JsError::new(intern(formatcp!("{ROUTER_MISSING_ERR}{ERR_CODE_MSG_SEP}")))
}

pub fn account_missing_err(pubkey: &[u8; 32]) -> JsError {
    let b58pkstr = Bs58PkString::encode(pubkey);
    JsError::new(&format!(
        "{ACCOUNT_MISSING_ERR}{ERR_CODE_MSG_SEP}{b58pkstr} missing from AccountMap"
    ))
}

pub fn marinade_err(e: MarinadeError) -> JsError {
    const MARINADE_ERR_PREFIX: &str = "MarinadeError::";

    let s = match e {
        MarinadeError::DepositAmountIsTooLow
        | MarinadeError::TooLowDelegationInDepositingStake
        | MarinadeError::WithdrawStakeLamportsIsTooLow
        | MarinadeError::SelectedStakeAccountHasNotEnoughFunds
        | MarinadeError::StakeAccountRemainderTooLow
        | MarinadeError::WrongValidatorAccountOrIndex => {
            format!("{USER_ERR}{ERR_CODE_MSG_SEP}{MARINADE_ERR_PREFIX}{e}")
        }
        MarinadeError::ProgramIsPaused
        | MarinadeError::StakingIsCapped
        | MarinadeError::WithdrawStakeAccountIsNotEnabled
        | MarinadeError::StakeAccountIsEmergencyUnstaking => {
            format!("{POOL_ERR}{ERR_CODE_MSG_SEP}{MARINADE_ERR_PREFIX}{e}")
        }
        MarinadeError::CalculationFailure => {
            format!("{INTERNAL_ERR}{ERR_CODE_MSG_SEP}{MARINADE_ERR_PREFIX}{e}")
        }
    };
    JsError::new(&s)
}

pub fn spl_err(e: SplStakePoolError) -> JsError {
    const SPL_ERR_PREFIX: &str = "SplStakePoolError::";

    let s = match e {
        SplStakePoolError::IncorrectDepositVoteAddress
        | SplStakePoolError::IncorrectWithdrawVoteAddress
        | SplStakePoolError::InvalidSolDepositAuthority
        | SplStakePoolError::InvalidStakeDepositAuthority
        | SplStakePoolError::SolWithdrawalTooLarge
        | SplStakePoolError::StakeLamportsNotEqualToMinimum
        | SplStakePoolError::ValidatorNotFound => {
            format!("{USER_ERR}{ERR_CODE_MSG_SEP}{SPL_ERR_PREFIX}{e}")
        }
        SplStakePoolError::InvalidState | SplStakePoolError::StakeListAndPoolOutOfDate => {
            format!("{POOL_ERR}{ERR_CODE_MSG_SEP}{SPL_ERR_PREFIX}{e}")
        }
        SplStakePoolError::CalculationFailure => {
            format!("{INTERNAL_ERR}{ERR_CODE_MSG_SEP}{SPL_ERR_PREFIX}{e}")
        }
    };
    JsError::new(&s)
}

pub fn lido_err(e: LidoError) -> JsError {
    const LIDO_ERR_PREFIX: &str = "LidoError::";

    let s = match e {
        LidoError::InvalidAmount | LidoError::ValidatorWithMoreStakeExists => {
            format!("{USER_ERR}{ERR_CODE_MSG_SEP}{LIDO_ERR_PREFIX}{e}")
        }
        LidoError::ExchangeRateNotUpdatedInThisEpoch => {
            format!("{POOL_ERR}{ERR_CODE_MSG_SEP}{LIDO_ERR_PREFIX}{e}")
        }
        LidoError::CalculationFailure => {
            format!("{INTERNAL_ERR}{ERR_CODE_MSG_SEP}{LIDO_ERR_PREFIX}{e}")
        }
    };
    JsError::new(&s)
}

pub fn reserve_err(e: ReserveError) -> JsError {
    const RESERVE_ERR_PREFIX: &str = "ReserveError::";

    let s = match e {
        ReserveError::NotEnoughLiquidity => {
            format!("{POOL_ERR}{ERR_CODE_MSG_SEP}{RESERVE_ERR_PREFIX}{e}")
        }
        ReserveError::InternalError => {
            format!("{INTERNAL_ERR}{ERR_CODE_MSG_SEP}{RESERVE_ERR_PREFIX}{e}")
        }
    };
    JsError::new(&s)
}

pub fn prefund_wsq_err<E>(
    e: PrefundWithdrawStakeQuoteErr<E>,
    handle_pool: fn(E) -> JsError,
) -> JsError {
    match e {
        PrefundWithdrawStakeQuoteErr::Reserve(e) => reserve_err(e),
        PrefundWithdrawStakeQuoteErr::Pool(e) => handle_pool(e),
    }
}

pub fn prefund_svsq_err<W, D>(
    e: PrefundSwapViaStakeQuoteErr<W, D>,
    handle_w: fn(W) -> JsError,
    handle_d: fn(D) -> JsError,
) -> JsError {
    match e {
        PrefundSwapViaStakeQuoteErr::NoMatch => {
            JsError::new(&format!("{POOL_ERR}{ERR_CODE_MSG_SEP}NoMatch"))
        }
        PrefundSwapViaStakeQuoteErr::Reserve(e) => reserve_err(e),
        PrefundSwapViaStakeQuoteErr::WithdrawStake(e) => handle_w(e),
        PrefundSwapViaStakeQuoteErr::DepositStake(e) => handle_d(e),
    }
}

pub fn unsupported_update_err(ty: PoolUpdateType, mint: &[u8; 32]) -> JsError {
    let b58mintstr = Bs58PkString::encode(mint);
    JsError::new(&format!(
        "{UNSUPPORTED_UPDATE_ERR}{ERR_CODE_MSG_SEP}{ty:?} not supported by pool of mint {b58mintstr}"
    ))
}

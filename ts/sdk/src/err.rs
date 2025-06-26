use std::fmt::Display;

use const_format::formatcp;
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
    // base-case: (msut be placed above recursive-case for match prio)
    //
    // - expand individual using def_errconst!()
    // - appends `typeof $NAME` to ts_union expr, but omits trailing |, since this is the last one,
    //   then puts the `ts_union` expr into the proper `typescript_custom_section` export
    (@ts_union $ts_union:expr; $NAME:ident) => {
        def_errconst!($NAME);

        const _FINAL_ERR_TS_UNION: &str = concat!($ts_union, "typeof ", stringify!($NAME));

        #[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
        const _ERR_TS_UNION: &str = const_format::formatcp!(r#"export type SanctumRouterErr = {_FINAL_ERR_TS_UNION};"#);
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
    UNSUPPORTED_UPDATE_ERR
);

const ERR_CODE_MSG_SEP: &str = ":";

pub fn account_missing_err(pubkey: &[u8; 32]) -> JsError {
    JsError::new(&format!(
        "{ACCOUNT_MISSING_ERR}{ERR_CODE_MSG_SEP}{} missing from AccountMap",
        Bs58PkString::encode(pubkey)
    ))
}

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

pub fn generic_err(e: impl Display) -> JsError {
    JsError::new(&format!("{e}"))
}

pub fn unsupported_update_err(ty: PoolUpdateType, mint: &[u8; 32]) -> JsError {
    JsError::new(&format!(
        "{UNSUPPORTED_UPDATE_ERR}{ERR_CODE_MSG_SEP}{:?} not supported by pool of mint {}",
        ty,
        Bs58PkString::encode(mint)
    ))
}

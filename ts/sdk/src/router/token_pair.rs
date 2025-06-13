use sanctum_router_core::{TokenQuote, WithRouterFee};
use serde::{Deserialize, Serialize};
use tsify_next::Tsify;

use crate::interface::B58PK;

#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi, large_number_types_as_bigints)]
#[serde(rename_all = "camelCase")]
pub struct TokenQuoteParams {
    pub amt: u64,

    /// Input mint
    pub inp: B58PK,

    /// Output mint
    pub out: B58PK,
}

// need to use a simple newtype here instead of type alias
// otherwise wasm_bindgen shits itself with missing generics
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi, large_number_types_as_bigints)]
#[serde(rename_all = "camelCase")]
pub struct TokenQuoteWithRouterFee(pub(crate) WithRouterFee<TokenQuote>);

#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi, large_number_types_as_bigints)]
#[serde(rename_all = "camelCase")]
pub struct TokenSwapParams {
    pub amt: u64,

    /// Input mint
    pub inp: B58PK,

    /// Output mint
    pub out: B58PK,

    /// Input token account to transfer `amt` tokens from
    pub signer_inp: B58PK,

    /// Output token account to receive tokens to
    pub signer_out: B58PK,

    /// Signing authority of `self.signer_inp`; user making the swap.
    pub signer: B58PK,
}

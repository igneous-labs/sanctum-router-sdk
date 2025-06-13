use sanctum_router_core::{TokenQuote, WithRouterFee};
use serde::{Deserialize, Serialize};
use tsify_next::Tsify;

// need to use a simple newtype here instead of type alias
// otherwise wasm_bindgen shits itself with missing generics
#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi, large_number_types_as_bigints)]
#[serde(rename_all = "camelCase")]
pub struct TokenQuoteWithRouterFee(pub(crate) WithRouterFee<TokenQuote>);

use crate::ActiveStakeParams;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct WithdrawStakeQuote {
    /// Input pool tokens
    pub inp: u64,

    /// The stake account that will be withdrawn
    pub out: ActiveStakeParams,

    /// In terms of input tokens, charged by the stake pool
    pub fee: u64,
}

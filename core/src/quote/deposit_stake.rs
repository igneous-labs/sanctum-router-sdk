use crate::{ActiveStakeParams, WithRouterFee, DEPOSIT_STAKE_GLOBAL_FEE_BPS};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DepositStakeQuote {
    /// The stake account to be deposited
    pub inp: ActiveStakeParams,

    /// Output tokens, after subtracting fees
    pub out: u64,

    /// In terms of output tokens
    pub fee: u64,
}

impl DepositStakeQuote {
    pub fn with_router_fee(self) -> WithRouterFee<Self> {
        let Self { inp, out, fee } = self;

        if out == 0 {
            return WithRouterFee::zero(self);
        }

        // cast-safety: DEPOSIT_STAKE_GLOBAL_FEE_BPS is < 10_000, no overflow should happen
        let router_fee =
            ((out as u128) * (DEPOSIT_STAKE_GLOBAL_FEE_BPS as u128) / 10_000u128) as u64;
        let router_fee = router_fee.max(1);

        WithRouterFee {
            quote: Self {
                inp,
                out: out - router_fee,
                fee,
            },
            router_fee,
        }
    }
}

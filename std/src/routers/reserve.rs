use std::ops::Deref;

use sanctum_router_core::{
    sanctum_reserve_core::{
        self, stake_account_record_seeds, Fee, FeeEnum, Pool, ProtocolFee, ProtocolFeeRatios,
        Rational,
    },
    ReserveDepositStakeQuoter, ReserveDepositStakeSufAccs,
};

use crate::{DepositStake, DepositStakeAddrs};

#[derive(Debug, Clone, PartialEq)]
pub struct ReserveRouter<F> {
    pub fee_account: FeeEnum,
    pub protocol_fee_dest: [u8; 32],
    pub protocol_fee_ratios: ProtocolFeeRatios,
    pub pool_incoming_stake: u64,
    pub pool_sol_reserves: u64,
    pub find_pda: F,
}

impl<F> ReserveRouter<F> {
    #[inline]
    pub const fn new(
        fee: Fee,
        pf: &ProtocolFee,
        pool: &Pool,
        pool_sol_reserves: u64,
        find_pda: F,
    ) -> Self {
        let mut res = Self {
            pool_sol_reserves,
            find_pda,
            // fillers to be replaced immediately
            fee_account: FeeEnum::Flat(Rational::ZERO),
            protocol_fee_dest: [0u8; 32],
            protocol_fee_ratios: ProtocolFeeRatios {
                fee_ratio: Rational::ZERO,
                referrer_fee_ratio: Rational::ZERO,
            },
            pool_incoming_stake: 0,
        };

        res.update_fee(fee);
        res.update_protocol_fee(pf);
        res.update_pool(pool);

        res
    }

    #[inline]
    pub const fn update_fee(&mut self, fee: Fee) {
        self.fee_account = fee.0;
    }

    #[inline]
    pub const fn update_protocol_fee(&mut self, pf: &ProtocolFee) {
        self.protocol_fee_dest = pf.destination;
        self.protocol_fee_ratios = pf.fee_ratios();
    }

    #[inline]
    pub const fn update_pool(&mut self, pool: &Pool) {
        self.pool_incoming_stake = pool.incoming_stake;
    }

    #[inline]
    pub const fn reserve_deposit_stake_quoter(&self) -> ReserveDepositStakeQuoter<'_> {
        ReserveDepositStakeQuoter {
            fee_account: &self.fee_account,
            protocol_fee: &self.protocol_fee_ratios,
            pool_incoming_stake: self.pool_incoming_stake,
            pool_sol_reserves: self.pool_sol_reserves,
        }
    }
}

impl<F: Fn(&[&[u8]], &[u8; 32]) -> Option<([u8; 32], u8)>> ReserveRouter<F> {
    #[inline]
    pub fn reserve_deposit_stake_suf_accs(
        &self,
        stake_account_addr: &[u8; 32],
    ) -> Option<ReserveDepositStakeSufAccs> {
        let (s1, s2) = stake_account_record_seeds(&sanctum_reserve_core::POOL, stake_account_addr);
        let find_pda = &self.find_pda;
        let (stake_acc_record_addr, _) = find_pda(
            &[s1.as_slice(), s2.as_slice()],
            &sanctum_reserve_core::UNSTAKE_PROGRAM,
        )?;
        Some(ReserveDepositStakeSufAccs {
            stake_acc_record_addr,
            protocol_fee_dest: self.protocol_fee_dest,
        })
    }
}

impl<F: Fn(&[&[u8]], &[u8; 32]) -> Option<([u8; 32], u8)>> DepositStake for ReserveRouter<F> {
    type Quoter<'a>
        = ReserveDepositStakeQuoter<'a>
    where
        F: 'a;

    type SufAccs<'a>
        = ReserveDepositStakeSufAccs
    where
        F: 'a;

    #[inline]
    fn deposit_stake_quoter(&self) -> Self::Quoter<'_> {
        self.reserve_deposit_stake_quoter()
    }

    #[inline]
    fn deposit_stake_suf_accs(
        &self,
        DepositStakeAddrs {
            stake: stake_addr, ..
        }: &DepositStakeAddrs,
    ) -> Option<Self::SufAccs<'_>> {
        self.reserve_deposit_stake_suf_accs(stake_addr)
    }
}

/// [`ReserveRouter`] post-prefund.
///
/// Quotes based on state of reserve after
/// a stake-acc exemption prefund operation
#[derive(Debug, Clone, PartialEq)]
pub struct ReserveRouterPpf<F>(pub ReserveRouter<F>);

impl<F> Deref for ReserveRouterPpf<F> {
    type Target = ReserveRouter<F>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<F: Fn(&[&[u8]], &[u8; 32]) -> Option<([u8; 32], u8)>> DepositStake for ReserveRouterPpf<F> {
    type Quoter<'a>
        = ReserveDepositStakeQuoter<'a>
    where
        F: 'a;

    type SufAccs<'a>
        = ReserveDepositStakeSufAccs
    where
        F: 'a;

    #[inline]
    fn deposit_stake_quoter(&self) -> Self::Quoter<'_> {
        let quoter = self.0.deposit_stake_quoter();
        quoter.after_prefund().unwrap_or(ReserveDepositStakeQuoter {
            // if reserve is unable to service a prefund op
            // then it shouldnt be able to service any other
            // stake deposits
            pool_sol_reserves: 0,
            ..quoter
        })
    }

    #[inline]
    fn deposit_stake_suf_accs(&self, dsa: &DepositStakeAddrs) -> Option<Self::SufAccs<'_>> {
        self.0.deposit_stake_suf_accs(dsa)
    }
}

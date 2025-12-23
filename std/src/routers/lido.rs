use sanctum_router_core::{
    solido_legacy_core::{self, validator_stake_seeds, ExchangeRate, Lido, Validator},
    LidoWithdrawStakeQuoter, LidoWithdrawStakeSufAccs,
};

use crate::WithdrawStake;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LidoRouterValData {
    pub vote: [u8; 32],
    pub effective_stake_balance: u64,
    pub stake_seeds_begin: u64,
}

impl LidoRouterValData {
    pub const ZEROS: Self = Self {
        vote: [0; 32],
        effective_stake_balance: 0,
        stake_seeds_begin: 0,
    };
}

impl Default for LidoRouterValData {
    #[inline]
    fn default() -> Self {
        Self::ZEROS
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LidoRouter<F> {
    pub validator_list_addr: [u8; 32],
    pub exchange_rate: ExchangeRate,
    pub curr_epoch: u64,

    /// None if lido pool no longer has any validators.
    ///
    /// Lido only allows withdrawing from max stake validator,
    /// so we only need to store max stake validator's data
    pub largest_stake: Option<LidoRouterValData>,

    pub find_pda: F,
}

impl<F> LidoRouter<F> {
    #[inline]
    pub fn new(state: &Lido, vlist: &[Validator], curr_epoch: u64, find_pda: F) -> Self {
        let mut res = Self {
            validator_list_addr: state.validator_list,
            exchange_rate: Default::default(),
            largest_stake: None,
            curr_epoch,
            find_pda,
        };
        res.update_exchange_rate(state);
        res.update_largest_stake(vlist);
        res
    }

    #[inline]
    pub const fn update_exchange_rate(&mut self, state: &Lido) {
        self.exchange_rate = state.exchange_rate;
    }

    #[inline]
    pub fn update_largest_stake(&mut self, vlist: &[Validator]) {
        let largest_stake = vlist
            .iter()
            .max_by_key(|v| v.effective_stake_balance())
            .map(|v| LidoRouterValData {
                vote: *v.vote_account_address(),
                effective_stake_balance: v.effective_stake_balance(),
                stake_seeds_begin: v.stake_seeds().begin(),
            });
        self.largest_stake = largest_stake;
    }

    #[inline]
    pub const fn lido_withdraw_stake_quoter(&self) -> LidoWithdrawStakeQuoter<'_> {
        let largest_stake = match self.largest_stake.as_ref() {
            Some(x) => x,
            None => &LidoRouterValData::ZEROS,
        };
        LidoWithdrawStakeQuoter {
            exchange_rate: &self.exchange_rate,
            curr_epoch: self.curr_epoch,
            largest_stake_vote: &largest_stake.vote,
            largest_stake_effective_stake_balance: largest_stake.effective_stake_balance,
        }
    }
}

impl<F: Fn(&[&[u8]], &[u8; 32]) -> Option<([u8; 32], u8)>> LidoRouter<F> {
    #[inline]
    pub fn lido_withdraw_stake_suf_accs(&self) -> Option<LidoWithdrawStakeSufAccs<'_>> {
        let largest_stake = self.largest_stake.as_ref()?;
        let (s1, s2, s3, s4) =
            validator_stake_seeds(&largest_stake.vote, largest_stake.stake_seeds_begin);
        let find_pda = &self.find_pda;
        let (stake_to_split, _) = find_pda(
            &[s1.as_slice(), s2.as_slice(), s3.as_slice(), s4.as_slice()],
            &solido_legacy_core::PROGRAM_ID,
        )?;
        Some(LidoWithdrawStakeSufAccs {
            validator_list_addr: &self.validator_list_addr,
            largest_stake_vote: &largest_stake.vote,
            stake_to_split,
        })
    }
}

impl<F: Fn(&[&[u8]], &[u8; 32]) -> Option<([u8; 32], u8)>> WithdrawStake for LidoRouter<F> {
    type Quoter<'a>
        = LidoWithdrawStakeQuoter<'a>
    where
        F: 'a;

    type SufAccs<'a>
        = LidoWithdrawStakeSufAccs<'a>
    where
        F: 'a;

    #[inline]
    fn withdraw_stake_quoter(&self) -> Self::Quoter<'_> {
        self.lido_withdraw_stake_quoter()
    }

    #[inline]
    fn withdraw_stake_suf_accs(&self, _vote: &[u8; 32]) -> Option<Self::SufAccs<'_>> {
        self.lido_withdraw_stake_suf_accs()
    }
}

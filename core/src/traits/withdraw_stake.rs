use sanctum_reserve_core::{FeeEnum, PoolBalance};
use sanctum_spl_stake_pool_core::STAKE_ACCOUNT_RENT_EXEMPT_LAMPORTS;

use crate::{
    reserves_has_enough_for_slumdog, slumdog_target_lamports, Prefund, StakeAccountLamports,
    WithdrawStakeQuote,
};

pub trait WithdrawStake {
    type Accs: AsRef<[[u8; 32]]>;
    type AccFlags: AsRef<[bool]>;

    fn get_withdraw_stake_quote(&self, pool_tokens: u64) -> Option<WithdrawStakeQuote>;

    /// The default impl here assumes the program does not fund rent-exemption for the
    /// destination stake account that is split to during withdrawal.
    /// (get_withdraw_stake_quote()'s returned quote.out.unstaked = 0)
    fn get_prefund_withdraw_stake_quote(
        &self,
        pool_tokens: u64,
        reserves_balance: &PoolBalance,
        reserves_fee: &FeeEnum,
    ) -> Option<Prefund<WithdrawStakeQuote>> {
        let WithdrawStakeQuote {
            inp,
            out: StakeAccountLamports { staked, .. },
            fee,
        } = self.get_withdraw_stake_quote(pool_tokens)?;
        if !reserves_has_enough_for_slumdog(reserves_balance) {
            // NotEnoughLiquidity
            return None;
        }
        // amount of active stake that will be split
        // from the withdrawn stake account to slumdog
        let prefund_fee = slumdog_target_lamports(reserves_balance, reserves_fee)?
            .saturating_sub(STAKE_ACCOUNT_RENT_EXEMPT_LAMPORTS);
        Some(Prefund {
            quote: WithdrawStakeQuote {
                inp,
                out: StakeAccountLamports {
                    // return None if original quote does not give enough
                    // sol to repay prefund flash loan
                    staked: staked.checked_sub(prefund_fee)?,
                    unstaked: STAKE_ACCOUNT_RENT_EXEMPT_LAMPORTS,
                },
                fee,
            },
            prefund_fee,
        })
    }

    /// Returned array must have `length = self.suffix_accounts_len()`
    fn suffix_accounts(&self) -> Self::Accs;

    /// Returned array must have `length = self.suffix_accounts_len()`
    fn suffix_is_signer(&self) -> Self::AccFlags;

    /// Returned array must have `length = self.suffix_accounts_len()`
    fn suffix_is_writable(&self) -> Self::AccFlags;
}

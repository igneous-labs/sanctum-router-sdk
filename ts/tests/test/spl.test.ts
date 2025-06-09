import { describe, it } from "vitest";
import {
  depositSolFixturesTest,
  depositStakeFixturesTest,
  PICOSOL_MINT,
  testFixturesStakeAcc,
  withdrawSolFixturesTest,
} from "../utils";

describe("SPL Test", async () => {
  it("spl-stake-pool-deposit-sol", async () => {
    await depositSolFixturesTest(1000000n, PICOSOL_MINT, {
      inp: "spl-signer-wsol-token",
      out: "signer-pico-token",
    });
  });

  it("spl-stake-pool-withdraw-sol", async () => {
    await withdrawSolFixturesTest(1000000n, PICOSOL_MINT, {
      inp: "signer-pico-token",
      out: "spl-signer-wsol-token",
    });
  });

  it("spl-stake-pool-deposit-stake", async () => {
    const stakeAccName = "picosol-deposit-stake";
    const { unstakedLamports, stakedLamports } =
      testFixturesStakeAcc(stakeAccName);
    // Need to quote as if the stake account to be deposited has
    // 0 staked sol because we are currently in epoch 0 so this
    // stake account and the pool's vsa are actually activating,
    // not active, so only sol deposit fees are applied to the
    // full stake account's balance
    const override = {
      staked: 0n,
      unstaked: unstakedLamports + stakedLamports,
    };

    await depositStakeFixturesTest(
      PICOSOL_MINT,
      {
        inp: stakeAccName,
        out: "signer-pico-token",
      },
      override
    );
  });
});

import { describe, it } from "vitest";
import {
  depositSolFixturesTest,
  depositStakeFixturesTest,
  NATIVE_MINT,
  PICOSOL_MINT,
  prefundSwapViaStakeFixturesTest,
  prefundWithdrawStakeFixturesTest,
  withdrawSolFixturesTest,
} from "../utils";

const PICOSOL_TOKEN_ACC_NAME = "signer-picosol-token";

describe("SPL Test", async () => {
  // DepositSol
  it("spl-stake-pool-deposit-sol", async () => {
    await depositSolFixturesTest(1000000n, PICOSOL_MINT, {
      inp: "spl-signer-wsol-token",
      out: PICOSOL_TOKEN_ACC_NAME,
    });
  });

  // WithdrawSol
  it("spl-stake-pool-withdraw-sol", async () => {
    await withdrawSolFixturesTest(1000000n, PICOSOL_MINT, {
      inp: PICOSOL_TOKEN_ACC_NAME,
      out: "spl-signer-wsol-token",
    });
  });

  // DepositStake
  it("spl-stake-pool-deposit-stake", async () => {
    await depositStakeFixturesTest(PICOSOL_MINT, {
      inp: "picosol-deposit-stake",
      out: PICOSOL_TOKEN_ACC_NAME,
    });
  });

  // PrefundWithdrawStake
  it("spl-stake-pool-prefund-withdraw-stake", async () => {
    await prefundWithdrawStakeFixturesTest(
      1_000_000_000n,
      PICOSOL_MINT,
      PICOSOL_TOKEN_ACC_NAME
    );
  });

  // PrefundSwapViaStake

  it("spl-stake-pool-prefund-swap-via-stake-picosol-into-reserve", async () => {
    await prefundSwapViaStakeFixturesTest(
      1_000_000_000n,
      {
        inp: PICOSOL_MINT,
        out: NATIVE_MINT,
      },
      {
        inp: PICOSOL_TOKEN_ACC_NAME,
        out: "reserve-signer-wsol-token",
      }
    );
  });
});

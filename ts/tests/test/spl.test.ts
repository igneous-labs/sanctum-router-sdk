import { describe, it } from "vitest";
import {
  depositSolFixturesTest,
  depositStakeFixturesTest,
  PICOSOL_MINT,
  prefundWithdrawStakeFixturesTest,
  withdrawSolFixturesTest,
} from "../utils";

describe("SPL Test", async () => {
  it("spl-stake-pool-deposit-sol", async () => {
    await depositSolFixturesTest(1000000n, PICOSOL_MINT, {
      inp: "spl-signer-wsol-token",
      out: "signer-picosol-token",
    });
  });

  it("spl-stake-pool-withdraw-sol", async () => {
    await withdrawSolFixturesTest(1000000n, PICOSOL_MINT, {
      inp: "signer-picosol-token",
      out: "spl-signer-wsol-token",
    });
  });

  it("spl-stake-pool-deposit-stake", async () => {
    await depositStakeFixturesTest(PICOSOL_MINT, {
      inp: "picosol-deposit-stake",
      out: "signer-picosol-token",
    });
  });

  it("spl-stake-pool-prefund-withdraw-stake", async () => {
    await prefundWithdrawStakeFixturesTest(
      1_000_000_000n,
      PICOSOL_MINT,
      "signer-picosol-token"
    );
  });
});

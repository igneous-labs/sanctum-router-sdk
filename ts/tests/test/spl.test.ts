import { describe, it } from "vitest";
import {
  depositSolFixturesTest,
  depositStakeFixturesTest,
  PICOSOL_MINT,
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
    await depositStakeFixturesTest(PICOSOL_MINT, {
      inp: "picosol-deposit-stake",
      out: "signer-pico-token",
    });
  });
});

import { describe, it } from "vitest";
import {
  depositSolFixturesTest,
  depositStakeFixturesTest,
  MSOL_MINT,
} from "../utils";

describe("Marinade Test", async () => {
  it("marinade-deposit-sol", async () => {
    await depositSolFixturesTest(1000000n, MSOL_MINT, {
      inp: "marinade-signer-wsol-token",
      out: "signer-msol-token",
    });
  });

  it("marinade-deposit-stake", async () => {
    await depositStakeFixturesTest(MSOL_MINT, {
      inp: "marinade_stake_account",
      out: "signer-msol-token",
    });
  });
});

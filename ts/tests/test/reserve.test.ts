import { describe, it } from "vitest";
import { depositStakeFixturesTest, NATIVE_MINT } from "../utils";

describe("Reserve Test", async () => {
  // DepositStake
  it("reserve-deposit-stake", async () => {
    await depositStakeFixturesTest(NATIVE_MINT, {
      inp: "reserve-deposit-stake",
      out: "reserve-signer-wsol-token",
    });
  });
});

import { describe, it } from "vitest";
import { depositStakeFixturesTest } from "../utils";

describe("Reserve Test", async () => {
  // DepositStake
  it("reserve-deposit-stake", async () => {
    await depositStakeFixturesTest({
      inp: "reserve-deposit-stake",
      out: "reserve-signer-wsol-token",
    });
  });
});

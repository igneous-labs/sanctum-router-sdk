import { describe, it } from "vitest";
import { depositStakeFixturesTest, NATIVE_MINT } from "../utils";

describe("Reserve Test", async () => {
  it("reserve-deposit-stake", async () => {
    await depositStakeFixturesTest(NATIVE_MINT, {
      inp: "reserve-stake-account",
      out: "reserve-signer-wsol-token",
    });
  });
});

import { describe, it } from "vitest";
import { prefundWithdrawStakeFixturesTest, STSOL_MINT } from "../utils";

describe("Lido Test", async () => {
  it("lido-prefund-withraw-stake", async () => {
    await prefundWithdrawStakeFixturesTest(
      1_000_000_000n,
      STSOL_MINT,
      "signer-stsol-token"
    );
  });
});

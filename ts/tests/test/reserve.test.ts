import { describe, expect, it } from "vitest";
import {
  depositStakeFixturesTest,
  localRpc,
  NATIVE_MINT,
  parseRouterErr,
  PICO_VOTE_ACC,
  routerForSwaps,
  STAKE_ACCOUNT_RENT_EXEMPT_LAMPORTS,
} from "../utils";
import { quoteDepositStake } from "@sanctumso/sanctum-router";

describe("Reserve Test", async () => {
  // DepositStake
  it("reserve-deposit-stake", async () => {
    await depositStakeFixturesTest({
      inp: "reserve-deposit-stake",
      out: "signer-wsol-token",
    });
  });

  it("reserve-deposit-stake-fails-withdrawal-too-large", async () => {
    const rpc = localRpc();
    const router = await routerForSwaps(rpc, [
      { swap: "depositStake", out: NATIVE_MINT },
    ]);
    try {
      quoteDepositStake(router, {
        vote: PICO_VOTE_ACC,
        inp: {
          // a very large amount
          staked: 1_000_000_000_000_000_000n,
          unstaked: STAKE_ACCOUNT_RENT_EXEMPT_LAMPORTS,
        },
        out: NATIVE_MINT,
      });
      expect.fail("should have thrown");
    } catch (e) {
      expect(e).toSatisfy((e) => {
        const [code] = parseRouterErr(e);
        return code === "PoolErr";
      });
    }
  });
});

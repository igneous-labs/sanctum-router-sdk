import { describe, it } from "vitest";
import {
  depositStakeFixturesTest,
  expectRouterErr,
  localRpc,
  NATIVE_MINT,
  PICO_VOTE_ACC,
  routerForSwaps,
  STAKE_ACCOUNT_RENT_EXEMPT_LAMPORTS,
} from "../utils";
import { quoteDepositStake } from "@sanctumso/sanctum-router";

describe("Reserve Test", async () => {
  // DepositStake
  it("reserve-deposit-stake-small", async () => {
    await depositStakeFixturesTest({
      inp: "reserve-deposit-stake-small",
      out: "signer-wsol-token",
    });
  });

  it("reserve-deposit-stake-large", async () => {
    await depositStakeFixturesTest({
      inp: "reserve-deposit-stake-large",
      out: "signer-wsol-token",
    });
  });

  it("reserve-deposit-stake-fails-withdrawal-too-large", async () => {
    const rpc = localRpc();
    const router = await routerForSwaps(rpc, [
      { swap: "depositStake", out: NATIVE_MINT },
    ]);
    await expectRouterErr(
      () =>
        quoteDepositStake(router, {
          vote: PICO_VOTE_ACC,
          inp: {
            staked: 1_000_000_000_000_000_000n,
            unstaked: STAKE_ACCOUNT_RENT_EXEMPT_LAMPORTS,
          },
          out: NATIVE_MINT,
        }),
      "SizeTooLargeErr:ReserveError::NotEnoughLiquidity",
    );
  });
});

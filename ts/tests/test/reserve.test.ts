import { describe, expect, it } from "vitest";
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
    const q = await depositStakeFixturesTest({
      inp: "reserve-deposit-stake-small",
      out: "signer-wsol-token",
    });
    expect(q).toMatchInlineSnapshot(`
      {
        "quote": {
          "fee": 4619n,
          "inp": {
            "staked": 2287499n,
            "unstaked": 2282880n,
          },
          "out": 4565760n,
          "vote": "3xjfK9C9YNcta8MvK1US4sQ3bc6DEjoJoR3qLExGf9xE",
        },
        "routerFee": 0n,
      }
    `);
  });

  it("reserve-deposit-stake-large", async () => {
    const q = await depositStakeFixturesTest({
      inp: "reserve-deposit-stake-large",
      out: "signer-wsol-token",
    });
    expect(q).toMatchInlineSnapshot(`
      {
        "quote": {
          "fee": 1049843187n,
          "inp": {
            "staked": 888437281569n,
            "unstaked": 2282880n,
          },
          "out": 887389721262n,
          "vote": "3xjfK9C9YNcta8MvK1US4sQ3bc6DEjoJoR3qLExGf9xE",
        },
        "routerFee": 0n,
      }
    `);
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

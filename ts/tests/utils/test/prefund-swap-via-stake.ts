import {
  prefundSwapViaStakeIx,
  quotePrefundSwapViaStake,
  type SwapViaStakeSwapParams,
} from "@sanctumso/sanctum-router";
import { mapTup } from "../ops";
import { routerForSwaps } from "../router";
import { testFixturesTokenAcc } from "../token";
import { localRpc } from "../rpc";
import { simTokenSwapAssertQuoteMatches } from "./swap";

// Assume bridge stake seed 0 is always unused
const BRIDGE_STAKE_SEED = 0;

export async function prefundSwapViaStakeFixturesTest(
  amt: bigint,
  tokenAccFixtures: { inp: string; out: string }
) {
  const { inp: inpTokenAccName, out: outTokenAccName } = tokenAccFixtures;
  const [
    { addr: inpTokenAcc, owner: inpTokenAccOwner, mint: inpMint },
    { addr: outTokenAcc, mint: outMint },
  ] = mapTup([inpTokenAccName, outTokenAccName], testFixturesTokenAcc);
  const rpc = localRpc();

  const router = await routerForSwaps(rpc, [
    { prefundSwapViaStake: { inp: inpMint, out: outMint } },
  ]);

  const {
    quote: { quote, routerFee },
  } = quotePrefundSwapViaStake(router, {
    amt,
    out: outMint,
    inp: inpMint,
  });
  const params: SwapViaStakeSwapParams = {
    amt,
    inp: inpMint,
    out: outMint,
    signerInp: inpTokenAcc,
    signerOut: outTokenAcc,
    signer: inpTokenAccOwner,
    bridgeStakeSeed: BRIDGE_STAKE_SEED,
  };
  const ix = prefundSwapViaStakeIx(router, params);

  // TODO: replace this fn with a custom fn that further
  // asserts correctness of prefundFee and inpFee
  await simTokenSwapAssertQuoteMatches(
    rpc,
    { quote: { ...quote, fee: quote.outFee }, routerFee },
    params,
    ix
  );
}

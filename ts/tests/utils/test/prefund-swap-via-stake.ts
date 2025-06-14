import {
  prefundSwapViaStakeIx,
  quotePrefundSwapViaStake,
  type SwapViaStakeSwapParams,
} from "@sanctumso/sanctum-router";
import { mapTup } from "../ops";
import { routerForMints } from "../router";
import { NATIVE_MINT, testFixturesTokenAcc } from "../token";
import { localRpc } from "../rpc";
import { simTokenSwapAssertQuoteMatches } from "./swap";

// Assume bridge stake seed 0 is always unused
const BRIDGE_STAKE_SEED = 0;

export async function prefundSwapViaStakeFixturesTest(
  amt: bigint,
  mints: { inp: string; out: string },
  tokenAccFixtures: { inp: string; out: string }
) {
  const { inp: inpMint, out: outMint } = mints;
  const { inp: inpTokenAccName, out: outTokenAccName } = tokenAccFixtures;
  const [
    { addr: inpTokenAcc, owner: inpTokenAccOwner },
    { addr: outTokenAcc },
  ] = mapTup([inpTokenAccName, outTokenAccName], testFixturesTokenAcc);
  const rpc = localRpc();

  // TODO: this API is very ass bec we need to remember to include NATIVE_MINT
  // as part of the array or else we will get ReserveError(NotEnoughLiquidity)
  // when quoting because reserves' sol reserves acc is not fetched.
  //
  // GH issue #18 fine-grained updates should aim to solve this
  const router = await routerForMints(rpc, [inpMint, outMint, NATIVE_MINT]);

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

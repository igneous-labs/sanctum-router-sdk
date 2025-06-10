import {
  getWithdrawSolIx,
  getWithdrawSolQuote,
  type TokenSwapParams,
} from "@sanctumso/sanctum-router";
import { mapTup } from "../ops";
import { routerForMints } from "../router";
import { localRpc } from "../rpc";
import { NATIVE_MINT, testFixturesTokenAcc } from "../token";
import { simTokenSwapAssertQuoteMatches } from "./swap";

export async function withdrawSolFixturesTest(
  amt: bigint,
  mint: string,
  tokenAccFixtures: { inp: string; out: string }
) {
  const { inp: inpTokenAccName, out: outTokenAccName } = tokenAccFixtures;
  const [
    { addr: inpTokenAcc, owner: inpTokenAccOwner },
    { addr: outTokenAcc },
  ] = mapTup([inpTokenAccName, outTokenAccName], testFixturesTokenAcc);
  const rpc = localRpc();
  const router = await routerForMints(rpc, [mint]);

  const quote = getWithdrawSolQuote(router, {
    amt,
    inpMint: mint,
    outMint: NATIVE_MINT,
  })!;
  const params: TokenSwapParams = {
    amt,
    signerInp: inpTokenAcc,
    signerOut: outTokenAcc,
    signer: inpTokenAccOwner,
    inp: mint,
    out: NATIVE_MINT,
  };
  const ix = getWithdrawSolIx(router, params);

  await simTokenSwapAssertQuoteMatches(rpc, quote, params, ix);
}

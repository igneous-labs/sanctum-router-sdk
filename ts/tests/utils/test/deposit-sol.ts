import {
  depositSolIx,
  quoteDepositSol,
  type TokenSwapParams,
} from "@sanctumso/sanctum-router";
import { mapTup } from "../ops";
import { routerForMints } from "../router";
import { NATIVE_MINT, testFixturesTokenAcc } from "../token";
import { localRpc } from "../rpc";
import { simTokenSwapAssertQuoteMatches } from "./swap";

export async function depositSolFixturesTest(
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

  const quote = quoteDepositSol(router, {
    amt,
    out: mint,
  });
  const params: TokenSwapParams = {
    amt,
    inp: NATIVE_MINT,
    out: mint,
    signerInp: inpTokenAcc,
    signerOut: outTokenAcc,
    signer: inpTokenAccOwner,
  };
  const ix = depositSolIx(router, params);

  await simTokenSwapAssertQuoteMatches(rpc, quote, params, ix);
}

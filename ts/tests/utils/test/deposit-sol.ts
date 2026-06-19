import {
  depositSolIx,
  quoteDepositSol,
  type DepositSolSwapParams,
  type TokenQuoteWithRouterFee,
} from "@sanctumso/sanctum-router";
import { mapTup } from "../ops";
import { routerForSwaps } from "../router";
import { NATIVE_MINT, testFixturesTokenAcc } from "../token";
import { localRpc } from "../rpc";
import { simTokenSwapAssertQuoteMatches } from "./swap";

export async function depositSolFixturesTest(
  amt: bigint,
  tokenAccFixtures: { inp: string; out: string },
): Promise<TokenQuoteWithRouterFee> {
  const { inp: inpTokenAccName, out: outTokenAccName } = tokenAccFixtures;
  const [
    { addr: inpTokenAcc, owner: inpTokenAccOwner },
    { addr: outTokenAcc, mint: outMint },
  ] = mapTup([inpTokenAccName, outTokenAccName], testFixturesTokenAcc);
  const rpc = localRpc();
  const router = await routerForSwaps(rpc, [
    { swap: "depositSol", out: outMint },
  ]);

  const res = quoteDepositSol(router, {
    amt,
    out: outMint,
  });
  const params: DepositSolSwapParams = {
    amt,
    out: outMint,
    signerInp: inpTokenAcc,
    signerOut: outTokenAcc,
    signer: inpTokenAccOwner,
  };
  const ix = depositSolIx(router, params);

  await simTokenSwapAssertQuoteMatches(
    rpc,
    res,
    { ...params, inp: NATIVE_MINT },
    ix,
  );

  return res;
}

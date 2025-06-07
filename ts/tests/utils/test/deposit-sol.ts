import { getDepositSolIx, getDepositSolQuote } from "@sanctumso/sanctum-router";
import { mapTup } from "../ops";
import { routerForMints } from "../router";
import { NATIVE_MINT, testFixturesTokenAcc } from "../token";
import { localRpc } from "../rpc";
import { simTokenSwapAssertQuoteMatches } from "./swap";

export async function depositSolFixturesTest(
  amount: bigint,
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

  const quote = getDepositSolQuote(router, {
    amount,
    inputMint: NATIVE_MINT,
    outputMint: mint,
  })!;
  const params = {
    amount,
    sourceTokenAccount: inpTokenAcc,
    destinationTokenAccount: outTokenAcc,
    tokenTransferAuthority: inpTokenAccOwner,
    source: NATIVE_MINT,
    destinationMint: mint,
  };
  const ix = getDepositSolIx(router, params);

  await simTokenSwapAssertQuoteMatches(rpc, quote, params, ix);
}

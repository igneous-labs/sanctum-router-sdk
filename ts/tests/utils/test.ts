import {
  getDepositSolIx,
  getDepositSolQuote,
  type Instruction,
  type SwapParams,
  type TokenQuote,
} from "@sanctumso/sanctum-router";
import {
  address,
  getBase64Encoder,
  type Rpc,
  type SolanaRpcApi,
} from "@solana/kit";
import { expect } from "vitest";
import { mapTup } from "./ops";
import { ixToSimTx } from "./tx";
import { NATIVE_MINT, testFixturesTokenAcc, tokenAccBalance } from "./token";
import { fetchAccountMap, localRpc } from "./rpc";
import { routerForMints } from "./router";
import { PICOSOL_ACCS } from "./spl";

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
  // TODO: picosol is currently the only SPL being tested, may need
  // to add to this list in the future.
  const spls = [PICOSOL_ACCS];
  const router = await routerForMints(rpc, spls, [mint]);

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

async function simTokenSwapAssertQuoteMatches(
  rpc: Rpc<SolanaRpcApi>,
  {
    inAmount,
    outAmount,
    // TODO: need to also assert that the router fee accounts received the correct amount of
    // fees but that would mean modifying the TokenQuote struct def to have fine-grained fee breakdowns
    // of stake pool fees + router fees
    feeAmount: _,
  }: TokenQuote,
  {
    amount,
    sourceTokenAccount,
    destinationTokenAccount,
    tokenTransferAuthority,
  }: SwapParams,
  ix: Instruction
) {
  expect(inAmount).toStrictEqual(amount);

  // `addresses` layout:
  // - sourceTokenAccount
  // - destinationTokenAccount
  const addresses = mapTup(
    [sourceTokenAccount, destinationTokenAccount],
    address
  );

  const befSwap = await fetchAccountMap(rpc, addresses);
  const [sourceTokenAccountBalanceBef, destinationTokenAccountBalanceBef] =
    mapTup(addresses, (addr) => tokenAccBalance(befSwap.get(addr)!.data));

  const tx = ixToSimTx(address(tokenTransferAuthority), ix);
  const {
    value: { err, accounts: aftSwap, logs },
  } = await rpc
    .simulateTransaction(tx, {
      accounts: {
        addresses,
        encoding: "base64",
      },
      encoding: "base64",
      sigVerify: false,
      replaceRecentBlockhash: true,
    })
    .send();

  const debugMsg = `tx: ${tx}\nlogs:\n` + (logs ?? []).join("\n") + "\n";
  expect(err, debugMsg).toBeNull();

  const [sourceTokenAccountBalanceAft, destinationTokenAccountBalanceAft] =
    mapTup([0, 1], (i) =>
      tokenAccBalance(
        new Uint8Array(getBase64Encoder().encode(aftSwap[i]!.data[0]))
      )
    );

  expect(sourceTokenAccountBalanceBef - sourceTokenAccountBalanceAft).toEqual(
    inAmount
  );
  expect(
    destinationTokenAccountBalanceAft - destinationTokenAccountBalanceBef
  ).toEqual(outAmount);
}

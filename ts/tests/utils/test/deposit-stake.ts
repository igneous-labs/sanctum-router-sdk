import {
  findFeeTokenAccountPda,
  getDepositStakeIx,
  getDepositStakeQuote,
  type DepositStakeQuoteWithRouterFee,
  type Instruction,
  type SwapParams,
} from "@sanctumso/sanctum-router";
import { routerForMints } from "../router";
import { fetchAccountMap, localRpc } from "../rpc";
import { testFixturesStakeAcc } from "../stake";
import { testFixturesTokenAcc, tokenAccBalance } from "../token";
import {
  address,
  getBase64Encoder,
  type Rpc,
  type SolanaRpcApi,
} from "@solana/kit";
import { mapTup } from "../ops";
import { ixToSimTx } from "../tx";
import { txSimParams } from "./common";
import { expect } from "vitest";

export async function depositStakeFixturesTest(
  mint: string,
  { inp: inpStakeAccName, out: outTokenAccName }: { inp: string; out: string }
) {
  const { addr: outTokenAcc } = testFixturesTokenAcc(outTokenAccName);
  const {
    addr: inpStakeAcc,
    vote,
    stakedLamports,
    unstakedLamports,
    withdrawer,
  } = testFixturesStakeAcc(inpStakeAccName);
  const rpc = localRpc();
  const router = await routerForMints(rpc, [mint]);

  const stakeAccountLamports = {
    staked: stakedLamports,
    unstaked: unstakedLamports,
  };
  const quote = getDepositStakeQuote(router, {
    validatorVote: vote,
    outputMint: mint,
    stakeAccountLamports,
  })!;
  const params = {
    destinationMint: mint,
    sourceTokenAccount: inpStakeAcc,
    destinationTokenAccount: outTokenAcc,
    tokenTransferAuthority: withdrawer,
    source: vote,
    // dont cares
    amount: 0n,
  };

  const ix = getDepositStakeIx(router, params);

  await simDepositStakeAssertQuoteMatches(rpc, quote, params, ix);
}

async function simDepositStakeAssertQuoteMatches(
  rpc: Rpc<SolanaRpcApi>,
  { quote: { tokensOut }, routerFee }: DepositStakeQuoteWithRouterFee,
  {
    destinationTokenAccount,
    tokenTransferAuthority,
    destinationMint,
  }: SwapParams,
  ix: Instruction
) {
  // `addresses` layout:
  // - destinationTokenAccount
  // - router fee token account
  const addresses = [
    address(destinationTokenAccount),
    address(findFeeTokenAccountPda(destinationMint)[0]),
  ];

  const befSwap = await fetchAccountMap(rpc, addresses);
  const [destinationTokenAccountBalanceBef, feeTokenAccountBalanceBef] = mapTup(
    addresses,
    (addr) => tokenAccBalance(befSwap.get(addr)!.data)
  );

  const tx = ixToSimTx(address(tokenTransferAuthority), ix);
  const {
    value: { err, accounts: aftSwap, logs },
  } = await rpc.simulateTransaction(tx, txSimParams(addresses)).send();

  const debugMsg = `tx: ${tx}\nlogs:\n` + (logs ?? []).join("\n") + "\n";
  expect(err, debugMsg).toBeNull();

  const [destinationTokenAccountBalanceAft, feeTokenAccountBalanceAft] = mapTup(
    [0, 1],
    (i) =>
      tokenAccBalance(
        new Uint8Array(getBase64Encoder().encode(aftSwap[i]!.data[0]))
      )
  );

  expect(
    destinationTokenAccountBalanceAft - destinationTokenAccountBalanceBef
  ).toEqual(tokensOut);
  expect(feeTokenAccountBalanceAft - feeTokenAccountBalanceBef).toEqual(
    routerFee
  );
}

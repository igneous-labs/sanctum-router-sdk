import {
  getDepositStakeIx,
  getDepositStakeQuote,
  type DepositStakeQuote,
  type Instruction,
  type StakeAccountLamports,
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
  {
    tokensOut,
    // TODO: need to also assert that the router fee accounts received the correct amount of
    // fees but that would mean modifying the TokenQuote struct def to have fine-grained fee breakdowns
    // of stake pool fees + router fees
    feeAmount: _,
  }: DepositStakeQuote,
  { destinationTokenAccount, tokenTransferAuthority }: SwapParams,
  ix: Instruction
) {
  // `addresses` layout:
  // - destinationTokenAccount
  const addresses = [address(destinationTokenAccount)];

  const befSwap = await fetchAccountMap(rpc, addresses);
  const [destinationTokenAccountBalanceBef] = mapTup(addresses, (addr) =>
    tokenAccBalance(befSwap.get(addr)!.data)
  );

  const tx = ixToSimTx(address(tokenTransferAuthority), ix);
  const {
    value: { err, accounts: aftSwap, logs },
  } = await rpc.simulateTransaction(tx, txSimParams(addresses)).send();

  const debugMsg = `tx: ${tx}\nlogs:\n` + (logs ?? []).join("\n") + "\n";
  expect(err, debugMsg).toBeNull();

  const [destinationTokenAccountBalanceAft] = mapTup([0], (i) =>
    tokenAccBalance(
      new Uint8Array(getBase64Encoder().encode(aftSwap[i]!.data[0]))
    )
  );

  expect(
    destinationTokenAccountBalanceAft - destinationTokenAccountBalanceBef
  ).toEqual(tokensOut);
}

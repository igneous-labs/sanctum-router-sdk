import {
  findFeeTokenAccountPda,
  depositStakeIx,
  quoteDepositStake,
  type DepositStakeQuoteWithRouterFee,
  type DepositStakeSwapParams,
  type Instruction,
} from "@sanctumso/sanctum-router";
import { routerForSwaps } from "../router";
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
import { ixToSimTx, txSimParams } from "../tx";
import { expect } from "vitest";

export async function depositStakeFixturesTest({
  inp: inpStakeAccName,
  out: outTokenAccName,
}: {
  inp: string;
  out: string;
}) {
  const { addr: outTokenAcc, mint: outMint } =
    testFixturesTokenAcc(outTokenAccName);
  const {
    addr: inpStakeAcc,
    vote,
    stakedLamports,
    unstakedLamports,
    withdrawer,
  } = testFixturesStakeAcc(inpStakeAccName);
  const rpc = localRpc();
  const router = await routerForSwaps(rpc, [
    { depositStake: { out: outMint } },
  ]);

  const inpStake = {
    staked: stakedLamports,
    unstaked: unstakedLamports,
  };
  const quote = quoteDepositStake(router, {
    vote,
    out: outMint,
    inp: inpStake,
  });
  const params: DepositStakeSwapParams = {
    inp: vote,
    out: outMint,
    signerInp: inpStakeAcc,
    signerOut: outTokenAcc,
    signer: withdrawer,
  };

  const ix = depositStakeIx(router, params);

  await simDepositStakeAssertQuoteMatches(rpc, quote, params, ix);
}

async function simDepositStakeAssertQuoteMatches(
  rpc: Rpc<SolanaRpcApi>,
  {
    quote: {
      out,
      // TODO: we might want to test that the collected fee matches too.
      // Probably just pass poolFeeTokenAcc as an arg to this fn,
      // and then assert balance changes match fee here
      fee: _,
    },
    routerFee,
  }: DepositStakeQuoteWithRouterFee,
  { out: outMint, signerOut, signer }: DepositStakeSwapParams,
  ix: Instruction
) {
  // `addresses` layout:
  // - signerOut
  // - router fee token account
  const addresses = [
    address(signerOut),
    address(findFeeTokenAccountPda(outMint)[0]),
  ];

  const befSwap = await fetchAccountMap(rpc, addresses);
  const [outTokenAccBalBef, feeTokenAccBalBef] = mapTup(addresses, (addr) =>
    tokenAccBalance(befSwap.get(addr)!.data)
  );

  const tx = ixToSimTx(address(signer), ix);
  const {
    value: { err, accounts: aftSwap, logs },
  } = await rpc.simulateTransaction(tx, txSimParams(addresses)).send();

  const debugMsg = `tx: ${tx}\nlogs:\n` + (logs ?? []).join("\n") + "\n";
  expect(err, debugMsg).toBeNull();

  const [outTokenAccBalAft, feeTokenAccBalAft] = mapTup([0, 1], (i) =>
    tokenAccBalance(
      new Uint8Array(getBase64Encoder().encode(aftSwap[i]!.data[0]))
    )
  );

  expect(outTokenAccBalAft - outTokenAccBalBef).toEqual(out);
  expect(feeTokenAccBalAft - feeTokenAccBalBef).toEqual(routerFee);
}

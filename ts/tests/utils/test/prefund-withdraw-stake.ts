import {
  createSlumdogStakeAddr,
  findBridgeStakeAccPda,
  prefundWithdrawStakeIx,
  quotePrefundWithdrawStake,
  type Instruction,
  type PrefundWithdrawStakeQuote,
  type WithdrawStakeSwapParams,
} from "@sanctumso/sanctum-router";
import { routerForSwaps } from "../router";
import { fetchAccountMap, localRpc } from "../rpc";
import { testFixturesTokenAcc, tokenAccBalance } from "../token";
import {
  address,
  getBase64Encoder,
  type Rpc,
  type SolanaRpcApi,
} from "@solana/kit";
import { ixToSimTx, txSimParams } from "../tx";
import { expect } from "vitest";
import {
  STAKE_ACCOUNT_RENT_EXEMPT_LAMPORTS,
  stakeAccStake,
  stakeAccVote,
} from "../stake";
import { mapTup } from "../ops";

// Assume bridge stake seed 0 is always unused
const BRIDGE_STAKE_SEED = 0;

export async function prefundWithdrawStakeFixturesTest(
  amt: bigint,
  inpTokenAccName: string,
  outVote?: string | undefined
) {
  const {
    addr: inpTokenAcc,
    owner: signer,
    mint: inpMint,
  } = testFixturesTokenAcc(inpTokenAccName);
  const rpc = localRpc();

  const router = await routerForSwaps(rpc, [
    { swap: "prefundWithdrawStake", inp: inpMint },
  ]);

  const quote = quotePrefundWithdrawStake(router, {
    amt,
    inp: inpMint,
    out: outVote,
  });
  const params: WithdrawStakeSwapParams = {
    amt,
    inp: inpMint,
    out: quote.quote.vote,
    signerInp: inpTokenAcc,
    bridgeStakeSeed: BRIDGE_STAKE_SEED,
    signer,
  };

  const ix = prefundWithdrawStakeIx(router, params);

  await simPrefundWithdrawStakeAssertQuoteMatches(rpc, quote, params, ix);
}

async function simPrefundWithdrawStakeAssertQuoteMatches(
  rpc: Rpc<SolanaRpcApi>,
  {
    quote: {
      vote,
      inp,
      out: { staked, unstaked },
      // TODO: we might want to test that the collected fee matches too.
      // Probably just pass poolFeeTokenAcc as an arg to this fn
      // and then assert balance changes match fee here
      fee: _f,
    },
    prefundFee,
  }: PrefundWithdrawStakeQuote,
  { signerInp, signer }: WithdrawStakeSwapParams,
  ix: Instruction
) {
  const bridgeStakeAddr = address(
    findBridgeStakeAccPda(signer, BRIDGE_STAKE_SEED)[0]
  );
  // `addresses` layout:
  // - signer input token acc
  // - output bridge stake acc
  // - slumdog bridge stake acc
  const addresses = [
    address(signerInp),
    bridgeStakeAddr,
    address(createSlumdogStakeAddr(bridgeStakeAddr)),
  ];
  // Need to omit nonexistent addrs else fetchAccountMap() throws
  const existingAddrs = [address(signerInp)];

  const befSwap = await fetchAccountMap(rpc, existingAddrs);
  const inpTokenAccBalBef = tokenAccBalance(befSwap.get(signerInp)!.data);

  const tx = ixToSimTx(address(signer), ix);
  const {
    value: { err, accounts: aftSwap, logs },
  } = await rpc.simulateTransaction(tx, txSimParams(addresses)).send();

  const debugMsg = `tx: ${tx}\nlogs:\n` + (logs ?? []).join("\n") + "\n";
  expect(err, debugMsg).toBeNull();

  const inpTokenAccBalAft = tokenAccBalance(
    new Uint8Array(getBase64Encoder().encode(aftSwap[0]!.data[0]))
  );
  expect(inpTokenAccBalBef - inpTokenAccBalAft).toEqual(inp);

  const [
    [bridgeStakeAccLamportsAft, bridgeStakeAccDataAft],
    [slumdogStakeAccLamportsAft, slumdogStakeAccDataAft],
  ] = mapTup([1, 2], (i) => {
    const stakeAccAft = aftSwap[i]!;
    return [
      stakeAccAft.lamports,
      getBase64Encoder().encode(stakeAccAft.data[0]),
    ] as const;
  });

  const bridgeStakeAccStakeAft = stakeAccStake(bridgeStakeAccDataAft);
  const bridgeStakeAccVoteAft = stakeAccVote(bridgeStakeAccDataAft);
  expect(bridgeStakeAccVoteAft).toEqual(vote);
  expect(bridgeStakeAccStakeAft).toEqual(staked);
  expect(bridgeStakeAccLamportsAft).toEqual(staked + unstaked);

  expect(
    slumdogStakeAccLamportsAft - STAKE_ACCOUNT_RENT_EXEMPT_LAMPORTS
  ).toEqual(prefundFee);
  expect(stakeAccStake(slumdogStakeAccDataAft)).toEqual(prefundFee);
}

import type { Instruction } from "@sanctumso/sanctum-router";
import {
  appendTransactionMessageInstructions,
  blockhash,
  compileTransaction,
  compressTransactionMessageUsingAddressLookupTables,
  createTransactionMessage,
  getAddressDecoder,
  getBase64EncodedWireTransaction,
  getBase64Encoder,
  pipe,
  setTransactionMessageFeePayer,
  setTransactionMessageLifetimeUsingBlockhash,
  type Address,
  type AddressesByLookupTableAddress,
  type Base64EncodedWireTransaction,
  type IInstruction,
} from "@solana/kit";
import { getSetComputeUnitLimitInstruction } from "@solana-program/compute-budget";
import { readTestFixturesJsonFile } from "./file";

const LUT_ADDRS_START_OFFSET = 56;

function readSrlut(): AddressesByLookupTableAddress {
  const acc = readTestFixturesJsonFile("srlut");
  const b64Enc = getBase64Encoder();
  const bytes = new Uint8Array(b64Enc.encode(acc.account.data[0]));
  const addrDec = getAddressDecoder();
  const addrs = [];
  for (let i = LUT_ADDRS_START_OFFSET; i < bytes.length; i += 32) {
    addrs.push(addrDec.decode(bytes, i));
  }
  return {
    [acc.pubkey]: addrs,
  };
}

export function ixToSimTx(
  payer: Address,
  ix: Instruction
): Base64EncodedWireTransaction {
  // Examples of very expensive transactions that require >200k default CUs
  // - (Prefund)SwapViaStake
  // - (Prefund)WithdrawStake for lido
  const cuLimitIx = getSetComputeUnitLimitInstruction({ units: 1_500_000 });

  return pipe(
    createTransactionMessage({ version: 0 }),
    (txm) =>
      appendTransactionMessageInstructions(
        [cuLimitIx, ix as unknown as IInstruction],
        txm
      ),
    (txm) => setTransactionMessageFeePayer(payer, txm),
    (txm) =>
      compressTransactionMessageUsingAddressLookupTables(txm, readSrlut()),
    (txm) =>
      setTransactionMessageLifetimeUsingBlockhash(
        {
          blockhash: blockhash("11111111111111111111111111111111"),
          lastValidBlockHeight: 0n,
        },
        txm
      ),
    compileTransaction,
    getBase64EncodedWireTransaction
  );
}

export function txSimParams(addresses: Address[]) {
  return {
    accounts: {
      addresses,
      encoding: "base64",
    },
    encoding: "base64",
    sigVerify: false,
    replaceRecentBlockhash: true,
  } as const;
}

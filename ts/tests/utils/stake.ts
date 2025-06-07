import {
  getAddressDecoder,
  getBase64Encoder,
  getU64Decoder,
  type Address,
  type ReadonlyUint8Array,
} from "@solana/kit";
import { readTestFixturesJsonFile } from "./file";

const STAKE_ACC_WITHDRAWER_OFFSET = 44;
const STAKE_ACC_VOTE_OFFSET = 124;
const STAKE_ACC_STAKE_OFFSET = 156;

export function stakeAccWithdrawer(accData: ReadonlyUint8Array): Address {
  return getAddressDecoder().decode(accData, STAKE_ACC_WITHDRAWER_OFFSET);
}

export function stakeAccVote(accData: ReadonlyUint8Array): Address {
  return getAddressDecoder().decode(accData, STAKE_ACC_VOTE_OFFSET);
}

export function stakeAccStake(accData: ReadonlyUint8Array): bigint {
  return getU64Decoder().decode(accData, STAKE_ACC_STAKE_OFFSET);
}

export function testFixturesStakeAcc(stakeAccFname: string): {
  addr: Address;
  vote: Address;
  stakedLamports: bigint;
  unstakedLamports: bigint;
  withdrawer: Address;
} {
  const {
    pubkey,
    account: {
      data: [data],
      lamports,
    },
  } = readTestFixturesJsonFile(stakeAccFname);
  const accData = getBase64Encoder().encode(data);
  const withdrawer = stakeAccWithdrawer(accData);
  const vote = stakeAccVote(accData);
  const stakedLamports = stakeAccStake(accData);
  const unstakedLamports = BigInt(lamports) - stakedLamports;

  if (unstakedLamports < 0) {
    throw new Error(
      `unstakedLamports < 0 for stake=${stakedLamports}, lamports=${lamports}`
    );
  }

  return {
    addr: pubkey,
    withdrawer,
    vote,
    stakedLamports,
    unstakedLamports,
  };
}

import {
  getAddressDecoder,
  getBase64Encoder,
  getU64Decoder,
  type Address,
  type ReadonlyUint8Array,
} from "@solana/kit";
import { readTestFixturesJsonFile } from "./file";
import { mapTup } from "./ops";
import { CURR_EPOCH } from "./consts";

const STAKE_ACC_WITHDRAWER_OFFSET = 44;
const STAKE_ACC_VOTE_OFFSET = 124;
export const STAKE_ACC_STAKE_OFFSET = 156;
export const STAKE_ACC_ACTIVATION_EPOCH_OFFSET = 164;
const STAKE_ACC_DEACTIVATION_EPOCH_OFFSET = 172;

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

  const stake = stakeAccStake(accData);
  const biLamports = BigInt(lamports);
  const [activationEpoch, deactivationEpoch] = mapTup(
    [STAKE_ACC_ACTIVATION_EPOCH_OFFSET, STAKE_ACC_DEACTIVATION_EPOCH_OFFSET],
    (offset) => getU64Decoder().decode(accData, offset)
  );

  let stakedLamports: bigint;
  let unstakedLamports: bigint;
  if (activationEpoch >= CURR_EPOCH || deactivationEpoch < CURR_EPOCH) {
    stakedLamports = 0n;
    unstakedLamports = biLamports;
  } else {
    stakedLamports = stake;
    unstakedLamports = biLamports - stakedLamports;
  }

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

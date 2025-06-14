/**
 * prints index of validator on list and exit 0 if a given vote account is part of a pool's validator list
 * prints `false` and exit 1 otherwise
 */

import {
  address,
  getAddressDecoder,
  getBase64Encoder,
  getU32Decoder,
  type Address,
} from "@solana/kit";
import { readTestFixturesJsonFile } from "../utils/file";
import { exit } from "process";

function main() {
  const [_node, _script, pool, voteStr] = process.argv;

  if (!pool || !voteStr) {
    console.log(
      "Usage: is-on-validator-list.ts <marinade | [spl-validator-list-test-fixtures-name]> <vote-addr>"
    );
    return;
  }

  const vote = address(voteStr);

  let idx: number | undefined;
  switch (pool) {
    case "marinade":
      idx = marinadeMain(vote);
      break;
    default:
      idx = splMain(vote, pool);
      break;
  }

  if (idx == null) {
    console.log(false);
    exit(1);
  }

  console.log(idx);
}

function marinadeMain(vote: Address) {
  const VALIDATOR_RECORDS_START = 8; // after 8-bytes discm
  const VALIDATOR_RECORD_LEN = 61;
  const VALIDATOR_RECORD_VOTE_ACC_OFFSET = 0;

  const d = accData("marinade-validator-list");
  const addrDec = getAddressDecoder();
  const len = (d.length - VALIDATOR_RECORDS_START) / VALIDATOR_RECORD_LEN;
  for (let i = 0; i < len; i++) {
    if (
      addrDec.decode(
        d,
        VALIDATOR_RECORDS_START +
          i * VALIDATOR_RECORD_LEN +
          VALIDATOR_RECORD_VOTE_ACC_OFFSET
      ) === vote
    ) {
      return i;
    }
  }
}

function splMain(vote: Address, valListName: string) {
  const VSI_LIST_LEN_OFFSET = 5;
  const VSI_LIST_START = 9;
  const VSI_LEN = 73;
  const VSI_VOTE_ACC_OFFSET = 41;

  const d = accData(valListName);
  const len = getU32Decoder().decode(d, VSI_LIST_LEN_OFFSET);
  const addrDec = getAddressDecoder();
  for (let i = 0; i < len; i++) {
    if (
      addrDec.decode(d, VSI_LIST_START + i * VSI_LEN + VSI_VOTE_ACC_OFFSET) ===
      vote
    ) {
      return i;
    }
  }
}

function accData(name: string): Uint8Array {
  const acc = readTestFixturesJsonFile(name);
  const b64Enc = getBase64Encoder();
  return new Uint8Array(b64Enc.encode(acc.account.data[0]));
}

main();

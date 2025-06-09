/**
 * Change a stake account fixture's stake amount
 */

import {
  getBase64Codec,
  getU64Encoder,
  lamports,
  type Base64EncodedBytes,
} from "@solana/kit";
import {
  readTestFixturesJsonFile,
  writeTestFixturesJsonFile,
} from "../utils/file";
import { STAKE_ACC_STAKE_OFFSET } from "../utils/stake";

function main() {
  const [_node, _script, name, stakeStr] = process.argv;

  if (!name || !stakeStr) {
    console.log(
      "Usage: set-activation-epoch.ts <test-fixtures-filename-without-json-ext> <stake-lamports>"
    );
    return;
  }

  const acc = readTestFixturesJsonFile(name);

  const b64codec = getBase64Codec();
  const stake = BigInt(stakeStr);
  const bytes = new Uint8Array(b64codec.encode(acc.account.data[0]));
  bytes.set(getU64Encoder().encode(stake), STAKE_ACC_STAKE_OFFSET);
  acc.account.data[0] = b64codec.decode(bytes) as Base64EncodedBytes;
  acc.account.lamports = lamports(stake + 2282880n);

  writeTestFixturesJsonFile(name, acc);
}

main();

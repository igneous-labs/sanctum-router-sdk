/**
 * Change a stake account fixture's activation epoch
 */

import {
  getBase64Codec,
  getU64Encoder,
  type Base64EncodedBytes,
} from "@solana/kit";
import {
  readTestFixturesJsonFile,
  writeTestFixturesJsonFile,
} from "../utils/file";
import { STAKE_ACC_ACTIVATION_EPOCH_OFFSET } from "../utils/stake";

function main() {
  const [_node, _script, name, activationEpoch] = process.argv;

  if (!name || !activationEpoch) {
    console.log(
      "Usage: set-activation-epoch.ts <test-fixtures-filename-without-json-ext> <activation-epoch>"
    );
    return;
  }

  const acc = readTestFixturesJsonFile(name);

  const b64codec = getBase64Codec();

  const bytes = new Uint8Array(b64codec.encode(acc.account.data[0]));
  bytes.set(
    getU64Encoder().encode(BigInt(activationEpoch)),
    STAKE_ACC_ACTIVATION_EPOCH_OFFSET
  );
  acc.account.data[0] = b64codec.decode(bytes) as Base64EncodedBytes;

  writeTestFixturesJsonFile(name, acc);
}

main();

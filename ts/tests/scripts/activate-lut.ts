/**
 * Make a LUT ready for use by setting
 * `last_extended_slot=0` and `last_extended_slot_start_index=0`
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

const LAST_EXTENDED_SLOT_OFFSET = 12;
const LAST_EXTENDED_SLOT_START_INDEX_OFFSET = 20;

function main() {
  const [_node, _script, name] = process.argv;

  if (!name) {
    console.log(
      "Usage: activate-lut.ts <test-fixtures-filename-without-json-ext>"
    );
    return;
  }

  const acc = readTestFixturesJsonFile(name);

  const b64codec = getBase64Codec();

  const bytes = new Uint8Array(b64codec.encode(acc.account.data[0]));
  bytes.set(getU64Encoder().encode(BigInt(0)), LAST_EXTENDED_SLOT_OFFSET);
  bytes[LAST_EXTENDED_SLOT_START_INDEX_OFFSET] = 0;
  acc.account.data[0] = b64codec.decode(bytes) as Base64EncodedBytes;

  writeTestFixturesJsonFile(name, acc);
}

main();

/**
 * Change a token account fixture's owner/authority
 */

import {
  address,
  getAddressEncoder,
  getBase64Codec,
  type Base64EncodedBytes,
} from "@solana/kit";
import {
  readTestFixturesJsonFile,
  writeTestFixturesJsonFile,
} from "../utils/file";
import { TOKEN_ACC_OWNER_OFFSET } from "../utils/token";

function main() {
  const [_node, _script, name, ownerStr] = process.argv;

  if (!name || !ownerStr) {
    console.log(
      "Usage: set-token-acc-owner.ts <test-fixtures-filename-without-json-ext> <bs58-new-owner-pk>"
    );
    return;
  }

  const owner = address(ownerStr);
  const acc = readTestFixturesJsonFile(name);

  const b64codec = getBase64Codec();
  const bytes = new Uint8Array(b64codec.encode(acc.account.data[0]));
  bytes.set(getAddressEncoder().encode(owner), TOKEN_ACC_OWNER_OFFSET);
  acc.account.data[0] = b64codec.decode(bytes) as Base64EncodedBytes;

  writeTestFixturesJsonFile(name, acc);
}

main();

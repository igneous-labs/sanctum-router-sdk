import {
  address,
  getAddressDecoder,
  getBase64Encoder,
  getU64Decoder,
  type Address,
  type ReadonlyUint8Array,
} from "@solana/kit";
import { readTestFixturesJsonFile } from "./file";

// mints
export const NATIVE_MINT = address(
  "So11111111111111111111111111111111111111112"
);
export const MSOL_MINT = address("mSoLzYCxHdYgdzU16g5QSh3i5K3z3KZK7ytfqcJm7So");
export const PICOSOL_MINT = address(
  "picobAEvs6w7QEknPce34wAE4gknZA9v5tTonnmHYdX"
);
export const STSOL_MINT = address(
  "7dHbWXmci3dT8UFYWYZweBLXgycu7Y3iL6trKn1Y7ARj"
);

export const TOKEN_ACC_OWNER_OFFSET = 32;
const TOKEN_ACC_BALANCE_OFFSET = 64;

export function tokenAccOwner(accData: ReadonlyUint8Array): Address {
  return getAddressDecoder().decode(accData, TOKEN_ACC_OWNER_OFFSET);
}

export function tokenAccBalance(accData: ReadonlyUint8Array): bigint {
  return getU64Decoder().decode(accData, TOKEN_ACC_BALANCE_OFFSET);
}

export function testFixturesTokenAcc(tokenAccFname: string): {
  addr: Address;
  owner: Address;
} {
  const {
    pubkey,
    account: {
      data: [data],
    },
  } = readTestFixturesJsonFile(tokenAccFname);
  const owner = tokenAccOwner(getBase64Encoder().encode(data));
  return {
    addr: pubkey,
    owner,
  };
}

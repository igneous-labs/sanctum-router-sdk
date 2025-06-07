import type { Address } from "@solana/kit";

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

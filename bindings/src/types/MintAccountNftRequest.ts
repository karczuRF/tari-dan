// This file was generated by [ts-rs](https://github.com/Aleph-Alpha/ts-rs). Do not edit this file manually.
import type { ComponentAddressOrName } from "./ComponentAddressOrName";

export interface MintAccountNftRequest {
  account: ComponentAddressOrName;
  metadata: string;
  mint_fee: number | null;
  create_account_nft_fee: number | null;
}
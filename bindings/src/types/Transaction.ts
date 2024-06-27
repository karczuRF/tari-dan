// This file was generated by [ts-rs](https://github.com/Aleph-Alpha/ts-rs). Do not edit this file manually.
import type { Epoch } from "./Epoch";
import type { Instruction } from "./Instruction";
import type { SubstateRequirement } from "./SubstateRequirement";
import type { TransactionSignature } from "./TransactionSignature";
import type { VersionedSubstateId } from "./VersionedSubstateId";

export interface Transaction {
  id: string;
  fee_instructions: Array<Instruction>;
  instructions: Array<Instruction>;
  inputs: Array<SubstateRequirement>;
  min_epoch: Epoch | null;
  max_epoch: Epoch | null;
  signatures: Array<TransactionSignature>;
  filled_inputs: Array<VersionedSubstateId>;
}

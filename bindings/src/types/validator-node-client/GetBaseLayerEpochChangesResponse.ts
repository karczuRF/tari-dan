// This file was generated by [ts-rs](https://github.com/Aleph-Alpha/ts-rs). Do not edit this file manually.
import type { Epoch } from "../Epoch";
import type { ValidatorNodeChange } from "./ValidatorNodeChange";

export interface GetBaseLayerEpochChangesResponse {
  changes: Array<[Epoch, Array<ValidatorNodeChange>]>;
}

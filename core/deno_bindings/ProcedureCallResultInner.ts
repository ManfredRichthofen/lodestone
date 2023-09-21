// This file was generated by [ts-rs](https://github.com/Aleph-Alpha/ts-rs). Do not edit this file manually.
import type { ConfigurableManifest } from "./ConfigurableManifest.ts";
import type { Game } from "./Game.ts";
import type { GenericPlayer } from "./GenericPlayer.ts";
import type { InstanceState } from "./InstanceState.ts";
import type { PerformanceReport } from "./PerformanceReport.ts";
import { SetupManifest } from "./SetupManifest.ts";

export type ProcedureCallResultInner =
  | { String: string }
  | { Monitor: PerformanceReport }
  | { State: InstanceState }
  | { Num: number }
  | { Game: Game }
  | { Bool: boolean }
  | { ConfigurableManifest: ConfigurableManifest }
  | { Player: Array<GenericPlayer> }
  | { SetupManifest: SetupManifest }
  | "Void";

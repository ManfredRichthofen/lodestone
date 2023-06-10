// This file was generated by [ts-rs](https://github.com/Aleph-Alpha/ts-rs). Do not edit this file manually.
import type { CausedBy } from "./CausedBy.ts";
import type { ConfigurableValue } from "./ConfigurableValue.ts";
import type { DotLodestoneConfig } from "./DotLodestoneConfig.ts";
import type { ManifestValue } from "./ManifestValue.ts";

export type ProcedureCallInner =
  | {
    type: "SetupInstance";
    dot_lodestone_config: DotLodestoneConfig;
    setup_value: ManifestValue;
    path: string;
  }
  | {
    type: "RestoreInstance";
    dot_lodestone_config: DotLodestoneConfig;
    path: string;
  }
  | { type: "DestructInstance" }
  | { type: "GetSetupManifest" }
  | { type: "GetName" }
  | { type: "GetDescription" }
  | { type: "GetVersion" }
  | { type: "GetGame" }
  | { type: "GetPort" }
  | { type: "GetAutoStart" }
  | { type: "GetRestartOnCrash" }
  | { type: "SetName"; new_name: string }
  | { type: "SetDescription"; new_description: string }
  | { type: "SetPort"; new_port: number }
  | { type: "SetAutoStart"; new_auto_start: boolean }
  | { type: "SetRestartOnCrash"; new_restart_on_crash: boolean }
  | { type: "GetConfigurableManifest" }
  | {
    type: "UpdateConfigurable";
    section_id: string;
    setting_id: string;
    new_value: ConfigurableValue;
  }
  | { type: "StartInstance"; caused_by: CausedBy; block: boolean }
  | { type: "StopInstance"; caused_by: CausedBy; block: boolean }
  | { type: "RestartInstance"; caused_by: CausedBy; block: boolean }
  | { type: "KillInstance"; caused_by: CausedBy }
  | { type: "GetState" }
  | { type: "SendCommand"; command: string; caused_by: CausedBy }
  | { type: "Monitor" }
  | { type: "GetPlayerCount" }
  | { type: "GetMaxPlayerCount" }
  | { type: "GetPlayerList" }
  | { type: "GetMacroList" }
  | { type: "GetTaskList" }
  | { type: "GetHistoryList" }
  | { type: "DeleteMacro"; name: string }
  | { type: "CreateMacro"; name: string; content: string }
  | {
    type: "RunMacro";
    name: string;
    args: Array<string>;
    caused_by: CausedBy;
  };

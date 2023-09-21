import * as atom from "https://raw.githubusercontent.com/Lodestone-Team/lodestone_core/dev/src/implementations/generic/js/main/libs/atom_instance.ts";
import { EventStream } from "https://raw.githubusercontent.com/Lodestone-Team/lodestone-macro-lib/main/events.ts";

export default class TestInstance extends atom.AtomInstance {
    uuid!: string;
    _state : atom.InstanceState = "Stopped";
    public async setupManifest(): Promise<atom.SetupManifest> {
        return {
            setting_sections: {
                "test": {
                    section_id: "section_id1",
                    name: "section_name1",
                    description: "section_description1",
                    settings: {
                        "setting_id1": {
                            setting_id: "setting_id1",
                            name: "setting_name1",
                            description: "setting_description1",
                            value: null,
                            value_type: { type: "String", regex: null },
                            default_value: null,
                            is_secret: false,
                            is_required: true,
                            is_mutable: true,
                        }
                    },
                }
            }
        };
    }
    public async setup(setupValue: atom.SetupValue, dotLodestoneConfig: atom.DotLodestoneConfig, path: string): Promise<void> {
        this.uuid = dotLodestoneConfig.uuid;
        return;
    }
    public async restore(dotLodestoneConfig: atom.DotLodestoneConfig, path: string): Promise<void> {
        this.uuid = dotLodestoneConfig.uuid;
        return;
    }
    public async start(caused_by: atom.CausedBy, block: boolean): Promise<void> {
        console.log("start");
        new EventStream(this.uuid, "test_im_in_ts").emitStateChange("Running");
        return;
    }
    public async stop(caused_by: atom.CausedBy, block: boolean): Promise<void> {
        console.log("stop");
        new EventStream(this.uuid, "test_im_in_ts").emitStateChange("Stopped");
        return;
    }
    public restart(caused_by: atom.CausedBy, block: boolean): Promise<void> {
        throw new Error("Method not implemented.");
    }
    public kill(caused_by: atom.CausedBy): Promise<void> {
        throw new Error("Method not implemented.");
    }
    public async state(): Promise<atom.InstanceState> {
        return "Stopped"
    }
    public sendCommand(command: string, caused_by: atom.CausedBy): Promise<void> {
        throw new Error("Method not implemented.");
    }
    public monitor(): Promise<atom.PerformanceReport> {
        throw new Error("Method not implemented.");
    }
    public configurableManifest(): Promise<atom.ConfigurableManifest> {
        throw new Error("Method not implemented.");
    }
    public async name(): Promise<string> {
        return "test_im_in_ts";
    }
    public version(): Promise<string> {
        throw new Error("Method not implemented.");
    }
    public game(): Promise<atom.Game> {
        throw new Error("Method not implemented.");
    }
    public description(): Promise<string> {
        throw new Error("Method not implemented.");
    }
    public port(): Promise<number> {
        throw new Error("Method not implemented.");
    }
    public getAutoStart(): Promise<boolean> {
        throw new Error("Method not implemented.");
    }
    public getRestartOnCrash(): Promise<boolean> {
        throw new Error("Method not implemented.");
    }
    public setName(name: string): Promise<void> {
        throw new Error("Method not implemented.");
    }
    public setDescription(description: string): Promise<void> {
        throw new Error("Method not implemented.");
    }
    public setPort(port: number): Promise<void> {
        throw new Error("Method not implemented.");
    }
    public setAutoStart(auto_start: boolean): Promise<void> {
        throw new Error("Method not implemented.");
    }
    public setRestartOnCrash(restart_on_crash: boolean): Promise<void> {
        throw new Error("Method not implemented.");
    }
    public playerCount(): Promise<number> {
        throw new Error("Method not implemented.");
    }
    public maxPlayerCount(): Promise<number> {
        throw new Error("Method not implemented.");
    }
    public playerList(): Promise<atom.GenericPlayer[]> {
        throw new Error("Method not implemented.");
    }
    public updateConfigurable(section_id: string, setting_id: string, value: atom.ConfigurableValue): Promise<void> {
        throw new Error("Method not implemented.");
    }
}

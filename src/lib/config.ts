import { invoke } from "@tauri-apps/api/tauri";

export type MultiProvider = "opgg" | "deeplol" | "ugg" | "tracker";

export const MULTI_PROVIDERS: ReadonlyArray<{
    label: string;
    value: MultiProvider;
}> = [
    { label: "OP.GG", value: "opgg" },
    { label: "DeepLoL", value: "deeplol" },
    { label: "U.GG", value: "ugg" },
    { label: "Tracker.gg", value: "tracker" },
];

export interface Config {
    autoOpen: boolean;
    autoOpenDelaySeconds: number;
    autoAccept: boolean;
    acceptDelay: number;
    multiProvider: MultiProvider;
}

export const DEFAULT_CONFIG: Config = {
    autoOpen: true,
    autoOpenDelaySeconds: 6,
    autoAccept: false,
    acceptDelay: 2_000,
    multiProvider: "opgg",
};

export async function updateConfig(config: Config): Promise<void> {
    await invoke<void>("set_config", {
        newCfg: config,
    });
}

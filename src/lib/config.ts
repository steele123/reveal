import { invoke } from "@tauri-apps/api/tauri";

export type MultiProvider =
    | "opgg"
    | "ugg"
    | "fow"
    | "porofessor"
    | "laddereye";

export const MULTI_PROVIDERS: ReadonlyArray<{
    label: string;
    value: MultiProvider;
}> = [
    { label: "OP.GG", value: "opgg" },
    { label: "U.GG", value: "ugg" },
    { label: "FOW.LOL", value: "fow" },
    { label: "Porofessor", value: "porofessor" },
    { label: "LadderEye", value: "laddereye" },
];

export interface Config {
    autoOpen: boolean;
    autoOpenDelaySeconds: number;
    autoAccept: boolean;
    acceptDelay: number;
    pauseQueueAfterDodge: boolean;
    multiProvider: MultiProvider;
}

export const DEFAULT_CONFIG: Config = {
    autoOpen: true,
    autoOpenDelaySeconds: 6,
    autoAccept: false,
    acceptDelay: 2_000,
    pauseQueueAfterDodge: false,
    multiProvider: "opgg",
};

export async function updateConfig(config: Config): Promise<void> {
    await invoke<void>("set_config", {
        newCfg: config,
    });
}

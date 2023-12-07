import { invoke } from "@tauri-apps/api/tauri";

export interface Config {
    autoOpen: boolean;
    autoAccept: boolean;
    acceptDelay: number;
}

export async function updateConfig(config: Config) {
    await invoke("set_config", {
        newCfg: config
    });
}
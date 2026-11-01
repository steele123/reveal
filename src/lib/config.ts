import { invoke } from "@tauri-apps/api/tauri";
import type { CountersProvider, Role } from "$lib/matchups";

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

export type PlayerClickAction = "details" | "opgg";

export const PLAYER_CLICK_ACTIONS: ReadonlyArray<{
    label: string;
    value: PlayerClickAction;
}> = [
    { label: "Show details", value: "details" },
    { label: "Open profile", value: "opgg" },
];

export interface Config {
    autoOpen: boolean;
    autoOpenDelaySeconds: number;
    autoAccept: boolean;
    acceptDelay: number;
    pauseQueueAfterDodge: boolean;
    multiProvider: MultiProvider;
    discordPromptSeen: boolean;
    playerClickAction: PlayerClickAction;
    countersProvider: CountersProvider;
    /** Queue again 30 seconds after a game ends. */
    autoRequeue: boolean;
    /** Leave and rejoin the queue once the wait passes League's estimate. */
    requeuePastEstimate: boolean;
    /** Ban the first available of `banChampions` in your ban's last seconds, if you haven't. */
    autoBan: boolean;
    banChampions: number[];
    /** Lock in your hover, or else your role's first available pick, in your pick's last seconds. */
    autoPick: boolean;
    pickChampions: Partial<Record<Role, number[]>>;
    notifications: NotificationSettings;
}

/** Which alerts go where, by `NotificationAlert`. */
export interface NotificationSettings {
    /** Windows notifications. */
    desktop: string[];
    /** Sent to `ntfyTopic` on ntfy.sh, for the ntfy phone app. */
    phone: string[];
    ntfyTopic: string;
    /** Posted to the channel of `discordWebhook`. */
    discord: string[];
    discordWebhook: string;
}

export const DEFAULT_CONFIG: Config = {
    autoOpen: true,
    autoOpenDelaySeconds: 6,
    autoAccept: false,
    acceptDelay: 2_000,
    pauseQueueAfterDodge: false,
    multiProvider: "opgg",
    discordPromptSeen: false,
    playerClickAction: "details",
    countersProvider: "lolalytics",
    autoRequeue: false,
    requeuePastEstimate: false,
    autoBan: false,
    banChampions: [],
    autoPick: false,
    pickChampions: {},
    notifications: {
        desktop: ["autoAction", "dodge"],
        phone: ["autoQueue", "matchFound", "yourTurn", "dodge"],
        ntfyTopic: "",
        discord: ["autoQueue", "matchFound", "yourTurn", "dodge"],
        discordWebhook: "",
    },
};

export async function updateConfig(config: Config): Promise<void> {
    await invoke<void>("set_config", {
        newCfg: config,
    });
}

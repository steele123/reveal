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
    /** Queue again `requeueDelaySeconds` after a game ends. */
    autoRequeue: boolean;
    /** Wait before queueing again after a game; 0 is right away. */
    requeueDelaySeconds: number;
    /** Leave and rejoin the queue once the wait passes League's estimate. */
    requeuePastEstimate: boolean;
    /** Wait between leaving a long queue and joining it again; 0 is right away. */
    longQueueRequeueDelaySeconds: number;
    /** How long a pause after a dodge lasts before Reveal queues again; 0 waits for you. */
    dodgePauseSeconds: number;
    /** Ban the first available of `banChampions` in your ban's last seconds, if you haven't. */
    autoBan: boolean;
    banChampions: number[];
    /** Seconds before the ban timer runs out to ban; 1 is the last second. */
    autoBanSecondsLeft: number;
    /** Lock in your hover, or else your role's first available pick, in your pick's last seconds. */
    autoPick: boolean;
    pickChampions: Partial<Record<Role, number[]>>;
    /** Seconds before the pick timer runs out to lock in; 1 is the last second. */
    autoPickSecondsLeft: number;
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
    requeueDelaySeconds: 30,
    longQueueRequeueDelaySeconds: 30,
    dodgePauseSeconds: 0,
    requeuePastEstimate: false,
    autoBan: false,
    banChampions: [],
    autoBanSecondsLeft: 1,
    autoPick: false,
    pickChampions: {},
    autoPickSecondsLeft: 1,
    notifications: {
        desktop: ["autoAction", "dodge"],
        phone: ["matchFound", "yourTurn", "dodge"],
        ntfyTopic: "",
        discord: ["matchFound", "yourTurn", "dodge"],
        discordWebhook: "",
    },
};

export async function updateConfig(config: Config): Promise<void> {
    await invoke<void>("set_config", {
        newCfg: config,
    });
}

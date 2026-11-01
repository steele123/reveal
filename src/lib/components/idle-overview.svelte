<script lang="ts">
  import { championIconUrl, positionBadge } from "$lib/champ_select";
  import type { Config } from "$lib/config";
  import {
    formatTimeAgo,
    participantKey,
    seenBeforeIndex,
    type LobbyHistoryEntry,
  } from "$lib/lobby_history";
  import { LANES } from "$lib/matchups";
  import { isDiscordWebhook, isNtfyTopic } from "$lib/notifications";

  /** Between Champ Selects: your last lobbies and what Reveal will do for you. */
  export let history: LobbyHistoryEntry[] = [];
  export let config: Config | null = null;
  export let onOpenPage: (page: "history" | "auto") => void = () => {};

  const RECENT_LOBBIES = 4;
  const BAN_ICONS = 4;

  $: recent = history.slice(0, RECENT_LOBBIES);
  // Players in more than one stored lobby stand out, since spotting them is what history is for.
  $: appearances = seenBeforeIndex(history, null);
  $: pickRoles = LANES.filter((lane) => config?.pickChampions[lane]?.length).map(
    (lane) => `${positionBadge(lane)} ${config?.pickChampions[lane]?.length}`,
  );
  $: channels = config ? notificationChannels(config) : [];

  function outcome(entry: LobbyHistoryEntry): { label: string; tone: string } {
    if (entry.gameStartedAt) return { label: "Played", tone: "text-emerald-300 bg-emerald-500/10" };
    if (entry.dodgedAt) return { label: "Dodged", tone: "text-amber-300 bg-amber-500/10" };
    return { label: "Lobby", tone: "text-muted-foreground bg-white/[0.05]" };
  }

  /** The channels that would actually send something. */
  function notificationChannels(config: Config): string[] {
    const settings = config.notifications;
    return [
      settings.desktop.length > 0 && "This PC",
      settings.phone.length > 0 && isNtfyTopic(settings.ntfyTopic) && "Phone",
      settings.discord.length > 0 && isDiscordWebhook(settings.discordWebhook) && "Discord",
    ].filter((channel): channel is string => Boolean(channel));
  }
</script>

<div class="grid min-h-0 flex-1 grid-cols-2 gap-3">
  <section class="reveal-panel flex min-h-0 flex-col p-3">
    <div class="flex items-center justify-between">
      <h2 class="text-[10px] font-medium uppercase tracking-[0.12em] text-muted-foreground">
        Recent lobbies
      </h2>
      {#if history.length}
        <button
          type="button"
          class="text-[10px] text-muted-foreground transition hover:text-foreground"
          on:click={() => onOpenPage("history")}>All {history.length} ›</button
        >
      {/if}
    </div>
    {#if recent.length}
      <div class="reveal-scrollbar mt-2 min-h-0 flex-1 space-y-1 overflow-y-auto">
        {#each recent as entry (entry.id)}
          {@const result = outcome(entry)}
          <button
            type="button"
            class="block w-full rounded-lg border border-white/[0.06] bg-white/[0.02] px-2.5 py-1.5 text-left transition hover:border-white/15 hover:bg-white/[0.04]"
            on:click={() => onOpenPage("history")}
          >
            <div class="flex items-center gap-2 text-[10px]">
              <span class="text-muted-foreground">{formatTimeAgo(entry.revealedAt)}</span>
              <span class="rounded px-1.5 py-px text-[9px] font-medium {result.tone}">{result.label}</span>
            </div>
            <div class="mt-0.5 truncate text-[11px]">
              {#each entry.participants as participant, index (participantKey(participant))}
                {@const lobbies = appearances.get(participantKey(participant))?.lobbies ?? 1}
                {#if index > 0}<span class="text-muted-foreground">{" · "}</span>{/if}<span
                  class:text-blue-300={lobbies > 1}
                  title={lobbies > 1 ? `In ${lobbies} of your stored lobbies` : undefined}
                  >{participant.game_name}</span
                >
              {/each}
            </div>
          </button>
        {/each}
      </div>
    {:else}
      <div class="grid flex-1 place-items-center text-center text-[11px] text-muted-foreground">
        Lobbies you reveal show up here, so you can spot players you've met before.
      </div>
    {/if}
  </section>

  <section class="reveal-panel flex min-h-0 flex-col p-3">
    <div class="flex items-center justify-between">
      <h2 class="text-[10px] font-medium uppercase tracking-[0.12em] text-muted-foreground">
        Automation
      </h2>
      <button
        type="button"
        class="text-[10px] text-muted-foreground transition hover:text-foreground"
        on:click={() => onOpenPage("auto")}>Change ›</button
      >
    </div>
    <dl class="mt-2 space-y-2 text-[11px]">
      <div class="flex items-center justify-between gap-2">
        <dt class="text-muted-foreground">Accept matches</dt>
        <dd class:text-emerald-300={config?.autoAccept}>{config?.autoAccept ? "On" : "Off"}</dd>
      </div>
      <div class="flex items-center justify-between gap-2">
        <dt class="text-muted-foreground">Queue again after a game</dt>
        <dd class:text-emerald-300={config?.autoRequeue}>{config?.autoRequeue ? "On" : "Off"}</dd>
      </div>
      <div class="flex items-center justify-between gap-2">
        <dt class="text-muted-foreground">Pause after a dodge</dt>
        <dd class:text-emerald-300={config?.pauseQueueAfterDodge}>
          {config?.pauseQueueAfterDodge ? "On" : "Off"}
        </dd>
      </div>
      <div class="flex items-center justify-between gap-2">
        <dt class="text-muted-foreground">Ban</dt>
        <dd class="flex items-center gap-1">
          {#if config?.autoBan && config.banChampions.length}
            {#each config.banChampions.slice(0, BAN_ICONS) as championId (championId)}
              <img src={championIconUrl(championId)} alt="" class="h-5 w-5 rounded bg-white/5 object-cover" />
            {/each}
            {#if config.banChampions.length > BAN_ICONS}
              <span class="text-[10px] text-muted-foreground">+{config.banChampions.length - BAN_ICONS}</span>
            {/if}
          {:else}
            <span>{config?.autoBan ? "On, no list yet" : "Off"}</span>
          {/if}
        </dd>
      </div>
      <div class="flex items-center justify-between gap-2">
        <dt class="text-muted-foreground">Lock in</dt>
        <dd class:text-emerald-300={config?.autoPick} class="truncate">
          {#if config?.autoPick}
            {pickRoles.length ? `Hover, then ${pickRoles.join(" · ")}` : "Your hover"}
          {:else}
            Off
          {/if}
        </dd>
      </div>
      <div class="flex items-center justify-between gap-2">
        <dt class="text-muted-foreground">Notifications</dt>
        <dd class:text-emerald-300={channels.length > 0} class="truncate">
          {channels.length ? channels.join(" · ") : "Off"}
        </dd>
      </div>
    </dl>
  </section>
</div>

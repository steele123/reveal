<script lang="ts">
  import {
    championIconUrl,
    formatRank,
    profileIconUrl,
  } from "$lib/champ_select";
  import type { Config } from "$lib/config";
  import {
    formatTimeAgo,
    participantKey,
    seenBeforeIndex,
    type LobbyHistoryEntry,
  } from "$lib/lobby_history";
  import { LANES } from "$lib/matchups";
  import { logFrontendError } from "$lib/logging";
  import { isDiscordWebhook, isNtfyTopic } from "$lib/notifications";
  import { PREVIEW_OWN_PROFILE } from "$lib/preview";
  import { formatRecord, getOwnProfile, recordWinRate, type OwnProfile } from "$lib/profile";
  import { isTauriRuntime } from "$lib/runtime";

  /** Between Champ Selects: your last lobbies and what Reveal will do for you. */
  export let history: LobbyHistoryEntry[] = [];
  export let config: Config | null = null;
  export let onOpenPage: (page: "history" | "auto") => void = () => {};
  export let connected = false;

  /** You, as signed in to League. Read again each time this screen shows, e.g. after a game. */
  let profile: OwnProfile | null = null;
  let profileLoading = false;
  let profileFailed = false;
  $: if (!connected) {
    // Another account may sign in next.
    profile = null;
    profileFailed = false;
  }
  $: if (connected && !profile && !profileLoading && !profileFailed) void loadProfile();

  async function loadProfile() {
    profileLoading = true;
    try {
      profile = isTauriRuntime() ? await getOwnProfile() : PREVIEW_OWN_PROFILE;
    } catch (error) {
      profileFailed = true;
      logFrontendError("Failed to read your profile from League", error);
    } finally {
      profileLoading = false;
    }
  }

  const QUEUES: ReadonlyArray<{ label: string; key: "solo" | "flex" }> = [
    { label: "Solo/Duo", key: "solo" },
    { label: "Flex", key: "flex" },
  ];

  /** Shown until the rows are measured; then as many as fit the window's height. */
  const RECENT_LOBBIES = 3;
  /** The gap between lobby rows (space-y-1). */
  const ROW_GAP = 4;
  const BAN_ICONS = 4;

  let listHeight = 0;
  let rowHeight = 0;
  $: fits = rowHeight
    ? Math.max(1, Math.floor((listHeight + ROW_GAP) / (rowHeight + ROW_GAP)))
    : RECENT_LOBBIES;
  $: recent = history.slice(0, fits);
  // Players in more than one stored lobby stand out, since spotting them is what history is for.
  $: appearances = seenBeforeIndex(history, null);
  /** Every role's picks in lane order, each champion once. */
  $: pickChampions = [
    ...new Set(LANES.flatMap((lane) => config?.pickChampions[lane] ?? [])),
  ];
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

{#if connected}
  <section class="reveal-panel flex shrink-0 items-center gap-3 px-3 py-2">
    {#if profile}
      <img
        src={profileIconUrl(profile.profileIconId)}
        alt=""
        class="h-8 w-8 shrink-0 rounded-lg border border-white/10 bg-white/5 object-cover"
      />
      <div class="min-w-0 flex-1">
        <div class="truncate text-xs font-semibold">
          {profile.gameName}<span class="font-normal text-muted-foreground">#{profile.gameTag}</span>
        </div>
        <div class="text-[10px] text-muted-foreground">Level {profile.level}</div>
      </div>
      {#each QUEUES as queue (queue.key)}
        {@const rank = formatRank(profile[queue.key])}
        {@const record = formatRecord(profile[queue.key])}
        {@const winRate = recordWinRate(profile[queue.key])}
        <div class="shrink-0 text-right" title={record ? `${rank.tier} · ${record}` : rank.tier}>
          <div class="text-[9px] uppercase tracking-[0.08em] text-muted-foreground">{queue.label}</div>
          <div class="flex items-center justify-end gap-1 text-[11px]">
            <img src={rank.icon} alt="" class="h-4 w-4" />
            <span class={rank.tierClass}>{rank.shortTier}</span>
            <span class="text-[10px] text-muted-foreground"
              >{[rank.details[0], winRate].filter(Boolean).join(" · ")}</span
            >
          </div>
        </div>
      {/each}
    {:else if profileFailed}
      <div class="flex-1 text-[11px] text-muted-foreground">Couldn't read your rank from League.</div>
      <button
        type="button"
        class="shrink-0 text-[10px] text-muted-foreground transition hover:text-foreground"
        on:click={() => (profileFailed = false)}>Retry</button
      >
    {:else}
      <div class="h-8 w-8 animate-pulse rounded-lg bg-white/[0.05]"></div>
      <div class="h-3 w-32 animate-pulse rounded bg-white/[0.05]"></div>
    {/if}
  </section>
{/if}

<div class="flex min-h-0 flex-1 flex-col gap-3">
  <section class="reveal-panel shrink-0 p-3">
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
    <dl class="mt-2 grid grid-cols-2 gap-x-6 gap-y-2 text-[11px]">
      <div class="flex items-center justify-between gap-2">
        <dt class="text-muted-foreground">Accept matches</dt>
        <dd class:text-emerald-300={config?.autoAccept}>
          {#if config?.autoAccept}
            {config.acceptDelay >= 9000
              ? "Last second"
              : config.acceptDelay > 0
                ? `After ${Math.round(config.acceptDelay / 1000)} sec`
                : "Right away"}
          {:else}
            Off
          {/if}
        </dd>
      </div>
      <div class="flex items-center justify-between gap-2">
        <dt class="text-muted-foreground">Queue again after a game</dt>
        <dd class:text-emerald-300={config?.autoRequeue}>
          {#if config?.autoRequeue}
            {config.requeueDelaySeconds > 0 ? `After ${config.requeueDelaySeconds} sec` : "Right away"}
          {:else}
            Off
          {/if}
        </dd>
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
        <dd class="flex items-center gap-1">
          {#if config?.autoPick && pickChampions.length}
            {#each pickChampions.slice(0, BAN_ICONS) as championId (championId)}
              <img src={championIconUrl(championId)} alt="" class="h-5 w-5 rounded bg-white/5 object-cover" />
            {/each}
            {#if pickChampions.length > BAN_ICONS}
              <span class="text-[10px] text-muted-foreground">+{pickChampions.length - BAN_ICONS}</span>
            {/if}
          {:else}
            <span>{config?.autoPick ? "On, no list yet" : "Off"}</span>
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

  <section class="reveal-panel flex min-h-0 flex-1 flex-col p-3">
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
      <div bind:clientHeight={listHeight} class="mt-2 min-h-0 flex-1 space-y-1 overflow-hidden">
        {#each recent as entry (entry.id)}
          {@const result = outcome(entry)}
          <button
            type="button"
            bind:offsetHeight={rowHeight}
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
                  class:text-blue-300={lobbies > 1 && !participant.is_self}
                  title={lobbies > 1 && !participant.is_self ? `In ${lobbies} of your stored lobbies` : undefined}
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

</div>

<script lang="ts">
  import { onDestroy } from "svelte";
  import { fade, scale } from "svelte/transition";
  import {
    championIconUrl,
    formatAssignedPosition,
    formatRank,
    MIN_GAMES_FOR_WIN_RATE,
    offRoleReason,
    profileIconUrl,
    type Participant,
  } from "$lib/champ_select";
  import {
    copyLink,
    createOpggProfileLink,
    openExternalLink,
  } from "$lib/link_actions";
  import { logFrontendError } from "$lib/logging";

  export let participant: Participant;
  export let onClose: () => void = () => {};

  let copied = false;
  let copiedTimer: ReturnType<typeof setTimeout> | undefined;

  $: riotId = `${participant.game_name}#${participant.game_tag}`;
  $: rank = participant.rank
    ? formatRank(participant.rank, participant.recent)
    : null;
  $: recentGames = participant.recent
    ? participant.recent.wins + participant.recent.losses
    : 0;
  $: winRate =
    participant.recent && recentGames >= MIN_GAMES_FOR_WIN_RATE
      ? Math.round((participant.recent.wins / recentGames) * 100)
      : null;
  $: offRole = offRoleReason(participant);
  $: subtitle = [
    participant.assigned_position &&
      formatAssignedPosition(participant.assigned_position) +
        (participant.position_guessed ? " (likely)" : ""),
    participant.pick_turn !== undefined && `Pick ${participant.pick_turn + 1}`,
  ]
    .filter(Boolean)
    .join(" · ");

  function openOpgg() {
    openExternalLink(createOpggProfileLink(participant)).catch((error) =>
      logFrontendError("Failed to open OP.GG profile", error),
    );
  }

  async function copyRiotId() {
    try {
      await copyLink(riotId);
      copied = true;
      if (copiedTimer) clearTimeout(copiedTimer);
      copiedTimer = setTimeout(() => (copied = false), 2_000);
    } catch (error) {
      logFrontendError("Failed to copy Riot ID", error);
    }
  }

  function handleKeydown(event: KeyboardEvent) {
    if (event.key === "Escape") onClose();
  }

  onDestroy(() => {
    if (copiedTimer) clearTimeout(copiedTimer);
  });
</script>

<svelte:window on:keydown={handleKeydown} />

<!-- svelte-ignore a11y-click-events-have-key-events a11y-no-static-element-interactions -->
<div
  class="fixed inset-0 z-50 grid place-items-center bg-black/60 p-6 backdrop-blur-sm"
  transition:fade={{ duration: 120 }}
  on:click|self={onClose}
>
  <div
    role="dialog"
    aria-modal="true"
    aria-label={riotId}
    class="relative w-full max-w-md rounded-xl border border-white/10 bg-[#0b1220] p-4 shadow-2xl"
    transition:scale={{ duration: 120, start: 0.97 }}
  >
    <button
      class="absolute right-3 top-3 grid h-7 w-7 place-items-center rounded-md text-muted-foreground transition hover:bg-white/5 hover:text-foreground"
      aria-label="Close"
      on:click={onClose}
    >
      <svg viewBox="0 0 16 16" class="h-3.5 w-3.5" aria-hidden="true">
        <path
          d="m4 4 8 8M12 4l-8 8"
          fill="none"
          stroke="currentColor"
          stroke-linecap="round"
          stroke-width="1.5"
        />
      </svg>
    </button>

    <div class="flex items-center gap-3 pr-8">
      {#if participant.profile_icon_id !== undefined}
        <img
          src={profileIconUrl(participant.profile_icon_id)}
          alt=""
          class="h-14 w-14 shrink-0 rounded-lg bg-white/5 object-cover"
        />
      {/if}
      <div class="min-w-0">
        <div class="truncate text-base font-semibold">
          {participant.game_name}<span class="text-muted-foreground"
            >#{participant.game_tag}</span
          >
        </div>
        {#if subtitle}
          <div class="mt-0.5 text-xs text-muted-foreground">{subtitle}</div>
        {/if}
        {#if offRole}
          <div class="mt-1 text-[11px] font-medium text-amber-300">{offRole}</div>
        {/if}
      </div>
    </div>

    <div class="mt-4 grid grid-cols-2 gap-2">
      <div class="reveal-panel p-3">
        <div
          class="text-[10px] font-medium uppercase tracking-[0.12em] text-muted-foreground"
        >
          Solo/Duo
        </div>
        {#if rank && participant.rank}
          <div class="mt-1.5 flex items-center gap-2">
            <img src={rank.icon} alt="" class="h-6 w-7 object-contain" />
            <div class="min-w-0">
              <div class="truncate text-sm font-semibold {rank.tierClass}">
                {rank.tier}
              </div>
              {#if participant.rank.tier !== "UNRANKED"}
                <div class="text-[11px] text-muted-foreground">
                  {participant.rank.leaguePoints} LP
                </div>
              {/if}
            </div>
          </div>
          {#if participant.rank.wins > 0}
            <div class="mt-2 text-[11px] text-muted-foreground">
              {participant.rank.wins} wins this season
            </div>
          {/if}
          {#if participant.recent && recentGames > 0}
            <div class="mt-1 text-[11px] text-muted-foreground">
              Last {recentGames}:
              <span class="text-emerald-300">{participant.recent.wins}W</span>
              <span class="text-red-300">{participant.recent.losses}L</span>
              {#if winRate !== null}· {winRate}%{/if}
            </div>
            {#if winRate !== null}
              <div class="mt-1 h-1 overflow-hidden rounded-full bg-red-400/30">
                <div class="h-full bg-emerald-400" style="width: {winRate}%" />
              </div>
            {/if}
          {/if}
        {:else}
          <div class="mt-1.5 text-xs text-muted-foreground">Not available</div>
        {/if}
      </div>

      <div class="reveal-panel p-3">
        <div
          class="text-[10px] font-medium uppercase tracking-[0.12em] text-muted-foreground"
        >
          Most played
        </div>
        {#if participant.mains?.length}
          <div class="mt-1.5 space-y-1.5">
            {#each participant.mains as main (main.championId)}
              <div class="flex items-center gap-2">
                <img
                  src={championIconUrl(main.championId)}
                  alt=""
                  class="h-6 w-6 shrink-0 rounded-md bg-white/5 object-cover"
                />
                <div class="min-w-0 flex-1 truncate text-xs font-medium">
                  {main.name}
                </div>
                <div
                  class="shrink-0 text-[10px] text-muted-foreground"
                  title="Mastery level {main.level}"
                >
                  M{main.level} · {main.points >= 1000
                    ? `${Math.round(main.points / 1000)}k`
                    : main.points}
                </div>
              </div>
            {/each}
          </div>
        {:else}
          <div class="mt-1.5 text-xs text-muted-foreground">Not available</div>
        {/if}
      </div>
    </div>

    <div class="mt-4 flex gap-2">
      <button
        class="h-9 shrink-0 rounded-md border border-white/10 bg-white/[0.035] px-3 text-xs font-medium transition hover:bg-white/[0.07]"
        on:click={copyRiotId}
      >
        <!-- Both labels share one cell so the button keeps its width. -->
        <span class="grid">
          <span class="col-start-1 row-start-1" class:invisible={copied}
            >Copy Riot ID</span
          >
          <span class="col-start-1 row-start-1" class:invisible={!copied}
            >Copied</span
          >
        </span>
      </button>
      <button
        class="h-9 flex-1 rounded-md bg-blue-500 text-sm font-medium text-white shadow-[0_8px_24px_rgba(59,130,246,0.18)] transition hover:bg-blue-400"
        on:click={openOpgg}
      >
        Open OP.GG profile
      </button>
    </div>
  </div>
</div>

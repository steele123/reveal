<script lang="ts">
  import { onDestroy } from "svelte";
  import { invoke } from "@tauri-apps/api/tauri";
  import {
    championIconUrl,
    describeMain,
    formatAssignedPosition,
    formatRank,
    offRoleReason,
    positionBadge,
    profileIconUrl,
    type ChampSelect,
    type Participant,
  } from "$lib/champ_select";
  import { Button } from "./ui/button";
  import { logFrontendError } from "$lib/logging";
  import {
    copyLink,
    createOpggProfileLink,
    formatRiotIds,
    openExternalLink,
  } from "$lib/link_actions";
  import type { PlayerClickAction } from "$lib/config";
  import PlayerDetails from "./player-details.svelte";
  import {
    describeSeenBefore,
    participantKey,
    seenBeforeIndex,
    type LobbyHistoryEntry,
  } from "$lib/lobby_history";

  export let champSelect: ChampSelect | null = null;
  export let clickAction: PlayerClickAction = "details";
  export let history: LobbyHistoryEntry[] = [];
  export let activeLobbyId: string | null = null;

  $: seenIndex = seenBeforeIndex(history, activeLobbyId);
  $: seenBefore = (participant: Participant) =>
    participant.is_self ? undefined : seenIndex.get(participantKey(participant));

  let selectedPuuid: string | null = null;
  // Look the player up on every update so the popup fills in as ranks and mains arrive.
  $: selectedPlayer =
    champSelect?.participants.find(({ puuid }) => puuid === selectedPuuid) ?? null;

  function selectPlayer(participant: Participant) {
    if (clickAction === "opgg") {
      openExternalLink(createOpggProfileLink(participant)).catch((error) =>
        logFrontendError("Failed to open OP.GG profile", error),
      );
      return;
    }
    selectedPuuid = participant.puuid;
  }

  type CopyStatus = "idle" | "copied" | "failed";
  const COPY_LABELS: [CopyStatus, string][] = [
    ["idle", "Copy names"],
    ["copied", "Copied"],
    ["failed", "Copy failed"],
  ];

  let copyStatus: CopyStatus = "idle";
  let copyTimer: ReturnType<typeof setTimeout> | undefined;

  async function copyNames() {
    if (!champSelect) return;
    try {
      await copyLink(formatRiotIds(champSelect.participants));
      copyStatus = "copied";
    } catch (error) {
      logFrontendError("Failed to copy teammate names", error);
      copyStatus = "failed";
    }
    if (copyTimer) clearTimeout(copyTimer);
    copyTimer = setTimeout(() => (copyStatus = "idle"), 2_000);
  }

  onDestroy(() => {
    if (copyTimer) clearTimeout(copyTimer);
  });

  function openMultiLink() {
    void invoke<void>("open_opgg_link").catch((error) => {
      logFrontendError("Failed to open multi link", error);
    });
  }
</script>

<section class="flex min-h-0 flex-1 flex-col gap-2">
  <div class="flex items-center justify-between">
    <div>
      <div class="text-sm font-semibold">Champ Select</div>
      <div class="text-[11px] text-muted-foreground">
        Teammate identities are ready.
      </div>
    </div>
    <div
      class="rounded-full border border-blue-400/20 bg-blue-500/10 px-2 py-1 text-[10px] font-medium text-blue-300"
    >
      {champSelect?.participants.length ?? 0}/5 found
    </div>
  </div>

  <!-- One column, top to bottom in pick order, like the League client. -->
  <div class="-mr-1 flex min-h-0 flex-col gap-1.5 overflow-y-auto pr-1 text-sm">
    {#if champSelect}
      {#each champSelect.participants as participant, index (participant.puuid)}
        {@const offRole = offRoleReason(participant)}
        {@const seen = seenBefore(participant)}
        {@const badgeTitle = [
          participant.pick_turn === undefined
            ? `Player ${index + 1}`
            : `Pick ${participant.pick_turn + 1}`,
          participant.assigned_position &&
            (participant.position_guessed
              ? `Likely ${formatAssignedPosition(participant.assigned_position)}, guessed from their recent roles, summoner spells and champion (Riot hides this in ranked)`
              : formatAssignedPosition(participant.assigned_position)),
          offRole,
          seen && describeSeenBefore(seen),
        ]
          .filter(Boolean)
          .join(" · ")}
        {@const badgeLabel = participant.assigned_position
          ? positionBadge(participant.assigned_position) +
            (participant.position_guessed ? "?" : "")
          : participant.pick_turn !== undefined
            ? `P${participant.pick_turn + 1}`
            : String(index + 1).padStart(2, "0")}
        <button
          type="button"
          class="reveal-panel flex h-10 min-w-0 shrink-0 items-center gap-2.5 px-3 text-left transition hover:border-blue-400/30 hover:bg-blue-500/[0.06] focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-blue-400/50"
          aria-label={clickAction === "opgg"
            ? `Open ${participant.game_name}'s OP.GG profile`
            : `Show details for ${participant.game_name}`}
          on:click={() => selectPlayer(participant)}
        >
          {#if participant.profile_icon_id !== undefined}
            <div class="relative shrink-0" title={badgeTitle}>
              <img
                src={profileIconUrl(participant.profile_icon_id)}
                alt=""
                class="h-7 w-7 rounded-md bg-white/5 object-cover"
              />
              {#if seen}
                <span
                  class="absolute -right-1.5 -top-1.5 grid h-3.5 min-w-3.5 place-items-center rounded-full bg-violet-500 px-0.5 text-[8px] font-bold leading-none text-white ring-2 ring-[#0b1220]"
                  >{seen.lobbies}×</span
                >
              {/if}
              <span
                class="absolute -bottom-1 left-1/2 -translate-x-1/2 rounded bg-[#0b1220] px-1 text-[7px] font-semibold leading-[10px] ring-1 {offRole
                  ? 'text-amber-300 ring-amber-400/40'
                  : 'text-blue-300 ring-blue-400/20'}">{badgeLabel}</span
              >
            </div>
          {:else}
            <div
              class="relative grid h-6 min-w-7 shrink-0 place-items-center rounded-md px-1 text-[9px] font-semibold {offRole
                ? 'bg-amber-500/10 text-amber-300'
                : 'bg-blue-500/10 text-blue-300'}"
              title={badgeTitle}
            >
              {badgeLabel}
              {#if seen}
                <span
                  class="absolute -right-1.5 -top-1.5 grid h-3.5 min-w-3.5 place-items-center rounded-full bg-violet-500 px-0.5 text-[8px] font-bold leading-none text-white ring-2 ring-[#0b1220]"
                  >{seen.lobbies}×</span
                >
              {/if}
            </div>
          {/if}
          <div class="min-w-0 flex-1">
            <div class="truncate text-xs font-medium leading-tight">
              {participant.game_name}<span class="text-muted-foreground"
                >#{participant.game_tag}</span
              >
            </div>
            {#if participant.rank}
              {@const rank = formatRank(participant.rank, participant.recent)}
              <div
                class="truncate text-[10px] leading-tight text-muted-foreground"
                title="Solo/Duo: {rank.summary}"
              >
                <img
                  src={rank.icon}
                  alt=""
                  class="mr-1 inline-block h-3 w-3.5 object-contain align-[-2px]"
                /><span class="font-medium {rank.tierClass}">{rank.shortTier}</span
                >{#each rank.details as detail}<span>{` · ${detail}`}</span>{/each}
              </div>
            {/if}
          </div>
          {#if participant.mains?.length}
            <div class="flex shrink-0 -space-x-1" aria-label="Most played champions">
              {#each participant.mains as main (main.championId)}
                <img
                  src={championIconUrl(main.championId)}
                  alt={main.name}
                  title={describeMain(main)}
                  loading="lazy"
                  class="h-5 w-5 rounded-full border border-[#0b1220] bg-white/5 object-cover"
                />
              {/each}
            </div>
          {/if}
        </button>
      {/each}
    {:else}
      {#each Array.from({ length: 5 }) as _, index (index)}
        <div
          class="reveal-panel flex h-10 shrink-0 animate-pulse items-center gap-2.5 px-3"
        >
          <div class="h-6 w-6 rounded-md bg-white/5" />
          <div class="h-2.5 w-28 rounded-full bg-white/5" />
        </div>
      {/each}
    {/if}
  </div>

  <div class="mt-auto flex gap-2">
    <Button
      variant="outline"
      class="h-9 shrink-0 border-white/10 bg-white/[0.035] px-3 text-xs shadow-none hover:bg-white/[0.07]"
      disabled={!champSelect}
      on:click={copyNames}
      title="Copy all Riot IDs"
    >
      <!-- All labels share one grid cell so the button keeps the widest label's size. -->
      <span class="grid">
        {#each COPY_LABELS as [status, label]}
          <span
            class="col-start-1 row-start-1 text-center"
            class:invisible={copyStatus !== status}
            aria-hidden={copyStatus !== status}>{label}</span
          >
        {/each}
      </span>
    </Button>
    <Button
      class="h-9 flex-1 bg-blue-500 text-sm shadow-[0_8px_24px_rgba(59,130,246,0.18)] hover:bg-blue-400"
      disabled={!champSelect}
      on:click={openMultiLink}
    >
      Open team multi-search
      <svg
        viewBox="0 0 16 16"
        class="ml-2 h-3.5 w-3.5"
        aria-hidden="true"
      >
        <path
          d="M6 3h7v7m0-7L6.5 9.5M11 9v3a1 1 0 0 1-1 1H4a1 1 0 0 1-1-1V6a1 1 0 0 1 1-1h3"
          fill="none"
          stroke="currentColor"
          stroke-linecap="round"
          stroke-linejoin="round"
          stroke-width="1.25"
        />
      </svg>
    </Button>
  </div>
</section>

{#if selectedPlayer}
  <PlayerDetails
    participant={selectedPlayer}
    seen={seenBefore(selectedPlayer)}
    onClose={() => (selectedPuuid = null)}
  />
{/if}

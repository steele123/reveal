<script lang="ts">
  import {
    championIconUrl,
    formatAssignedPosition,
    formatRank,
    offRoleReason,
    positionBadge,
    profileIconUrl,
    type Participant,
  } from "$lib/champ_select";
  import { describeSeenBefore, type SeenBefore } from "$lib/lobby_history";
  import { logFrontendError } from "$lib/logging";
  import {
    csLead,
    describeMatchup,
    draftLanes,
    focusedEnemy,
    formatLead,
    formatWinRate,
    leadClass,
    MIN_RELIABLE_GAMES,
    setEnemyRole,
    winRateEdge,
    type CountersProvider,
    type DraftLane,
    type MatchupBoard,
    type Role,
  } from "$lib/matchups";
  import CountersTable from "./counters-table.svelte";

  export let allies: Participant[] = [];
  /** Enemy players, in the lobbies where Riot shows their names. */
  export let enemyPlayers: Participant[] = [];
  export let board: MatchupBoard | null = null;
  /** The saved choice; the board says which site its numbers are from. */
  export let provider: CountersProvider = "lolalytics";
  export let seenBefore: (participant: Participant) => SeenBefore | undefined = () => undefined;
  export let onSelectPlayer: (participant: Participant) => void = () => {};

  $: shownProvider = board?.provider ?? provider;
  /** Lanes the user dragged enemy picks to, by champion id. */
  let overrides = new Map<number, Role>();
  $: lanes = draftLanes(allies, enemyPlayers, board, overrides);
  let dragging: number | null = null;
  let dropRole: Role | null = null;

  function onDragStart(event: DragEvent, championId: number) {
    dragging = championId;
    event.dataTransfer?.setData("text/plain", String(championId));
    if (event.dataTransfer) event.dataTransfer.effectAllowed = "move";
  }

  function onDrop(role: Role | null) {
    const championId = dragging;
    dragging = null;
    dropRole = null;
    if (championId === null || role === null) return;
    overrides = new Map(overrides).set(championId, role);
    pickedId = championId;
    setEnemyRole(championId, role).catch((error) =>
      logFrontendError("Failed to move the enemy pick", error),
    );
  }
  /** The enemy you clicked; until then, the one in your lane. */
  let pickedId: number | null = null;
  $: focused = focusedEnemy(lanes, pickedId);

  // The matchup side of each row; the header uses the same widths so the columns line up.
  const MATCHUP_COLUMNS = "grid-cols-[14px_minmax(0,1fr)_44px_36px_32px_42px]";

  function laneLabel(lane: DraftLane): string {
    if (lane.role) return positionBadge(lane.role);
    return lane.player ? "—" : "?";
  }

  function matchupTitle(lane: DraftLane): string {
    const enemy = lane.enemy;
    if (!enemy) return "";
    const stats = enemy.ally?.stats;
    const matchup =
      enemy.ally && stats ? `${describeMatchup(stats, enemy.ally.champion, enemy.champion)}\n` : "";
    const guessed = enemy.roleGuessed
      ? `\nRiot hides enemy roles, so ${enemy.champion.name}'s lane is a best guess. Drag it to another lane if it's played there.`
      : "\nDrag it to another lane if it's played there.";
    return `${matchup}Click to see the best counters to ${enemy.champion.name}.${guessed}`;
  }

  function playerTitle(lane: DraftLane): string {
    const player = lane.player;
    if (!player) return "";
    const seen = seenBefore(player);
    return [
      lane.role &&
        (player.position_guessed
          ? `Likely ${formatAssignedPosition(lane.role)}, guessed from their recent roles, summoner spells and champion (Riot hides this in ranked)`
          : formatAssignedPosition(lane.role)),
      offRoleReason(player),
      seen && describeSeenBefore(seen),
      player.rank && `Solo/Duo: ${formatRank(player.rank, player.recent).summary}`,
    ]
      .filter(Boolean)
      .join(" · ");
  }

  /** A lane matchup with this few games says nothing, so it gets a note instead of numbers. */
  const MIN_LANE_GAMES = 20;

  function shownStats(lane: DraftLane) {
    const stats = lane.enemy?.ally?.stats ?? null;
    return stats && stats.games >= MIN_LANE_GAMES ? stats : null;
  }

  /** What the stat columns say when there are no numbers to show. */
  function statsNote(lane: DraftLane): string {
    if (!lane.enemy) return "";
    if (lane.enemy.loading) return "Loading…";
    if (!lane.enemy.ally) return "Waiting for your pick";
    const games = lane.enemy.ally.stats?.games ?? 0;
    if (games > 0) return `Only ${games} Master+ game${games === 1 ? "" : "s"}`;
    return "No Master+ games";
  }
</script>

<div class="flex min-h-0 flex-1 flex-col gap-1.5">
  <div class="reveal-scrollbar -mr-1 flex min-h-0 flex-1 flex-col gap-1.5 overflow-y-auto pr-1">
    {#if board?.error}
      <div class="shrink-0 rounded-lg border border-red-400/20 bg-red-500/10 px-3 py-1.5 text-[11px] text-red-300">
        {board.error}. Reveal will try again next Champ Select.
      </div>
    {/if}

    <div class="reveal-panel shrink-0 py-1">
      <div
        class="flex items-center gap-2 px-2 pb-0.5 text-[9px] font-medium uppercase tracking-[0.08em] text-muted-foreground"
      >
        <span class="w-7 shrink-0">Lane</span>
        <span class="min-w-0 flex-1 text-blue-300">Your team</span>
        <span class="grid w-[300px] shrink-0 {MATCHUP_COLUMNS} items-center gap-x-1.5">
          <span></span>
          <span class="text-red-300">Enemy</span>
          <span class="text-right" title="Your teammate's win rate against this enemy">Win</span>
          <span class="text-right" title="Kill lead at 15 minutes">Kills</span>
          <span class="text-right" title="Farm lead at 15 minutes: lane minions plus jungle camps">CS</span>
          <span class="text-right" title="Gold lead at 15 minutes">Gold</span>
        </span>
      </div>

      {#each lanes as lane, index (lane.role ?? lane.player?.puuid ?? `enemy-${index}`)}
        {@const player = lane.player}
        {@const enemy = lane.enemy}
        {@const stats = shownStats(lane)}
        {@const offRole = player ? offRoleReason(player) : null}
        {@const seen = player ? seenBefore(player) : undefined}
        <div
          class="flex items-center gap-2 border-t border-white/[0.04] px-2 py-[3px] {lane.isSelf
            ? 'bg-blue-500/[0.06]'
            : ''}"
        >
          <span
            class="w-7 shrink-0 text-[9px] font-semibold {offRole
              ? 'text-red-300'
              : lane.isSelf
                ? 'text-blue-300'
                : 'text-muted-foreground'}"
            >{laneLabel(lane)}{player?.position_guessed ? "?" : ""}</span
          >

          <button
            type="button"
            class="flex min-w-0 flex-1 items-center gap-2 rounded-md py-0.5 text-left transition enabled:hover:bg-white/[0.04] disabled:cursor-default"
            disabled={!player}
            title={playerTitle(lane)}
            aria-label={player ? `Show details for ${player.game_name}` : undefined}
            on:click={() => player && onSelectPlayer(player)}
          >
            {#if player}
              <span class="relative shrink-0">
                {#if player.profile_icon_id !== undefined}
                  <img
                    src={profileIconUrl(player.profile_icon_id)}
                    alt=""
                    class="h-7 w-7 rounded-md bg-white/5 object-cover"
                  />
                {:else}
                  <span class="block h-7 w-7 rounded-md bg-white/5"></span>
                {/if}
                {#if seen}
                  <span
                    class="absolute -right-1.5 -top-1.5 grid h-3.5 min-w-3.5 place-items-center rounded-full bg-violet-500 px-0.5 text-[8px] font-bold leading-none text-white ring-2 ring-[#0b1220]"
                    >{seen.lobbies}×</span
                  >
                {/if}
              </span>
              <span class="min-w-0 flex-1">
                <span class="flex min-w-0 items-center gap-1.5 text-xs font-medium leading-tight">
                  <span class="truncate" class:text-red-300={offRole} class:text-blue-300={!offRole && lane.isSelf}
                    >{player.game_name}<span class="text-muted-foreground">#{player.game_tag}</span></span
                  >
                  {#if offRole}
                    <span
                      class="shrink-0 rounded bg-red-500/15 px-1 text-[8px] font-bold uppercase leading-[13px] tracking-wide text-red-300 ring-1 ring-red-400/30"
                      >{player.autofilled ? "Fill" : "Off-role"}</span
                    >
                  {/if}
                </span>
                {#if player.rank}
                  {@const rank = formatRank(player.rank, player.recent)}
                  <span class="block truncate text-[10px] leading-tight text-muted-foreground">
                    <img src={rank.icon} alt="" class="mr-1 inline-block h-3 w-3.5 object-contain align-[-2px]" /><span
                      class="font-medium {rank.tierClass}">{rank.shortTier}</span
                    >{#each rank.details as detail}<span>{` · ${detail}`}</span>{/each}
                  </span>
                {/if}
              </span>
            {:else}
              <span class="h-7 w-7 shrink-0 rounded-md border border-dashed border-white/10"></span>
              <span class="min-w-0 flex-1 truncate text-[11px] text-muted-foreground">
                {lane.isSelf ? "You" : lane.role ? "Not revealed yet" : ""}
              </span>
            {/if}
            {#if lane.pick}
              <img
                src={championIconUrl(lane.pick.champion.id)}
                alt={lane.pick.champion.name}
                title={lane.pick.champion.name}
                class="h-6 w-6 shrink-0 rounded-md bg-white/5 object-cover ring-1 ring-blue-400/40"
              />
            {/if}
          </button>

          <button
            type="button"
            class="grid w-[300px] shrink-0 {MATCHUP_COLUMNS} items-center gap-x-1.5 rounded-md py-1 text-left text-[11px] transition enabled:hover:bg-white/[0.04] disabled:cursor-default {enemy &&
            enemy === focused
              ? 'bg-white/[0.07]'
              : ''}"
            class:ring-1={dropRole !== null && dropRole === lane.role}
            class:ring-blue-400={dropRole !== null && dropRole === lane.role}
            class:opacity-40={dragging !== null && enemy?.champion.id === dragging}
            disabled={!enemy && dragging === null}
            draggable={enemy !== null && lane.role !== null}
            title={matchupTitle(lane)}
            on:click={() => {
              if (enemy) pickedId = enemy.champion.id;
            }}
            on:dragstart={(event) => enemy && onDragStart(event, enemy.champion.id)}
            on:dragend={() => {
              dragging = null;
              dropRole = null;
            }}
            on:dragover|preventDefault={() => {
              if (dragging !== null && lane.role) dropRole = lane.role;
            }}
            on:dragleave={() => {
              if (dropRole === lane.role) dropRole = null;
            }}
            on:drop|preventDefault={() => onDrop(lane.role)}
          >
            <span class="text-center text-[9px] text-muted-foreground">vs</span>
            <span class="flex min-w-0 items-center gap-1.5">
              {#if enemy}
                <img
                  src={championIconUrl(enemy.champion.id)}
                  alt=""
                  class="h-6 w-6 shrink-0 rounded-md bg-white/5 object-cover ring-1 ring-red-400/40"
                />
                <span class="min-w-0">
                  <span class="block truncate leading-tight text-red-200">{enemy.champion.name}</span>
                  {#if lane.enemyPlayer}
                    <span class="block truncate text-[9px] leading-tight text-muted-foreground"
                      >{lane.enemyPlayer.game_name}</span
                    >
                  {/if}
                </span>
              {:else}
                <span class="h-6 w-6 shrink-0 rounded-md border border-dashed border-white/10"></span>
                <span class="truncate text-muted-foreground"
                  >{lane.enemyPlayer?.game_name ?? (lane.role ? "Not picked" : "")}</span
                >
              {/if}
            </span>
            {#if stats}
              <span
                class="text-right font-semibold tabular-nums {leadClass(winRateEdge(stats), 1)}"
                class:opacity-60={stats.games < MIN_RELIABLE_GAMES}
                >{formatWinRate(stats.winRate)}</span
              >
              <span class="text-right tabular-nums {leadClass(stats.killDiff15, 0.05)}"
                >{formatLead(stats.killDiff15, 1)}</span
              >
              <span class="text-right tabular-nums {leadClass(csLead(stats), 1)}"
                >{formatLead(csLead(stats))}</span
              >
              <span class="text-right tabular-nums {leadClass(stats.goldDiff15, 50)}"
                >{formatLead(stats.goldDiff15)}</span
              >
            {:else}
              <span class="col-span-4 truncate text-right text-[10px] text-muted-foreground"
                >{statsNote(lane)}</span
              >
            {/if}
          </button>
        </div>
      {/each}
    </div>

    {#if focused}
      <CountersTable enemy={focused} provider={shownProvider} />
    {/if}

    <!-- Shown from the first enemy pick, with the stats it dates. -->
    {#if board?.enemies.length && board.patch}
      <div class="shrink-0 px-1 pb-0.5 text-right text-[9px] text-muted-foreground">
        Master+ · Patch {board.patch}
      </div>
    {/if}
  </div>
</div>

<script lang="ts">
  import { championIconUrl, formatRank } from "$lib/champ_select";
  import {
    formatGameTime,
    splitTeams,
    teamKills,
    type LiveGame,
    type LivePlayer,
  } from "$lib/live_game";
  import {
    describeSeenBefore,
    participantKey,
    seenBeforeIndex,
    type LobbyHistoryEntry,
  } from "$lib/lobby_history";

  export let game: LiveGame;
  export let history: LobbyHistoryEntry[] = [];

  $: teams = splitTeams(game);
  $: seenIndex = seenBeforeIndex(history, null);
  $: seenBefore = (player: LivePlayer) =>
    player.isSelf || !player.puuid
      ? undefined
      : seenIndex.get(
          participantKey({
            puuid: player.puuid,
            game_name: player.gameName,
            game_tag: player.tagLine,
            region: "",
            cid: "",
            muted: false,
            pid: "",
          }),
        );

  // Inventory slots 0-5 are items, 6 is the trinket; show all seven in place.
  function slots(player: LivePlayer) {
    return Array.from({ length: 7 }, (_, slot) =>
      player.items.find((item) => item.slot === slot),
    );
  }
</script>

<section class="flex min-h-0 flex-1 flex-col gap-1">
  <div class="flex items-center justify-between px-1">
    <div class="text-xs text-muted-foreground">
      <span class="font-semibold text-foreground">Live game</span>
      · {game.gameMode === "CLASSIC" ? "Summoner's Rift" : game.gameMode} · {formatGameTime(game.gameTime)}
    </div>
    <div class="flex items-center gap-2 text-sm font-semibold tabular-nums">
      <span class="text-blue-300">{teamKills(teams.ally)}</span>
      <span class="text-[10px] font-medium text-muted-foreground">kills</span>
      <span class="text-red-300">{teamKills(teams.enemy)}</span>
    </div>
  </div>

  <div class="flex min-h-0 flex-col gap-1 overflow-y-auto">
    {#each [{ label: "Your team", players: teams.ally, tone: "text-blue-300" }, { label: "Enemy team", players: teams.enemy, tone: "text-red-300" }] as side}
      <div class="flex flex-col gap-0.5">
        <div class="px-1 text-[10px] font-medium uppercase tracking-[0.12em] {side.tone}">
          {side.label}
        </div>
        {#each side.players as player (player.gameName + player.tagLine + player.championName)}
          {@const rank = player.rank ? formatRank(player.rank, player.recent ?? undefined) : null}
          {@const seen = seenBefore(player)}
          <div
            class="reveal-panel flex h-[26px] shrink-0 items-center gap-2 px-2 text-[11px]"
            class:border-blue-400={player.isSelf}
          >
            <div class="relative shrink-0" title="{player.championName} · level {player.level}">
              {#if player.championId}
                <img
                  src={championIconUrl(player.championId)}
                  alt={player.championName}
                  class="h-5 w-5 rounded"
                  class:grayscale={player.isDead}
                  class:opacity-50={player.isDead}
                />
              {:else}
                <div class="h-5 w-5 rounded bg-white/5" />
              {/if}
              {#if player.isDead && player.respawnTimer > 0}
                <span
                  class="absolute inset-0 grid place-items-center text-[9px] font-bold text-white"
                  title="Respawns in {player.respawnTimer}s">{player.respawnTimer}</span
                >
              {:else}
                <span
                  class="absolute -bottom-0.5 -right-1 rounded bg-[#0b1220] px-0.5 text-[7px] font-semibold leading-[9px] text-muted-foreground"
                  >{player.level}</span
                >
              {/if}
            </div>

            <div class="flex min-w-0 flex-1 items-center gap-1">
              <span class="truncate font-medium" class:text-blue-200={player.isSelf}
                >{player.gameName}<span class="text-muted-foreground"
                  >{player.tagLine ? `#${player.tagLine}` : ""}</span
                ></span
              >
              {#if seen}
                <span
                  class="shrink-0 rounded-full bg-violet-500 px-1 text-[8px] font-bold leading-[12px] text-white"
                  title={describeSeenBefore(seen)}>{seen.lobbies}×</span
                >
              {/if}
            </div>

            <div class="w-[116px] shrink-0 truncate text-[10px] text-muted-foreground" title={rank ? `Solo/Duo: ${rank.summary}` : ""}>
              {#if rank}
                <img src={rank.icon} alt="" class="mr-0.5 inline-block h-2.5 w-3 object-contain align-[-1px]" /><span
                  class="font-medium {rank.tierClass}">{rank.shortTier}</span
                >{#if rank.details.length}<span>{` · ${rank.details[rank.details.length - 1]}`}</span>{/if}
              {:else if player.isBot}
                Bot
              {/if}
            </div>

            <div class="w-[48px] shrink-0 text-right font-medium tabular-nums" title="Kills / deaths / assists">
              {player.kills}/<span class="text-red-300">{player.deaths}</span>/{player.assists}
            </div>
            <div class="w-[30px] shrink-0 text-right tabular-nums text-muted-foreground" title="Creep score">
              {player.creepScore}
            </div>

            <div class="flex shrink-0 gap-px">
              {#each slots(player) as item, slot}
                {#if item?.icon}
                  <img src={item.icon} alt={item.name} title={item.name} class="h-4 w-4 rounded-sm" class:rounded-full={slot === 6} />
                {:else}
                  <div class="h-4 w-4 rounded-sm bg-white/[0.04]" class:rounded-full={slot === 6} />
                {/if}
              {/each}
            </div>
          </div>
        {/each}
      </div>
    {/each}
  </div>
</section>

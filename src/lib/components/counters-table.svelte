<script lang="ts">
  import { championIconUrl, formatAssignedPosition } from "$lib/champ_select";
  import { openExternalLink } from "$lib/link_actions";
  import { logFrontendError } from "$lib/logging";
  import {
    COUNTERS_PROVIDERS,
    csLead,
    describeMatchup,
    formatLead,
    formatWinRate,
    ladderEyeCountersLink,
    leadClass,
    leagueOfGraphsCountersLink,
    lolalyticsCountersLink,
    matchupLink,
    MIN_RELIABLE_GAMES,
    uggCountersLink,
    type ChampionRef,
    type CountersProvider,
    type EnemyMatchups,
    type MatchupStats,
    type SourceStats,
  } from "$lib/matchups";

  import InfoTip from "./info-tip.svelte";

  /** The enemy pick whose counters to list. */
  export let enemy: EnemyMatchups;
  /** The site the numbers came from. */
  export let provider: CountersProvider = "lolalytics";

  const COLUMNS = "grid-cols-[minmax(0,1fr)_46px_46px_46px_38px_32px_42px_42px_24px]";

  $: providerLabel = COUNTERS_PROVIDERS.find(({ value }) => value === provider)?.label ?? "Lolalytics";

  function open(link: string) {
    openExternalLink(link).catch((error) =>
      logFrontendError("Failed to open matchup page", error),
    );
  }

  /** The matchup's page where its numbers came from, when the client gave both names. */
  function openMatchup(champion: ChampionRef) {
    if (!enemy.role || !champion.slug || !enemy.champion.slug) return;
    open(matchupLink(provider, champion, enemy.champion, enemy.role));
  }

  /** One site's win rate, green or red by its own Δ. */
  function siteClass(side: SourceStats | null): string {
    return side ? leadClass(side.winRateDelta ?? side.winRate - 50, 1) : "text-muted-foreground";
  }

  function agreementLabel(stats: MatchupStats): { mark: string; tone: string; title: string } {
    const { agreement, sources } = stats;
    if (sources < 2) {
      return { mark: "·", tone: "text-muted-foreground", title: "Only one site has loaded so far" };
    }
    if (agreement === sources) {
      return {
        mark: "✓✓",
        tone: "text-emerald-300",
        title: `A real counter: on all ${sources} sites it wins this matchup more than its overall strength predicts`,
      };
    }
    if (agreement > 0) {
      return {
        mark: "✓",
        tone: "text-amber-300",
        title: `${agreement} of ${sources} sites have it winning more than its overall strength predicts (the others disagree or lack enough games)`,
      };
    }
    return {
      mark: "–",
      tone: "text-muted-foreground",
      title: "Not a counter as such: it wins this matchup often because it's strong overall, not because it does especially well here",
    };
  }

  function sources(enemy: EnemyMatchups) {
    if (!enemy.role || !enemy.champion.slug) return [];
    return [
      {
        label: "U.GG",
        title: `${enemy.champion.name} counters on U.GG (Master+)`,
        link: uggCountersLink(enemy.champion, enemy.role),
      },
      {
        label: "Lolalytics",
        title: `${enemy.champion.name} counters on Lolalytics (Master+)`,
        link: lolalyticsCountersLink(enemy.champion, enemy.role),
      },
      {
        label: "LadderEye",
        title: `${enemy.champion.name} counters on LadderEye (Master+)`,
        link: ladderEyeCountersLink(enemy.champion, enemy.role),
      },
      {
        label: "League of Graphs",
        title: `${enemy.champion.name} counters on League of Graphs (Master). It blocks apps, so Reveal can only link to it`,
        link: leagueOfGraphsCountersLink(enemy.champion, enemy.role),
      },
    ];
  }
</script>

<div class="reveal-panel shrink-0 py-1">
  <div class="flex items-center justify-between gap-2 px-2 pb-1">
    <div class="min-w-0 truncate text-[10px] font-semibold uppercase tracking-[0.08em]">
      Best counters vs <span class="text-red-300">{enemy.champion.name}</span>
      {#if enemy.role}
        <span class="font-medium normal-case tracking-normal text-muted-foreground"
          >· {formatAssignedPosition(enemy.role)}</span
        >
      {/if}
    </div>
    <div class="flex shrink-0 gap-1">
      {#each sources(enemy) as source (source.label)}
        <button
          type="button"
          class="rounded px-1.5 py-0.5 text-[9px] font-semibold text-muted-foreground ring-1 ring-white/10 transition hover:bg-white/[0.06] hover:text-foreground"
          title={source.title}
          on:click={() => open(source.link)}>{source.label} ↗</button
        >
      {/each}
    </div>
  </div>
  <div
    class="grid {COLUMNS} gap-x-1.5 px-2 pb-0.5 text-[9px] font-medium uppercase tracking-[0.08em] text-muted-foreground"
  >
    <span>Pick into {enemy.champion.name}</span>
    <span class="text-right" class:text-foreground={provider === "lolalytics"} title="Lolalytics win rate against this enemy">Lola</span>
    <span class="text-right" class:text-foreground={provider === "ugg"} title="U.GG win rate against this enemy">U.GG</span>
    <span class="text-right" class:text-foreground={provider === "laddereye"} title="LadderEye win rate against this enemy">Ladder</span>
    <span class="text-right" title="U.GG kill lead at 15 minutes">Kills</span>
    <span class="text-right" title="U.GG farm lead at 15 minutes: lane minions plus jungle camps">CS</span>
    <span class="text-right" title="U.GG gold lead at 15 minutes">Gold</span>
    <span class="text-right" title="Master+ games of this matchup on {providerLabel}">Games</span>
    <span class="text-center"
      ><InfoTip
        align="right"
        text={`Sorted by how often each pick wins this matchup, averaged over Lolalytics, U.GG and LadderEye, with small samples pulled towards what both champions usually win. ✓✓ marks real counters: they win it more than their overall strength predicts on every site. Lola, U.GG and Ladder are each site's win rate (LadderEye only lists each enemy's 40 most common opponents); kills, CS and gold are U.GG's leads at 15 minutes; faded rows have under ${MIN_RELIABLE_GAMES} games. Click a pick for its matchup page.`}
      /></span
    >
  </div>
  {#if enemy.counters.length}
    {#each enemy.counters as counter (counter.champion.id)}
      {@const stats = counter.stats}
      {@const agreement = agreementLabel(stats)}
      <button
        type="button"
        class="grid w-full {COLUMNS} items-center gap-x-1.5 border-t border-white/[0.04] px-2 py-[2px] text-left text-[11px] transition hover:bg-white/[0.05]"
        class:opacity-60={stats.games < MIN_RELIABLE_GAMES}
        title={`${describeMatchup(stats, counter.champion, enemy.champion)}\nClick for the matchup on ${providerLabel}`}
        on:click={() => openMatchup(counter.champion)}
      >
        <span class="flex min-w-0 items-center gap-1.5">
          <img
            src={championIconUrl(counter.champion.id)}
            alt=""
            loading="lazy"
            class="h-4 w-4 shrink-0 rounded-sm bg-white/5 object-cover"
          />
          <span class="truncate">{counter.champion.name}</span>
        </span>
        <span
          class="text-right tabular-nums {siteClass(stats.lolalytics)}"
          class:font-semibold={provider === "lolalytics"}
          >{stats.lolalytics ? formatWinRate(stats.lolalytics.winRate) : "–"}</span
        >
        <span
          class="text-right tabular-nums {siteClass(stats.ugg)}"
          class:font-semibold={provider === "ugg"}
          >{stats.ugg ? formatWinRate(stats.ugg.winRate) : "–"}</span
        >
        <span
          class="text-right tabular-nums {siteClass(stats.laddereye)}"
          class:font-semibold={provider === "laddereye"}
          >{stats.laddereye ? formatWinRate(stats.laddereye.winRate) : "–"}</span
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
        <span class="text-right tabular-nums text-muted-foreground"
          >{stats.games.toLocaleString("en-US")}</span
        >
        <span class="group relative text-center text-[10px] font-bold {agreement.tone}"
          >{agreement.mark}<span
            role="tooltip"
            class="pointer-events-none absolute bottom-full right-0 z-50 mb-1 hidden w-60 rounded-lg border border-white/10 bg-[#0f1726] px-2.5 py-2 text-left text-[10px] font-normal leading-relaxed text-foreground shadow-xl group-hover:block"
            >{agreement.title}</span
          ></span
        >
      </button>
    {/each}
  {:else if enemy.loading}
    {#each Array.from({ length: 5 }) as _, index (index)}
      <div class="mx-2 my-[3px] h-[14px] animate-pulse rounded bg-white/[0.04]"></div>
    {/each}
  {:else}
    <div class="px-2 py-1 text-[10px] text-muted-foreground">
      Not enough Master+ games to call counters.
    </div>
  {/if}
</div>

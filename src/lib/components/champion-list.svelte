<script lang="ts">
  import { championIconUrl } from "$lib/champ_select";
  import { searchChampions, type ChampionName } from "$lib/champions";

  /** Champion ids in order of preference. */
  export let ids: number[] = [];
  export let champions: ChampionName[] = [];
  export let id: string;
  export let placeholder = "Add a champion";
  export let disabled = false;
  export let onChange: (ids: number[]) => void = () => {};

  const MAX_CHAMPIONS = 8;
  let typed = "";
  let open = false;
  let highlighted = 0;

  $: names = new Map(champions.map((champion) => [champion.id, champion.name]));
  $: matches = searchChampions(champions, typed).filter((champion) => !ids.includes(champion.id));
  $: if (highlighted >= matches.length) highlighted = 0;

  function add(champion: ChampionName | undefined) {
    if (!champion) return;
    typed = "";
    highlighted = 0;
    if (ids.includes(champion.id) || ids.length >= MAX_CHAMPIONS) return;
    onChange([...ids, champion.id]);
  }

  function remove(championId: number) {
    onChange(ids.filter((id) => id !== championId));
  }

  /** Moves a champion one place earlier in the list. */
  function raise(index: number) {
    if (index === 0) return;
    const next = [...ids];
    [next[index - 1], next[index]] = [next[index], next[index - 1]];
    onChange(next);
  }

  function onKeydown(event: KeyboardEvent) {
    if (event.key === "ArrowDown") {
      event.preventDefault();
      open = true;
      highlighted = Math.min(highlighted + 1, matches.length - 1);
    } else if (event.key === "ArrowUp") {
      event.preventDefault();
      highlighted = Math.max(highlighted - 1, 0);
    } else if (event.key === "Enter") {
      event.preventDefault();
      add(matches[highlighted]);
    } else if (event.key === "Escape") {
      open = false;
    }
  }
</script>

<div class="flex flex-wrap items-center gap-1" class:opacity-50={disabled}>
  {#each ids as championId, index (championId)}
    <span
      class="flex items-center gap-1 rounded-md border border-white/10 bg-white/[0.04] py-0.5 pl-0.5 pr-1 text-[11px]"
    >
      <button
        type="button"
        class="flex items-center gap-1 rounded disabled:cursor-default"
        title={index === 0 ? "First choice" : "Move earlier"}
        disabled={index === 0}
        on:click={() => raise(index)}
      >
        <img
          src={championIconUrl(championId)}
          alt=""
          class="h-5 w-5 rounded-sm bg-white/5 object-cover"
        />
        <span class="text-[9px] font-semibold tabular-nums text-muted-foreground">{index + 1}</span>
        <span>{names.get(championId) ?? `Champion ${championId}`}</span>
      </button>
      <button
        type="button"
        class="ml-0.5 grid h-4 w-4 place-items-center rounded text-muted-foreground hover:bg-red-500/15 hover:text-red-300"
        aria-label={`Remove ${names.get(championId) ?? "champion"}`}
        on:click={() => remove(championId)}
      >
        <svg viewBox="0 0 16 16" class="h-2.5 w-2.5" aria-hidden="true">
          <path d="m4 4 8 8m0-8-8 8" fill="none" stroke="currentColor" stroke-width="1.75" />
        </svg>
      </button>
    </span>
  {/each}
  {#if ids.length < MAX_CHAMPIONS}
    <div class="relative">
      <input
        {id}
        class="h-7 w-[160px] rounded-md border border-white/10 bg-white/[0.035] px-2 text-[11px] placeholder:text-muted-foreground focus:border-blue-400/40 focus:outline-none"
        {placeholder}
        autocomplete="off"
        role="combobox"
        aria-expanded={open && matches.length > 0}
        aria-controls={`${id}-options`}
        bind:value={typed}
        on:focus={() => (open = true)}
        on:input={() => (open = true)}
        on:blur={() => (open = false)}
        on:keydown={onKeydown}
      />
      {#if open && typed.trim() && matches.length}
        <ul
          id={`${id}-options`}
          role="listbox"
          class="reveal-scrollbar absolute left-0 top-full z-50 mt-1 max-h-48 w-[200px] overflow-y-auto rounded-lg border border-white/10 bg-[#0f1726] p-1 shadow-xl"
        >
          {#each matches as champion, index (champion.id)}
            <li role="option" aria-selected={index === highlighted}>
              <!-- mousedown, so the pick lands before the input's blur closes the list -->
              <button
                type="button"
                tabindex="-1"
                class="flex w-full items-center gap-2 rounded-md px-1.5 py-1 text-left text-[11px] {index ===
                highlighted
                  ? 'bg-blue-500/15 text-blue-200'
                  : 'hover:bg-white/[0.06]'}"
                on:mousedown|preventDefault={() => add(champion)}
                on:mouseenter={() => (highlighted = index)}
              >
                <img
                  src={championIconUrl(champion.id)}
                  alt=""
                  loading="lazy"
                  class="h-5 w-5 shrink-0 rounded-sm bg-white/5 object-cover"
                />
                <span class="truncate">{champion.name}</span>
              </button>
            </li>
          {/each}
        </ul>
      {/if}
    </div>
  {/if}
</div>

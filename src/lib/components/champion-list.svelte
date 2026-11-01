<script lang="ts">
  import { tick } from "svelte";
  import { Popover } from "bits-ui";
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
  const GRID_COLUMNS = 6;
  let open = false;
  let typed = "";
  let highlighted = 0;
  let search: HTMLInputElement | undefined;
  let grid: HTMLDivElement | undefined;
  /** The order while a tile is being dragged, saved once it's dropped. */
  let dragOrder: number[] | null = null;
  let dragIndex: number | null = null;

  $: shown = dragOrder ?? ids;
  $: names = new Map(champions.map((champion) => [champion.id, champion.name]));
  $: matches = (typed.trim() ? searchChampions(champions, typed, Infinity) : champions).filter(
    (champion) => !ids.includes(champion.id),
  );
  $: if (highlighted >= matches.length) highlighted = 0;
  $: if (!open) typed = "";

  // Reactive, so the tiles get their names once the list loads after them.
  $: nameOf = (championId: number): string => names.get(championId) ?? `Champion ${championId}`;

  async function add(champion: ChampionName | undefined) {
    if (!champion || ids.includes(champion.id) || ids.length >= MAX_CHAMPIONS) return;
    onChange([...ids, champion.id]);
    typed = "";
    highlighted = 0;
    // Stay open for the next one, unless the list is now full.
    if (ids.length + 1 >= MAX_CHAMPIONS) open = false;
    else {
      await tick();
      search?.focus();
    }
  }

  function remove(championId: number) {
    onChange(ids.filter((id) => id !== championId));
  }

  function moved(list: number[], from: number, to: number): number[] {
    const next = [...list];
    const [championId] = next.splice(from, 1);
    next.splice(to, 0, championId);
    return next;
  }

  function onTileKeydown(event: KeyboardEvent, index: number) {
    const step = { ArrowLeft: -1, ArrowRight: 1 }[event.key];
    if (step) {
      event.preventDefault();
      const to = index + step;
      if (to < 0 || to >= ids.length) return;
      onChange(moved(ids, index, to));
      // Keep focus on the tile that moved.
      void tick().then(() =>
        (event.currentTarget as HTMLElement | null)?.parentElement
          ?.querySelectorAll<HTMLElement>("[data-tile]")
          [to]?.focus(),
      );
    } else if (event.key === "Delete" || event.key === "Backspace") {
      event.preventDefault();
      remove(ids[index]);
    }
  }

  function onDragStart(event: DragEvent, index: number) {
    dragIndex = index;
    dragOrder = [...ids];
    event.dataTransfer?.setData("text/plain", String(ids[index]));
    if (event.dataTransfer) event.dataTransfer.effectAllowed = "move";
  }

  function onDragOver(event: DragEvent, index: number) {
    if (dragIndex === null || !dragOrder) return;
    event.preventDefault();
    if (index !== dragIndex) {
      dragOrder = moved(dragOrder, dragIndex, index);
      dragIndex = index;
    }
  }

  function onDragEnd() {
    if (dragOrder && dragOrder.some((championId, index) => championId !== ids[index])) {
      onChange(dragOrder);
    }
    dragOrder = null;
    dragIndex = null;
  }

  async function onSearchKeydown(event: KeyboardEvent) {
    const step = {
      ArrowRight: 1,
      ArrowLeft: -1,
      ArrowDown: GRID_COLUMNS,
      ArrowUp: -GRID_COLUMNS,
    }[event.key];
    if (step) {
      event.preventDefault();
      highlighted = Math.min(Math.max(highlighted + step, 0), matches.length - 1);
      await tick();
      grid
        ?.querySelector(`[data-index="${highlighted}"]`)
        ?.scrollIntoView({ block: "nearest" });
    } else if (event.key === "Enter") {
      event.preventDefault();
      void add(matches[highlighted]);
    }
  }
</script>

<div
  class="flex flex-wrap items-start gap-1.5 transition-opacity"
  class:opacity-60={disabled}
  role="group"
  aria-label="Champions in order of preference"
>
  {#each shown as championId, index (championId)}
    <div
      data-tile
      role="button"
      aria-roledescription="sortable champion"
      tabindex="0"
      draggable="true"
      aria-label={`${nameOf(championId)}, choice ${index + 1}. Arrow keys reorder, Delete removes.`}
      title={`${nameOf(championId)} · drag to reorder`}
      class="group relative w-12 cursor-grab rounded-md outline-none focus-visible:ring-1 focus-visible:ring-blue-400 active:cursor-grabbing"
      class:opacity-40={dragIndex === index}
      on:dragstart={(event) => onDragStart(event, index)}
      on:dragover={(event) => onDragOver(event, index)}
      on:drop|preventDefault
      on:dragend={onDragEnd}
      on:keydown={(event) => onTileKeydown(event, index)}
    >
      <div class="relative h-12 w-12 overflow-hidden rounded-md border border-white/10 bg-white/5">
        <img
          src={championIconUrl(championId)}
          alt=""
          draggable="false"
          class="h-full w-full scale-110 object-cover"
        />
        <span
          class="absolute left-0 top-0 rounded-br-md bg-black/75 px-1 text-[9px] font-semibold tabular-nums leading-[14px] text-white"
        >
          {index + 1}
        </span>
      </div>
      <div class="mt-0.5 truncate text-center text-[9px] text-muted-foreground">
        {nameOf(championId)}
      </div>
      <button
        type="button"
        tabindex="-1"
        class="absolute -right-1.5 -top-1.5 hidden h-4 w-4 place-items-center rounded-full border border-white/15 bg-[#0f1726] text-muted-foreground shadow hover:border-red-400/40 hover:text-red-300 group-hover:grid group-focus-visible:grid"
        aria-label={`Remove ${nameOf(championId)}`}
        on:click={() => remove(championId)}
      >
        <svg viewBox="0 0 16 16" class="h-2 w-2" aria-hidden="true">
          <path d="m4 4 8 8m0-8-8 8" fill="none" stroke="currentColor" stroke-width="2" />
        </svg>
      </button>
    </div>
  {/each}

  {#if ids.length < MAX_CHAMPIONS}
    <Popover.Root bind:open>
      <Popover.Trigger
        {id}
        aria-label={placeholder}
        title={placeholder}
        class="grid h-12 w-12 place-items-center rounded-md border border-dashed border-white/15 text-muted-foreground transition hover:border-blue-400/40 hover:bg-blue-500/10 hover:text-blue-300 focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-blue-400 data-[state=open]:border-blue-400/40 data-[state=open]:text-blue-300"
      >
        <svg viewBox="0 0 16 16" class="h-4 w-4" aria-hidden="true">
          <path d="M8 3v10M3 8h10" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" />
        </svg>
      </Popover.Trigger>
      <Popover.Portal>
        <Popover.Content
          side="bottom"
          align="start"
          sideOffset={6}
          collisionPadding={8}
          onOpenAutoFocus={(event) => {
            event.preventDefault();
            search?.focus();
          }}
          class="z-50 w-[300px] rounded-lg border border-white/10 bg-[#0f1726] p-2 shadow-2xl focus:outline-none"
        >
          <input
            bind:this={search}
            bind:value={typed}
            class="h-8 w-full rounded-md border border-white/10 bg-white/[0.035] px-2 text-[11px] placeholder:text-muted-foreground focus:border-blue-400/40 focus:outline-none"
            placeholder="Search champions"
            autocomplete="off"
            spellcheck="false"
            aria-label="Search champions"
            aria-controls={`${id}-options`}
            on:input={() => (highlighted = 0)}
            on:keydown={onSearchKeydown}
          />
          <div
            bind:this={grid}
            id={`${id}-options`}
            role="listbox"
            aria-label="Champions"
            class="reveal-scrollbar mt-2 grid max-h-[228px] grid-cols-6 gap-1 overflow-y-auto pr-0.5"
          >
            {#each matches as champion, index (champion.id)}
              <button
                type="button"
                role="option"
                tabindex="-1"
                data-index={index}
                aria-selected={index === highlighted}
                title={champion.name}
                class="flex min-w-0 flex-col items-center rounded-md p-0.5 transition {index ===
                highlighted
                  ? 'bg-blue-500/15 text-blue-200'
                  : 'text-muted-foreground hover:bg-white/[0.06] hover:text-foreground'}"
                on:click={() => add(champion)}
                on:mouseenter={() => (highlighted = index)}
              >
                <img
                  src={championIconUrl(champion.id)}
                  alt=""
                  loading="lazy"
                  class="h-9 w-9 rounded bg-white/5 object-cover"
                />
                <span class="mt-0.5 w-full truncate text-center text-[8px] leading-tight">
                  {champion.name}
                </span>
              </button>
            {:else}
              <div class="col-span-6 py-6 text-center text-[10px] text-muted-foreground">
                {champions.length ? `No champion matches "${typed}"` : "Loading champions…"}
              </div>
            {/each}
          </div>
          <div class="mt-1.5 flex justify-between px-0.5 text-[9px] text-muted-foreground">
            <span>Enter adds the highlighted champion</span>
            <span class="tabular-nums">{ids.length}/{MAX_CHAMPIONS}</span>
          </div>
        </Popover.Content>
      </Popover.Portal>
    </Popover.Root>
  {/if}

  {#if ids.length === 0}
    <div class="self-center text-[10px] text-muted-foreground">
      {placeholder}, in order of preference
    </div>
  {:else if ids.length > 1}
    <div class="self-center text-[9px] text-muted-foreground/70">Drag to reorder</div>
  {/if}
</div>

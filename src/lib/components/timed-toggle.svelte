<script lang="ts">
  import { Popover } from "bits-ui";

  /**
   * An on/off setting with a time, as one pill: the dot switches it, the time beside it opens
   * the choices.
   */
  export let id: string | undefined = undefined;
  export let enabled = false;
  export let value: number;
  /** In order; `short` fits in the pill, `label` is for the menu. */
  export let options: { value: number; label: string; short: string }[];
  export let label: string;
  export let disabled = false;
  export let onToggle: (enabled: boolean) => void = () => {};
  export let onChange: (value: number) => void = () => {};

  let open = false;

  $: current = options.find((option) => option.value === value) ?? options[0];

  function choose(next: number) {
    open = false;
    onChange(next);
  }
</script>

<div
  class="flex h-7 shrink-0 items-center rounded-full border border-white/10 bg-white/[0.03] text-[11px] tabular-nums"
>
  <button
    {id}
    type="button"
    {disabled}
    role="switch"
    aria-checked={enabled}
    aria-label={label}
    title={enabled ? "On" : "Off"}
    class="grid h-full w-7 place-items-center rounded-l-full transition hover:bg-white/[0.05] focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-blue-400"
    on:click={() => onToggle(!enabled)}
  >
    <span
      class="h-2 w-2 rounded-full transition {enabled
        ? 'bg-emerald-400 shadow-[0_0_8px_rgba(52,211,153,0.8)]'
        : 'bg-white/20'}"
    ></span>
  </button>
  <span class="h-4 w-px bg-white/10"></span>
  <Popover.Root bind:open>
    <Popover.Trigger
      disabled={disabled || !enabled}
      aria-label={`${label}: ${current.label}`}
      title={current.label}
      class="flex h-full w-[54px] items-center justify-between gap-1 rounded-r-full pl-2 pr-1.5 font-medium transition hover:bg-white/[0.05] focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-blue-400 disabled:pointer-events-none {enabled
        ? 'text-foreground'
        : 'text-muted-foreground/50'}"
    >
      {current.short}
      <svg viewBox="0 0 16 16" class="h-3 w-3 opacity-60" aria-hidden="true">
        <path
          d="m4.5 6.5 3.5 3.5 3.5-3.5"
          fill="none"
          stroke="currentColor"
          stroke-width="1.5"
          stroke-linecap="round"
        />
      </svg>
    </Popover.Trigger>
    <Popover.Portal>
      <Popover.Content
        side="bottom"
        align="end"
        sideOffset={6}
        collisionPadding={8}
        class="z-50 w-[150px] rounded-lg border border-white/10 bg-[#0f1726] p-1 shadow-2xl focus:outline-none"
      >
        {#each options as option (option.value)}
          <button
            type="button"
            class="flex w-full items-center justify-between rounded-md px-2 py-1.5 text-left text-[11px] transition hover:bg-white/5 {option.value ===
            value
              ? 'text-blue-300'
              : ''}"
            on:click={() => choose(option.value)}
          >
            {option.label}
            {#if option.value === value}<span>✓</span>{/if}
          </button>
        {/each}
      </Popover.Content>
    </Popover.Portal>
  </Popover.Root>
</div>

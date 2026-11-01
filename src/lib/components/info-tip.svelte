<script lang="ts">
  /** How the setting works, shown while the icon is hovered or focused. */
  export let text: string;
  /** Which edge of the icon the box lines up with; "right" for icons near the right edge. */
  export let align: "left" | "right" = "left";
</script>

<span class="group relative ml-1 inline-flex align-[-2px]">
  <!-- Inside a setting's label, so a click must not reach the label and flip its switch. -->
  <button
    type="button"
    aria-label={text}
    class="grid h-3.5 w-3.5 cursor-help place-items-center rounded-full text-muted-foreground transition hover:text-foreground focus:text-foreground focus:outline-none"
    on:click|preventDefault|stopPropagation
  >
    <svg viewBox="0 0 16 16" class="h-3.5 w-3.5" aria-hidden="true">
      <circle cx="8" cy="8" r="6.25" fill="none" stroke="currentColor" stroke-width="1.25" />
      <path d="M8 7.25v4" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" />
      <circle cx="8" cy="4.9" r="0.85" fill="currentColor" />
    </svg>
  </button>
  <span
    role="tooltip"
    class="pointer-events-none absolute {align === 'right' ? 'right-0' : 'left-0'} top-full z-50 mt-1 hidden w-72 rounded-lg border border-white/10 bg-[#0f1726] px-2.5 py-2 text-[10px] font-normal normal-case leading-relaxed tracking-normal text-foreground shadow-xl group-focus-within:block group-hover:block"
  >
    {text}
  </span>
</span>

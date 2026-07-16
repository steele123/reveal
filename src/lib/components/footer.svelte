<script lang="ts">
  import { fade } from "svelte/transition";

  export let connected = false;
</script>

<footer
  class="flex h-10 shrink-0 items-center border-t border-white/10 px-5 text-xs"
>
  <div class="min-w-0">
    {#if connected}
      <div
        in:fade={{ duration: 250 }}
        class="flex items-center gap-2 text-muted-foreground"
      >
        <div class="h-1.5 w-1.5 rounded-full bg-emerald-400 shadow-[0_0_8px_rgba(52,211,153,0.7)]" />
        <div class="truncate">League connected</div>
      </div>
    {:else}
      <div
        in:fade={{ duration: 250 }}
        class="flex items-center gap-2 text-muted-foreground"
      >
        <div class="h-1.5 w-1.5 rounded-full bg-amber-400" />
        <div class="truncate">League unavailable</div>
      </div>
    {/if}
  </div>
  <div class="ml-auto">
    <a
      href="https://lobbyreveal.app/reveal"
      target="_blank"
      rel="noreferrer"
      class="group flex items-center gap-1.5 rounded-md px-2 py-1 text-muted-foreground transition hover:bg-white/5 hover:text-foreground"
    >
      Website
      <svg
        xmlns="http://www.w3.org/2000/svg"
        width="13"
        height="13"
        viewBox="0 0 24 24"
        fill="none"
        stroke="currentColor"
        stroke-width="2"
        stroke-linecap="round"
        stroke-linejoin="round"
        class="transition group-hover:text-blue-400"
        ><path d="M15 3h6v6" /><path d="M10 14 21 3" /><path
          d="M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6"
        /></svg
      >
    </a>
  </div>
</footer>

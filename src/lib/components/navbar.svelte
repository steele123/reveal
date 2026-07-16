<script lang="ts">
  import { onMount } from "svelte";
  import { getVersion } from "@tauri-apps/api/app";
  import { appWindow } from "@tauri-apps/api/window";
  import { isTauriRuntime } from "$lib/runtime";

  let version = "Preview";

  onMount(() => {
    if (!isTauriRuntime()) return;
    void getVersion().then((value) => (version = `v${value}`));
  });

  function minimize() {
    if (isTauriRuntime()) void appWindow.minimize();
  }

  function close() {
    if (isTauriRuntime()) void appWindow.close();
  }
</script>

<header
    data-tauri-drag-region
    class="flex h-12 w-full shrink-0 select-none items-center border-b border-white/10 px-4"
>
  <div data-tauri-drag-region class="flex min-w-0 items-center gap-2.5">
    <div
      class="grid h-7 w-7 shrink-0 place-items-center rounded-lg border border-blue-400/20 bg-blue-500/10"
    >
      <img alt="" src="/icon.png" class="h-5 w-5" />
    </div>
    <div data-tauri-drag-region class="flex items-baseline gap-2">
      <div class="text-sm font-semibold tracking-tight">Reveal</div>
      <div
        class="rounded-full border border-white/10 bg-white/5 px-1.5 py-0.5 text-[10px] font-medium text-muted-foreground"
      >
        {version}
      </div>
    </div>
  </div>

  <div class="ml-auto flex items-center gap-1">
    <button
      class="grid h-7 w-7 place-items-center rounded-md text-muted-foreground transition hover:bg-white/5 hover:text-foreground focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring"
      aria-label="Minimize Reveal"
      on:click={minimize}
    >
      <svg viewBox="0 0 16 16" class="h-3.5 w-3.5" aria-hidden="true">
        <path d="M3 8.5h10" fill="none" stroke="currentColor" stroke-width="1.5" />
      </svg>
    </button>
    <button
      class="grid h-7 w-7 place-items-center rounded-md text-muted-foreground transition hover:bg-red-500/15 hover:text-red-300 focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-red-400"
      aria-label="Close Reveal"
      on:click={close}
    >
      <svg viewBox="0 0 16 16" class="h-3.5 w-3.5" aria-hidden="true">
        <path d="m4 4 8 8m0-8-8 8" fill="none" stroke="currentColor" stroke-width="1.5" />
      </svg>
    </button>
  </div>
</header>

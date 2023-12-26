<script lang="ts">
  import { getVersion } from "@tauri-apps/api/app";
  import { appWindow } from "@tauri-apps/api/window";
  import { onMount } from "svelte";
  import { listen } from "@tauri-apps/api/event";
  import { invoke } from "@tauri-apps/api/tauri";
  import { type Config } from "./lib/config";
  import type { ChampSelect } from "./lib/champ_select";
  import { fade } from "svelte/transition";
  import Updater from "./components/updater.svelte";
  import Tool from "./components/tool.svelte";

  let state = "Unknown";
  let connected = false;
  let champSelect: ChampSelect | null = null;
  let config: Config | null = null;

  onMount(async () => {
    await listen<string>("client_state_update", (event) => {
      const newState = event.payload;
      if (newState === "ChampSelect") {
        champSelect = null;
      }

      state = newState;
    });

    await listen<boolean>("lcu_state_update", (event) => {
      connected = event.payload;
    });

    await listen<ChampSelect>("champ_select_started", (event) => {
      champSelect = event.payload;
    });

    invoke<Config>("app_ready").then((c) => {
      config = c;
    });
  });
</script>

<main class="h-[300px] bg-background border rounded-md">
  <Updater />
  <div
    data-tauri-drag-region
    class="flex border-b rounded-t-lg w-full select-none px-4 py-2"
  >
    <div class="flex items-center gap-2">
      <div>
        <img alt="" src="/icon.png" class="w-5 h-5" />
      </div>
      <div class="text-blue-500">reveal</div>
      {#await getVersion()}
        <div></div>
      {:then version}
        <div>v{version}</div>
      {:catch error}
        <div>{error.message}</div>
      {/await}
    </div>
    <div class="ml-auto flex gap-2">
      <button class="" on:click={async () => appWindow.minimize()}>-</button>
      <button class="text-xs" on:click={() => appWindow.close()}>X</button>
    </div>
  </div>
  <Tool {config} {state} {champSelect} />
  <div class="px-4 items-center flex pt-1 border-t">
    {#if connected}
      <div in:fade={{ duration: 2000 }} class="flex items-center gap-1 text-sm">
        <div class="bg-green-500 animate-pulse h-2 w-2 rounded-full" />
        <div>Connected to League Client</div>
      </div>
    {:else}
      <div in:fade={{ duration: 2000 }} class="flex items-center gap-1 text-sm">
        <div class="bg-red-500 animate-pulse h-2 w-2 rounded-full" />
        <div>Not Connected to League Client</div>
      </div>
    {/if}
    <a
      href="https://hyeb.gg/reveal"
      target="_blank"
      class="text-blue-500 ml-auto hover:underline text-xs">hyeb.gg/reveal</a
    >
  </div>
</main>

<script lang="ts">
  import { getVersion } from "@tauri-apps/api/app";
  import { appWindow } from "@tauri-apps/api/window";
  import Switch from "./components/switch.svelte";
  import Label from "./components/label.svelte";
  import { onMount } from "svelte";
  import { listen } from "@tauri-apps/api/event";
  import { invoke } from "@tauri-apps/api/tauri";
  import { updateConfig, type Config } from "./lib/config";
  import type { ChampSelect } from "./lib/champ_select";

  let state = "Unknown";
  let connected = false;
  let champSelect: ChampSelect | null = null;
  let config: Config | null = null;

  onMount(async () => {
    await listen<string>("client_state_update", (event) => {
      state = event.payload;
    });

    await listen<boolean>("lcu_state_update", (event) => {
      connected = event.payload;
    });

    await listen<ChampSelect>("champ_select_started", (event) => {
      champSelect = event.payload;
      console.log(champSelect);
    });

    setTimeout(() => {
      invoke<Config>("app_ready").then((c) => {
        console.log(c);
        config = c;
      });
    }, 3000);
  });
</script>

<main class="h-[300px] border bg-background rounded-md">
  <div
    data-tauri-drag-region
    class="flex border-b w-full select-none px-4 py-2"
  >
    <div class="flex gap-2">
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
  <div class="h-[225px] p-4 flex flex-col gap-2">
    {#if config}
      <div>
        {config.autoOpen}
      </div>
    {/if}
    <div class="flex items-center space-x-2">
      <Switch
        checked={config?.autoOpen}
        id="auto-open"
        onCheckedChange={(v) => {
          if (!config) return;
          config.autoOpen = v;
          updateConfig(config);
        }}
      />
      <Label for="auto-open">Auto Open OP.GG Multi</Label>
    </div>
    <div class="flex items-center space-x-2">
      <Switch
        checked={config?.autoAccept}
        id="auto-accept"
        onCheckedChange={(v) => {
          if (!config) return;
          config.autoAccept = v;
          updateConfig(config);
        }}
      />
      <Label for="auto-accept">Auto Accept</Label>
    </div>
    <div>
      <div class="text-sm">
        Client State: <span class="text-blue-500">{state}</span>
      </div>
    </div>
    {#if champSelect && state === "ChampSelect"}
      <div></div>
    {/if}
  </div>
  <div class="px-4 items-center flex pt-1 border-t">
    {#if connected}
      <div class="flex items-center gap-1 text-sm">
        <div class="bg-green-500 animate-pulse h-2 w-2 rounded-full" />
        <div>Connected to League Client</div>
      </div>
    {:else}
      <div class="flex items-center gap-1 text-sm">
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

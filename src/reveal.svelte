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
  import Button from "./components/button/button.svelte";
  import { fade } from "svelte/transition";

  let state = "Unknown";
  let connected = false;
  let champSelect: ChampSelect | null = null;
  let config: Config | null = null;
  let lcu_info;

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
  <div
    data-tauri-drag-region
    class="flex border-b rounded-t-lg w-full select-none px-4 py-2"
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
    <div class="flex gap-5">
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
        <Label for="auto-open">Auto Open OP.GG</Label>
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
    </div>
    <div>
      <div class="text-sm">
        Client State: <span class="text-blue-500">{state}</span>
      </div>
    </div>
    {#if state === "ChampSelect"}
      <div in:fade class="flex gap-5 w-full">
        <div class="grid grid-cols-2 gap-2 text-sm">
          {#if champSelect}
            {#each champSelect.participants as participant}
              <div class="flex flex-col justify-center items-center text-xs">
                <div>{participant.game_name}#{participant.game_tag}</div>
                <div class="text-blue-500">({participant.name})</div>
              </div>
            {/each}
          {:else}
            <div in:fade class="flex gap-2 items-center animate-pulse">
              Grabbing Champ Select Data...
            </div>
          {/if}
        </div>
        <div class="flex w-[140px] ml-auto flex-col gap-2">
          <Button
            size="sm"
            on:click={() =>
              invoke("open_opgg_link", {
                summoners: champSelect?.participants,
              })}>Open OP.GG</Button
          >
          <Button
            variant="destructive"
            size="sm"
            on:click={() => {
              invoke("dodge");
            }}>Dodge</Button
          >
          <!--
          <Button
            variant="destructive"
            size="sm"
            on:click={() => {
              invoke("enable_dodge");
            }}>Dodge Last Second</Button
          -->
        </div>
      </div>
    {:else}
      <div in:fade class="flex gap-2 items-center animate-pulse">
        Waiting for Lobby...
      </div>
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

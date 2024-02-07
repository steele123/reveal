<script lang="ts">
  import Switch from "../components/switch.svelte";
  import Label from "../components/label.svelte";
  import { invoke } from "@tauri-apps/api/tauri";
  import { updateConfig, type Config } from "../lib/config";
  import type { ChampSelect } from "../lib/champ_select";
  import Button from "../components/button/button.svelte";
  import { fade } from "svelte/transition";
  import RevealCount from "./reveal-count.svelte";

  export let config: Config | null = null;
  export let state = "Unknown";
  export let champSelect: ChampSelect | null = null;
  export let connected = false;
</script>

<div class="flex flex-col gap-2">
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
  <div class="grid grid-cols-2">
    <div class="flex gap-2">
      <div class="text-muted-foreground">State:</div>
      <div>{state}</div>
    </div>
    <div class="flex gap-2">
      <div class="text-muted-foreground">Reveals:</div>
      <div>
        <RevealCount />
      </div>
    </div>
  </div>
  {#if state === "ChampSelect"}
    <div in:fade class="flex gap-5 w-full">
      <div class="grid grid-cols-2 gap-2 text-sm">
        {#if champSelect}
          {#each champSelect.participants as participant}
            <div class="flex flex-col justify-center items-center text-xs">
              <div class="line-clamp-1">
                {participant.game_name}#{participant.game_tag}
              </div>
              <div class="text-blue-500">
                ({participant.name})
              </div>
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
  {:else if state === "InProgress"}
    <div in:fade class="flex gap-2 items-center animate-pulse">In Game</div>
  {:else if !connected}
    <div
      class="text-xs p-2 rounded mt-10 bg-accent border flex gap-2 text-muted-foreground"
    >
      <svg
        xmlns="http://www.w3.org/2000/svg"
        width="20"
        height="20"
        viewBox="0 0 24 24"
        fill="none"
        stroke="currentColor"
        stroke-width="2"
        stroke-linecap="round"
        stroke-linejoin="round"
        class="lucide lucide-info"
        ><circle cx="12" cy="12" r="10" /><path d="M12 16v-4" /><path
          d="M12 8h.01"
        /></svg
      >
      Issues Connecting? <br /> Try restarting the League Client and running as Reveal
      as Administrator.
    </div>
    <div in:fade class="flex gap-2 items-center animate-pulse">
      Waiting for Lobby...
    </div>
  {:else}
    <div in:fade class="flex gap-2 items-center animate-pulse">
      Waiting for Lobby...
    </div>
  {/if}
</div>

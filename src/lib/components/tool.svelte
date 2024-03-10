<script lang="ts">
  import { invoke } from "@tauri-apps/api/tauri";
  import { updateConfig, type Config } from "$lib/config";
  import { fade } from "svelte/transition";
  import RevealCount from "./reveal-count.svelte";
  import type { ChampSelect } from "$lib/champ_select";
  import { Switch } from "./ui/switch";
  import { Label } from "./ui/label";
  import { Button } from "./ui/button";
  import * as Select from "$lib/components/ui/select";

  export let config: Config | null = null;
  export let state = "Unknown";
  export let champSelect: ChampSelect | null = null;
  export let connected = false;

  let lastSecondDodgeEnabled = false;
  $: if (state !== "ChampSelect" && lastSecondDodgeEnabled) {
    // lobby is prob dodged or started, can reset state now
    lastSecondDodgeEnabled = false;
  }

  const multiProviders = [
    {
      label: "OP.GG",
      value: "opgg"
    },
    {
      label: "DeepLoL",
      value: "deeplol"
    },
    {
      label: "U.GG",
      value: "ugg"
    },
    {
      label: "Poro.gg",
      value: "poro"
    },
    {
      label: "Tracker.gg",
      value: "tracker"
    }
  ];
</script>

<div class="flex flex-col gap-2">
  <div class="flex gap-5 items-center">
    <div>
      <Label for="favoriteFruit">Multi Website</Label>
      <Select.Root
        onSelectedChange={(v) => {
          if (!config) return;
          if (v) {
            config.multiProvider = v.value;
          }
          updateConfig(config);
        }}
        selected={multiProviders.find((p) => p.value === config?.multiProvider)}
      >
        <Select.Trigger class="w-[180px]">
          <Select.Value />
        </Select.Trigger>
        <Select.Content>
          <Select.Group>
            {#each multiProviders as multi}
              <Select.Item value={multi.value} label={multi.label}
                >{multi.label}</Select.Item
              >
            {/each}
          </Select.Group>
        </Select.Content>
      </Select.Root>
    </div>
    <div class="flex flex-col gap-2">
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
        <Label for="auto-open">Auto Open Link</Label>
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
      {#if champSelect}
        <div class="grid grid-cols-2 gap-2 text-sm">
          {#each champSelect.participants as participant}
            <div class="flex flex-col justify-center items-center text-xs">
              <div class="line-clamp-1">
                {participant.game_name}#{participant.game_tag}
              </div>
              {#if participant.name}
                <div class="text-blue-500">
                  ({participant.name})
                </div>
              {/if}
            </div>
          {/each}
        </div>
      {:else}
        <div in:fade class="flex mb-auto gap-2 items-center animate-pulse">
          Grabbing Players...
        </div>
      {/if}
      <div class="flex w-[120px] ml-auto flex-col gap-2">
        <Button size="sm" on:click={() => invoke("open_opgg_link")}
          >Open OP.GG</Button
        >
        <Button
          variant="destructive"
          size="sm"
          on:click={() => {
            invoke("dodge");
          }}>Dodge</Button
        >
        <!--
        <div class="flex py-2 items-center gap-2">
          <Switch
            checked={lastSecondDodgeEnabled}
            id="last-second-dodge"
            onCheckedChange={async (v) => {
              await invoke("enable_dodge");
              lastSecondDodgeEnabled = v;
            }}
          />
          <Label class="text-center text-xs" for="last-second-dodge"
            >Last Second Dodge</Label
          >
        </div>
      -->
      </div>
    </div>
  {:else if state === "InProgress"}
    <div in:fade class="flex gap-2 items-center animate-pulse">In Game</div>
  {:else if !connected}
    <div in:fade class="flex gap-2 items-center animate-pulse">
      Trying to find League Client...
    </div>
    <div
      class="text-xs p-2 rounded bg-accent border flex gap-2 text-muted-foreground"
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
      Issues Connecting? <br /> Try restarting the League Client and running Reveal
      as Administrator.
    </div>
  {:else}
    <div in:fade class="flex gap-2 items-center animate-pulse">
      Waiting for Champ Select...
    </div>
  {/if}
</div>

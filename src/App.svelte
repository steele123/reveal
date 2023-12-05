<script lang="ts">
  import { getVersion, hide } from "@tauri-apps/api/app";
  import { exit } from "@tauri-apps/api/process";
  import Switch from "./components/switch.svelte";
  import Label from "./components/label.svelte";
  import { onMount } from "svelte";
  import { emit, listen } from '@tauri-apps/api/event'

  let state = "Unknown"

  onMount(async () => {
    const unlisten = await listen("client_state_update", (event) => {
      state = event.payload
    })

    return () => {
      unlisten()
    }
  })
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
      <button class="" on:click={() => hide}>-</button>
      <button class="text-xs" on:click={() => exit}>X</button>
    </div>
  </div>
  <div class="h-[225px] p-4 flex flex-col gap-2">
    <div class="flex items-center space-x-2">
      <Switch id="airplane-mode" />
      <Label for="airplane-mode">Auto Open OP.GG Multi</Label>
    </div>
    <div class="flex items-center space-x-2">
      <Switch id="airplane-mode" />
      <Label for="airplane-mode">Auto Accept</Label>
    </div>
    <div>
      <div class="text-sm">Client State: <span class="text-blue-500">{state}</span></div>

    </div>
  </div>
  <div class="px-4 items-center flex pt-1 border-t">
    <div class="flex items-center gap-1 text-sm">
      <div class="bg-green-500 animate-pulse h-2 w-2 rounded-full" />
      <div>Connected to League Client</div>
    </div>
    <a
      href="https://hyeb.gg/reveal"
      target="_blank"
      class="text-blue-500 ml-auto hover:underline text-xs">hyeb.gg/reveal</a
    >
  </div>
</main>

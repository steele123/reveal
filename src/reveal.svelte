<script lang="ts">
  import { getVersion } from "@tauri-apps/api/app";
  import { appWindow } from "@tauri-apps/api/window";
  import { onMount } from "svelte";
  import { listen } from "@tauri-apps/api/event";
  import { invoke } from "@tauri-apps/api/tauri";
  import { updateConfig, type Config } from "./lib/config";
  import type { ChampSelect } from "./lib/champ_select";
  import { fade } from "svelte/transition";
  import Updater from "./components/updater.svelte";
  import Tool from "./components/tool.svelte";
  import Navbar from "./components/navbar.svelte";
  import Footer from "./components/footer.svelte";
  import { checkUpdate, installUpdate } from "@tauri-apps/api/updater";
  import { relaunch } from "@tauri-apps/api/process";

  let state = "Unknown";
  let connected = false;
  let champSelect: ChampSelect | null = null;
  let config: Config | null = null;
  let updateStatus:
    | "Checking"
    | "Downloading"
    | "Installing"
    | "Restarting"
    | "UpToDate" = "Checking";

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

    let update = await checkUpdate();
    if (update.shouldUpdate) {
      updateStatus = "Downloading";
      await installUpdate();
      updateStatus = "Installing";
      await relaunch();
    } else {
      updateStatus = "UpToDate";
    }
  });
</script>

<main class="h-[300px] bg-background border rounded-md">
  <Navbar />
  <div class="h-[225px] p-2">
    {#if updateStatus === "Checking"}
      <div>Checking for updates...</div>
    {:else if updateStatus === "Downloading"}
      <div>Downloading update...</div>
    {:else if updateStatus === "Installing"}
      <div>Installing update...</div>
    {:else if updateStatus === "Restarting"}
      <div>Restarting...</div>
    {:else if updateStatus === "UpToDate"}
      <Tool {config} {state} {champSelect} />
    {/if}
  </div>
  <Footer {connected} />
</main>

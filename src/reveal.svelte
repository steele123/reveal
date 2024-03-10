<script lang="ts">
  import { onMount } from "svelte";
  import { listen } from "@tauri-apps/api/event";
  import { invoke } from "@tauri-apps/api/tauri";
  import { type Config } from "$lib/config";
  import "@fontsource-variable/inter";
  import type { ChampSelect } from "$lib/champ_select";
  import Tool from "$lib/components/tool.svelte";
  import Navbar from "$lib/components/navbar.svelte";
  import Footer from "$lib/components/footer.svelte";
  import { checkUpdate, installUpdate } from "@tauri-apps/api/updater";
  import { relaunch } from "@tauri-apps/api/process";

  let state = "Unknown";
  let connected = false;
  let champSelect: ChampSelect | null = null;
  let config: Config | null = null;
  let updateStatus: "Checking" | "Installing" | "Restarting" | "UpToDate" =
    "Checking";

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
      updateStatus = "Installing";
      setTimeout(async () => {
        try {
          await installUpdate();
        } catch (error) {
          console.error(error);
          updateStatus = "UpToDate";
          return;
        }

        updateStatus = "Restarting";
        await relaunch();
      }, 5000);
    } else {
      updateStatus = "UpToDate";
    }
  });
</script>

<main class="h-[325px] bg-background border rounded-md">
  <Navbar />
  <div class="h-[215px] px-4 pt-1">
    {#if updateStatus === "Checking"}
      <div>Checking for updates...</div>
    {:else if updateStatus === "Installing"}
      <div>Found update, installing latest update...</div>
    {:else if updateStatus === "Restarting"}
      <div>Restarting...</div>
    {:else if updateStatus === "UpToDate"}
      <Tool {config} {state} {champSelect} {connected} />
    {/if}
  </div>
  <Footer {connected} />
</main>

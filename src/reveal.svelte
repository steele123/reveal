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
  import Navbar from "./components/navbar.svelte";
  import Footer from "./components/footer.svelte";

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
  <Navbar />
  <Tool {config} {state} {champSelect} />
  <Footer {connected} />
</main>

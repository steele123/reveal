<script lang="ts">
  import { onMount } from "svelte";
  import { listen, type UnlistenFn } from "@tauri-apps/api/event";
  import { invoke } from "@tauri-apps/api/tauri";
  import { type Config } from "$lib/config";
  import "@fontsource-variable/inter";
  import type { ChampSelect } from "$lib/champ_select";
  import Tool from "$lib/components/tool.svelte";
  import Navbar from "$lib/components/navbar.svelte";
  import Footer from "$lib/components/footer.svelte";
  import { runUpdater, type UpdateStatus } from "$lib/updater";

  let state = "Unknown";
  let connected = false;
  let champSelect: ChampSelect | null = null;
  let config: Config | null = null;
  let updateStatus: UpdateStatus = "Checking";

  onMount(() => {
    let disposed = false;
    let unlisten: UnlistenFn[] = [];

    async function initialize() {
      const listeners: UnlistenFn[] = [];
      try {
        listeners.push(
          await listen<string>(
            "client_state_update",
            ({ payload: newState }) => {
              if (newState === "ChampSelect") champSelect = null;
              state = newState;
            },
          ),
        );
        listeners.push(
          await listen<boolean>("lcu_state_update", ({ payload }) => {
            connected = payload;
          }),
        );
        listeners.push(
          await listen<ChampSelect>("champ_select_started", ({ payload }) => {
            champSelect = payload;
          }),
        );

        if (disposed) {
          listeners.forEach((stopListening) => stopListening());
          return;
        }

        const loadedConfig = await invoke<Config>("app_ready");
        if (disposed) {
          listeners.forEach((stopListening) => stopListening());
          return;
        }

        unlisten = listeners;
        config = loadedConfig;
      } catch (error) {
        listeners.forEach((stopListening) => stopListening());
        console.error("Failed to initialize Reveal", error);
      }

      if (!disposed) {
        await runUpdater((status) => {
          if (!disposed) updateStatus = status;
        });
      }
    }

    void initialize();

    return () => {
      disposed = true;
      unlisten.forEach((stopListening) => stopListening());
    };
  });
</script>

<main class="h-[325px] bg-background border rounded-md">
  <Navbar />
  <div class="h-[240px] px-4 pt-1">
    {#if updateStatus === "Checking"}
      <div>Checking for updates...</div>
    {:else if updateStatus === "Installing"}
      <div>Found update, installing latest update...</div>
    {:else if updateStatus === "Restarting"}
      <div>Restarting...</div>
    {:else if updateStatus === "UpToDate"}
      <Tool
        {config}
        {state}
        {champSelect}
        {connected}
        onConfigChange={(nextConfig) => (config = nextConfig)}
      />
    {/if}
  </div>
  <Footer {connected} />
</main>

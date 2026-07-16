<script lang="ts">
  import { onMount } from "svelte";
  import { listen, type UnlistenFn } from "@tauri-apps/api/event";
  import { invoke } from "@tauri-apps/api/tauri";
  import { DEFAULT_CONFIG, type Config } from "$lib/config";
  import "@fontsource-variable/inter";
  import type { ChampSelect } from "$lib/champ_select";
  import Tool from "$lib/components/tool.svelte";
  import Navbar from "$lib/components/navbar.svelte";
  import Footer from "$lib/components/footer.svelte";
  import { runUpdater, type UpdateStatus } from "$lib/updater";
  import { isTauriRuntime } from "$lib/runtime";
  import { getPreviewState } from "$lib/preview";

  let state = "Unknown";
  let connected = false;
  let champSelect: ChampSelect | null = null;
  let config: Config | null = null;
  let updateStatus: UpdateStatus = "Checking";

  function setConfig(nextConfig: Config) {
    config = nextConfig;
  }

  $: updateMessage = {
    Checking: "Checking for updates",
    Installing: "Installing the latest version",
    Restarting: "Restarting Reveal",
    UpToDate: "Ready",
  }[updateStatus];

  onMount(() => {
    if (!isTauriRuntime()) {
      const preview = getPreviewState();
      config = { ...DEFAULT_CONFIG };
      connected = preview.connected;
      state = preview.state;
      champSelect = preview.champSelect;
      updateStatus = "UpToDate";
      return;
    }

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

<main
  class="reveal-shell flex h-screen w-screen flex-col overflow-hidden rounded-xl border border-white/10 shadow-2xl"
>
  <Navbar />
  <div class="min-h-0 flex-1 px-5 py-4">
    {#if updateStatus === "UpToDate"}
      <Tool
        {config}
        {state}
        {champSelect}
        {connected}
        onConfigChange={setConfig}
      />
    {:else}
      <div class="flex h-full items-center justify-center">
        <div class="flex flex-col items-center gap-4 text-center">
          <div
            class="grid h-12 w-12 place-items-center rounded-2xl border border-blue-400/20 bg-blue-500/10 shadow-[0_0_30px_rgba(59,130,246,0.15)]"
          >
            <img alt="" src="/icon.png" class="h-7 w-7" />
          </div>
          <div>
            <div class="text-sm font-medium">{updateMessage}</div>
            <div class="mt-1 text-xs text-muted-foreground">
              This should only take a moment.
            </div>
          </div>
          <div class="h-1 w-28 overflow-hidden rounded-full bg-muted">
            <div class="h-full w-1/2 animate-pulse rounded-full bg-blue-500" />
          </div>
        </div>
      </div>
    {/if}
  </div>
  <Footer {connected} />
</main>

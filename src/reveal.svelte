<script lang="ts">
  import { onMount } from "svelte";
  import { listen, type UnlistenFn } from "@tauri-apps/api/event";
  import { invoke } from "@tauri-apps/api/tauri";
  import { DEFAULT_CONFIG, updateConfig, type Config } from "$lib/config";
  import "@fontsource-variable/inter";
  import type { ChampSelect } from "$lib/champ_select";
  import { setPreviewRoleHandler, type MatchupBoard } from "$lib/matchups";
  import Tool from "$lib/components/tool.svelte";
  import Navbar from "$lib/components/navbar.svelte";
  import Footer from "$lib/components/footer.svelte";
  import HistoryPanel from "$lib/components/history-panel.svelte";
  import AutomationPanel from "$lib/components/automation-panel.svelte";
  import LegacyInstallBanner from "$lib/components/legacy-install-banner.svelte";
  import DiscordPrompt from "$lib/components/discord-prompt.svelte";
  import { DISCORD_URL, openExternalLink } from "$lib/link_actions";
  import type { LiveGame } from "$lib/live_game";
  import { runUpdater, type UpdateStatus } from "$lib/updater";
  import { isTauriRuntime } from "$lib/runtime";
  import { getPreviewState, previewForProvider, previewWithRole } from "$lib/preview";
  import {
    clearLobbyHistory,
    createLobbyHistoryId,
    loadLobbyHistory,
    lobbyEvent,
    markLobbyDodged,
    markLobbyGameStarted,
    recordLobbyReveal,
    seenBeforeIndex,
    type LobbyHistoryEntry,
  } from "$lib/lobby_history";
  import { lobbyNotice, lobbyPlayers, notifyLobbyRevealed } from "$lib/notifications";
  import { createMultiSearchLink } from "$lib/link_actions";
  import { logFrontendError, logFrontendInfo } from "$lib/logging";

  let state = "Unknown";
  let connected = false;
  let queuePaused = false;
  /** Reveal's countdown to queueing again, after a game or a queue that ran long. */
  let requeue: { deadline: number; after: "game" | "queue" | "dodge" } | null = null;
  let champSelect: ChampSelect | null = null;
  let matchups: MatchupBoard | null = null;
  let config: Config | null = null;
  let updateStatus: UpdateStatus = "Checking";
  let activePage: "reveal" | "history" | "auto" = "reveal";
  /** The browser preview's sample board, shown as the chosen provider would. */
  let previewMatchups: MatchupBoard | null = null;
  $: if (previewMatchups) {
    matchups = previewForProvider(previewMatchups, config?.countersProvider ?? "lolalytics");
  }
  let history: LobbyHistoryEntry[] = [];
  let activeLobbyId: string | null = null;
  /** The lobby the "team revealed" alert went out for, so each gets one. */
  let notifiedLobbyId: string | null = null;
  let lobbyNoticeTimer: ReturnType<typeof setTimeout> | undefined;
  /** Gives off-role checks, which read match history, time to finish before the alert. */
  const LOBBY_NOTICE_DELAY_MS = 5_000;

  /** Once all five teammates are in, sends the "team revealed" alert if it's turned on. */
  function scheduleLobbyNotice() {
    const lobbyId = activeLobbyId;
    const team = champSelect?.participants.filter((participant) => !participant.is_enemy) ?? [];
    const notifications = config?.notifications;
    const wanted = notifications
      ? [notifications.desktop, notifications.phone, notifications.discord].some((alerts) =>
          alerts.includes("lobbyRevealed"),
        )
      : false;
    if (!lobbyId || !wanted || team.length < 5 || notifiedLobbyId === lobbyId) return;
    notifiedLobbyId = lobbyId;
    lobbyNoticeTimer = setTimeout(() => {
      if (activeLobbyId !== lobbyId || !champSelect) return;
      const participants = champSelect.participants;
      const seen = seenBeforeIndex(history, lobbyId);
      const team = participants.filter((participant) => !participant.is_enemy);
      const link = config ? createMultiSearchLink(team, config.multiProvider) : null;
      notifyLobbyRevealed(
        lobbyNotice(participants, seen),
        lobbyPlayers(participants, seen),
        link,
      ).catch((error) =>
        logFrontendError("Failed to send the team revealed alert", error),
      );
    }, LOBBY_NOTICE_DELAY_MS);
  }
  let legacyInstall: { version: string } | null = null;
  let liveGame: LiveGame | null = null;
  let queueName: string | null = null;

  function setConfig(nextConfig: Config) {
    config = nextConfig;
  }

  /** Applies a settings change right away and saves it, undoing it if saving fails. */
  async function changeConfig(nextConfig: Config) {
    const previousConfig = config;
    config = nextConfig;
    if (!isTauriRuntime()) return;
    try {
      await updateConfig(nextConfig);
    } catch (error) {
      logFrontendError("Failed to save config", error);
      config = previousConfig;
    }
  }

  // Shown once, never over Champ Select or the update screen.
  $: showDiscordPrompt =
    config !== null &&
    !config.discordPromptSeen &&
    updateStatus === "UpToDate" &&
    state !== "ChampSelect";

  async function closeDiscordPrompt(joined: boolean) {
    if (!config) return;
    if (joined) {
      openExternalLink(DISCORD_URL).catch((error) =>
        logFrontendError("Failed to open Discord invite", error),
      );
    }

    const nextConfig = { ...config, discordPromptSeen: true };
    config = nextConfig;
    logFrontendInfo(`Discord prompt answered; joined=${joined}`);
    if (!isTauriRuntime()) return;
    try {
      await updateConfig(nextConfig);
    } catch (error) {
      logFrontendError("Failed to save Discord prompt state", error);
    }
  }

  function clearHistory() {
    clearLobbyHistory();
    history = [];
  }

  function navigate(page: "reveal" | "history" | "auto") {
    activePage = page;
  }

  $: updateMessage = {
    Checking: "Checking for updates",
    Installing: "Installing the latest version",
    Restarting: "Restarting Reveal",
    UpToDate: "Ready",
  }[updateStatus];

  // Grow the whole window's content with the window, from the size it opens at up to twice that.
  const BASE_SIZE = 680;
  const MAX_SCALE = 2;
  let scale = 1;
  function fitScale() {
    const fit = Math.min(window.innerWidth, window.innerHeight) / BASE_SIZE;
    scale = Math.min(MAX_SCALE, Math.max(1, fit));
  }

  onMount(() => {
    fitScale();
    window.addEventListener("resize", fitScale);
    const stopScaling = () => window.removeEventListener("resize", fitScale);
    history = loadLobbyHistory();

    if (!isTauriRuntime()) {
      const preview = getPreviewState();
      config = {
        ...DEFAULT_CONFIG,
        discordPromptSeen: !new URLSearchParams(location.search).has("discord"),
      };
      connected = preview.connected;
      state = preview.state;
      liveGame = preview.liveGame ?? null;
      queueName = preview.champSelect ? "Ranked Solo/Duo" : null;
      champSelect = preview.champSelect;
      previewMatchups = preview.matchups ?? null;
      setPreviewRoleHandler((championId, role) => {
        if (previewMatchups) previewMatchups = previewWithRole(previewMatchups, championId, role);
      });
      activePage = preview.activePage;
      if (preview.history.length > 0) history = preview.history;
      if (new URLSearchParams(location.search).has("legacy")) {
        legacyInstall = { version: "1.4.3" };
      }
      updateStatus = "UpToDate";
      return stopScaling;
    }

    let disposed = false;
    let unlisten: UnlistenFn[] = [];

    async function initialize() {
      logFrontendInfo("Frontend initialization started");
      const listeners: UnlistenFn[] = [];
      try {
        listeners.push(
          await listen<string>(
            "client_state_update",
            ({ payload: newState }) => {
              const event = lobbyEvent(state, newState);
              if (event === "new-lobby") {
                activeLobbyId = createLobbyHistoryId();
                champSelect = null;
                matchups = null;
              } else if (event === "game-started") {
                if (activeLobbyId) history = markLobbyGameStarted(activeLobbyId);
                activeLobbyId = null;
              } else if (event === "left") {
                if (activeLobbyId) history = markLobbyDodged(activeLobbyId);
                activeLobbyId = null;
              }
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
          await listen<string | null>("queue_name", ({ payload }) => {
            queueName = payload;
          }),
        );
        listeners.push(
          await listen<LiveGame | null>("live_game_update", ({ payload }) => {
            liveGame = payload;
          }),
        );
        listeners.push(
          await listen<typeof requeue>("requeue_countdown", ({ payload }) => {
            requeue = payload;
          }),
        );
        listeners.push(
          await listen<boolean>("queue_pause_update", ({ payload }) => {
            queuePaused = payload;
          }),
        );
        listeners.push(
          await listen<MatchupBoard | null>("champ_select_matchups", ({ payload }) => {
            matchups = payload;
          }),
        );
        listeners.push(
          await listen<ChampSelect>("champ_select_started", ({ payload }) => {
            champSelect = payload;
            activeLobbyId ??= createLobbyHistoryId();
            // History is about your own team.
            history = recordLobbyReveal(activeLobbyId, {
              ...payload,
              participants: payload.participants.filter((participant) => !participant.is_enemy),
            });
            scheduleLobbyNotice();
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
        logFrontendInfo("Frontend initialization completed");

        invoke<{ version: string } | null>("get_legacy_install")
          .then((install) => {
            if (!disposed) legacyInstall = install;
          })
          .catch((error) =>
            logFrontendError("Failed to check for an old MSI install", error),
          );
      } catch (error) {
        listeners.forEach((stopListening) => stopListening());
        logFrontendError("Failed to initialize Reveal", error);
      }

      if (!disposed) {
        await runUpdater((status) => {
          if (!disposed) updateStatus = status;
        });
      }
    }

    void initialize();

    return () => {
      stopScaling();
      clearTimeout(lobbyNoticeTimer);
      disposed = true;
      unlisten.forEach((stopListening) => stopListening());
    };
  });
</script>

<main
  class="reveal-shell relative flex flex-col overflow-hidden rounded-xl border border-white/10 shadow-2xl"
  style="zoom: {scale}; width: calc(100vw / {scale}); height: calc(100vh / {scale});"
>
  <Navbar
    {activePage}
    historyCount={history.length}
    onNavigate={navigate}
  />
  <!-- Hidden in Champ Select, where the fixed-size window has no room to spare. -->
  {#if legacyInstall && state !== "ChampSelect"}
    <LegacyInstallBanner
      version={legacyInstall.version}
      onRemoved={() => (legacyInstall = null)}
    />
  {/if}
  <div class="min-h-0 flex-1 px-5 py-3">
    {#if updateStatus === "UpToDate"}
      {#if activePage === "auto"}
        <AutomationPanel {config} onChange={changeConfig} />
      {:else if activePage === "history"}
        <HistoryPanel
          {history}
          provider={config?.multiProvider ?? "opgg"}
          onClear={clearHistory}
        />
      {:else}
        <Tool
          {config}
          {state}
          {champSelect}
          {matchups}
          {connected}
          {queuePaused}
          {history}
          {activeLobbyId}
          {liveGame}
          {queueName}
          {requeue}
          onConfigChange={setConfig}
          onOpenPage={navigate}
        />
      {/if}
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
            <div class="h-full w-1/2 animate-pulse rounded-full bg-blue-500"></div>
          </div>
        </div>
      </div>
    {/if}
  </div>
  <Footer {connected} {state} {queuePaused} inGame={liveGame !== null} />
  {#if showDiscordPrompt}
    <DiscordPrompt onClose={closeDiscordPrompt} />
  {/if}
</main>

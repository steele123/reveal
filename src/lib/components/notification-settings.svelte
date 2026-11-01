<script lang="ts">
  import type { Config, NotificationSettings } from "$lib/config";
  import { copyLink, openExternalLink } from "$lib/link_actions";
  import { logFrontendError } from "$lib/logging";
  import {
    generateNtfyTopic,
    isDiscordWebhook,
    isNtfyTopic,
    NOTIFICATION_ALERTS,
    ntfyTopicLink,
    sendTestNotification,
    type NotificationChannel,
  } from "$lib/notifications";
  import { isTauriRuntime } from "$lib/runtime";
  import InfoTip from "./info-tip.svelte";

  export let config: Config | null = null;
  export let onChange: (config: Config) => void = () => {};

  const CHANNELS: ReadonlyArray<{ key: NotificationChannel; label: string }> = [
    { key: "desktop", label: "This PC" },
    { key: "phone", label: "Phone" },
    { key: "discord", label: "Discord" },
  ];
  const TIP =
    "This PC shows Windows notifications. Phone sends to the free ntfy app (ntfy.sh) on a topic only you know. Discord posts to a channel through a webhook. Only Team revealed names players: on the phone and in Discord it lists your team's Riot IDs, ranks and roles, with a link to their multi-search.";

  const BUTTON =
    "h-7 shrink-0 rounded-md border border-white/10 bg-white/[0.04] px-2 text-[10px] font-medium text-muted-foreground transition hover:border-blue-400/30 hover:bg-blue-500/10 hover:text-foreground disabled:opacity-50";

  /** What the last "Send test" said, per channel. */
  let status: Partial<Record<NotificationChannel, { ok: boolean; text: string }>> = {};
  let showWebhook = false;

  $: settings = config?.notifications ?? null;
  $: phoneReady = Boolean(settings && isNtfyTopic(settings.ntfyTopic));
  $: discordReady = Boolean(settings && isDiscordWebhook(settings.discordWebhook));

  function update(patch: Partial<NotificationSettings>) {
    if (!config) return;
    onChange({ ...config, notifications: { ...config.notifications, ...patch } });
  }

  function toggle(channel: NotificationChannel, alert: string, on: boolean) {
    if (!settings) return;
    const alerts = settings[channel].filter((key) => key !== alert);
    update({ [channel]: on ? [...alerts, alert] : alerts });
  }

  function ready(channel: NotificationChannel): boolean {
    return channel === "desktop" || (channel === "phone" ? phoneReady : discordReady);
  }

  async function test(channel: NotificationChannel) {
    if (!settings) return;
    if (!isTauriRuntime()) {
      status = { ...status, [channel]: { ok: false, text: "Only in the app" } };
      return;
    }
    status = { ...status, [channel]: { ok: true, text: "Sending…" } };
    try {
      await sendTestNotification(channel, settings);
      status = { ...status, [channel]: { ok: true, text: "Sent" } };
    } catch (error) {
      logFrontendError(`Test notification to ${channel} failed`, error);
      status = { ...status, [channel]: { ok: false, text: String(error) } };
    }
  }

  function newTopic() {
    update({ ntfyTopic: generateNtfyTopic() });
    status = { ...status, phone: undefined };
  }

  function open(link: string) {
    openExternalLink(link).catch((error) => logFrontendError("Failed to open link", error));
  }

  function copyTopicLink() {
    if (!settings) return;
    copyLink(ntfyTopicLink(settings.ntfyTopic))
      .then(() => (status = { ...status, phone: { ok: true, text: "Link copied" } }))
      .catch((error) => logFrontendError("Failed to copy the ntfy link", error));
  }
</script>

<section class="reveal-panel shrink-0 p-1">
  <div class="px-3 pb-1.5 pt-2 text-[11px] font-medium uppercase tracking-[0.12em] text-muted-foreground">
    Notifications<InfoTip text={TIP} />
  </div>

  <div class="px-3 pb-2">
    <div
      class="grid grid-cols-[minmax(0,1fr)_56px_56px_56px] items-center gap-x-1 border-b border-white/10 pb-1 text-[9px] font-medium uppercase tracking-[0.08em] text-muted-foreground"
    >
      <span>Alert</span>
      {#each CHANNELS as channel (channel.key)}
        <span class="text-center" class:opacity-50={!ready(channel.key)}>{channel.label}</span>
      {/each}
    </div>
    {#each NOTIFICATION_ALERTS as alert (alert.key)}
      <div
        class="grid grid-cols-[minmax(0,1fr)_56px_56px_56px] items-center gap-x-1 border-b border-white/[0.05] py-1.5 last:border-b-0"
      >
        <div class="min-w-0">
          <div class="text-xs font-medium">{alert.label}</div>
          <div class="truncate text-[10px] text-muted-foreground" title={alert.description}>
            {alert.description}
          </div>
        </div>
        {#each CHANNELS as channel (channel.key)}
          <label class="grid cursor-pointer place-items-center" class:opacity-50={!ready(channel.key)}>
            <input
              type="checkbox"
              class="h-3.5 w-3.5 cursor-pointer accent-blue-500"
              aria-label={`${alert.label} on ${channel.label}`}
              disabled={!settings}
              checked={settings?.[channel.key].includes(alert.key) ?? false}
              on:change={(event) => toggle(channel.key, alert.key, event.currentTarget.checked)}
            />
          </label>
        {/each}
      </div>
    {/each}
  </div>

  <div class="divide-y divide-white/10 border-t border-white/10">
    <div class="flex items-center gap-2 px-3 py-2">
      <div class="min-w-0 flex-1">
        <div class="text-xs font-medium">This PC</div>
        <div class="text-[10px] text-muted-foreground">Windows notifications, quiet in Focus mode</div>
      </div>
      {#if status.desktop}
        <span class="text-[10px] {status.desktop.ok ? 'text-emerald-300' : 'text-red-300'}">{status.desktop.text}</span>
      {/if}
      <button type="button" class={BUTTON} disabled={!settings} on:click={() => test("desktop")}>
        Send test
      </button>
    </div>

    <div class="px-3 py-2">
      <div class="flex items-center gap-2">
        <div class="min-w-0 flex-1">
          <div class="text-xs font-medium">Phone</div>
          <div class="text-[10px] text-muted-foreground">
            Install the free
            <button type="button" class="text-blue-300 hover:underline" on:click={() => open("https://ntfy.sh/app")}>ntfy app</button>
            and subscribe to this topic. Anyone who knows it can read your alerts.
          </div>
        </div>
      </div>
      <div class="mt-1.5 flex items-center gap-1.5">
        <input
          class="h-7 min-w-0 flex-1 rounded-md border bg-white/[0.035] px-2 font-mono text-[11px] placeholder:font-sans placeholder:text-muted-foreground focus:outline-none {settings?.ntfyTopic && !phoneReady
            ? 'border-red-400/50'
            : 'border-white/10 focus:border-blue-400/40'}"
          placeholder="No topic yet"
          aria-label="ntfy topic"
          autocomplete="off"
          spellcheck="false"
          disabled={!settings}
          value={settings?.ntfyTopic ?? ""}
          on:change={(event) => update({ ntfyTopic: event.currentTarget.value.trim() })}
        />
        <button type="button" class={BUTTON} disabled={!settings} on:click={newTopic}>
          {settings?.ntfyTopic ? "New topic" : "Create topic"}
        </button>
        {#if phoneReady}
          <button type="button" class={BUTTON} title="Copy the subscribe link to open on your phone" on:click={copyTopicLink}>
            Copy link
          </button>
          <button type="button" class={BUTTON} on:click={() => test("phone")}>Send test</button>
        {/if}
      </div>
      {#if status.phone}
        <div class="mt-1 text-[10px] {status.phone.ok ? 'text-emerald-300' : 'text-red-300'}">{status.phone.text}</div>
      {:else if settings?.ntfyTopic && !phoneReady}
        <div class="mt-1 text-[10px] text-red-300">Use only letters, digits, - and _ (up to 64).</div>
      {/if}
    </div>

    <div class="px-3 py-2">
      <div class="text-xs font-medium">Discord</div>
      <div class="text-[10px] text-muted-foreground">
        In a channel's settings, open Integrations › Webhooks, create one and paste its URL. Keep it
        private: anyone with it can post there.
      </div>
      <div class="mt-1.5 flex items-center gap-1.5">
        <input
          class="h-7 min-w-0 flex-1 rounded-md border bg-white/[0.035] px-2 text-[11px] placeholder:text-muted-foreground focus:outline-none {settings?.discordWebhook && !discordReady
            ? 'border-red-400/50'
            : 'border-white/10 focus:border-blue-400/40'}"
          type={showWebhook ? "text" : "password"}
          placeholder="https://discord.com/api/webhooks/…"
          aria-label="Discord webhook URL"
          autocomplete="off"
          spellcheck="false"
          disabled={!settings}
          value={settings?.discordWebhook ?? ""}
          on:change={(event) => update({ discordWebhook: event.currentTarget.value.trim() })}
        />
        <button type="button" class={BUTTON} on:click={() => (showWebhook = !showWebhook)}>
          {showWebhook ? "Hide" : "Show"}
        </button>
        {#if discordReady}
          <button type="button" class={BUTTON} on:click={() => test("discord")}>Send test</button>
        {/if}
      </div>
      {#if status.discord}
        <div class="mt-1 text-[10px] {status.discord.ok ? 'text-emerald-300' : 'text-red-300'}">{status.discord.text}</div>
      {:else if settings?.discordWebhook && !discordReady}
        <div class="mt-1 text-[10px] text-red-300">That isn't a Discord webhook URL.</div>
      {/if}
    </div>
  </div>
</section>


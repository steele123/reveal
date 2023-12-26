<script lang="ts">
    import { getVersion } from "@tauri-apps/api/app";
    import { appWindow } from "@tauri-apps/api/window";
</script>

<div
    data-tauri-drag-region
    class="flex border-b rounded-t-lg w-full select-none px-4 py-2"
>
    <div class="flex items-center gap-2">
        <div>
            <img alt="" src="/icon.png" class="w-5 h-5" />
        </div>
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
        <button class="" on:click={async () => appWindow.minimize()}>-</button>
        <button class="text-xs" on:click={() => appWindow.close()}>X</button>
    </div>
</div>

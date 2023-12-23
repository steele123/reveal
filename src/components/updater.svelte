<script lang="ts">
  import { onMount } from "svelte";
  import { checkUpdate, installUpdate } from "@tauri-apps/api/updater";

  onMount(async () => {
    const update = await checkUpdate();
    if (update.shouldUpdate) {
        console.log(`Installing update ${update.manifest?.version}, ${update.manifest?.date}, ${update.manifest?.body}`);
        await installUpdate();
    }   
  });
</script>

<script lang="ts">
  import '../app.css';
  import { onMount } from 'svelte';
  import { check, type Update } from '@tauri-apps/plugin-updater';
  import { relaunch } from '@tauri-apps/plugin-process';
  import SplashScreen from '$lib/components/SplashScreen.svelte';
  
  let { children } = $props();
  let updateInfo = $state<Update | null>(null);
  let isDownloading = $state(false);
  let downloadProgress = $state(0);

  onMount(async () => {
    try {
      const update = await check();
      if (update?.available) {
        updateInfo = update;
      }
    } catch (err) {
      console.error('Gagal cek update:', err);
    }
  });

  async function handleInstall() {
    if (!updateInfo) return;
    isDownloading = true;

    let downloadedBytes = 0;
    let totalBytes = 0;

    // Download & Install dengan event progress
    await updateInfo.downloadAndInstall((event) => {
      switch (event.event) {
        case 'Started':
          totalBytes = event.data.contentLength || 0;
          break;
        case 'Progress':
          downloadedBytes += event.data.chunkLength;
          if (totalBytes > 0) {
            downloadProgress = Math.round((downloadedBytes / totalBytes) * 100);
          }
          break;
        case 'Finished':
          console.log('Download selesai, bersiap restart...');
          break;
      }
    });

    // Auto-restart aplikasi setelah install beres
    await relaunch();
  }
</script>

<SplashScreen />

{#if updateInfo}
  <div class="modal modal-open modal-bottom sm:modal-middle">
    <div class="modal-box bg-base-200 border border-primary/20">
      <h3 class="text-lg font-bold text-primary">Pembaruan Tersedia! 🚀</h3>
      <p class="py-2 text-sm">Versi <strong>v{updateInfo.version}</strong> sudah rilis di GitHub.</p>
      
      {#if isDownloading}
        <div class="mt-4 space-y-2">
          <progress class="progress progress-primary w-full" value={downloadProgress} max="100"></progress>
          <p class="text-xs text-center opacity-70">Mengunduh... {downloadProgress}%</p>
        </div>
      {:else}
        <div class="modal-action">
          <button class="btn btn-ghost btn-sm" onclick={() => updateInfo = null}>Nanti Aja</button>
          <button class="btn btn-primary btn-sm" onclick={handleInstall}>Update Sekarang</button>
        </div>
      {/if}
    </div>
  </div>
{/if}

<div data-theme="dark" class="bg-base-300 text-base-content min-h-screen flex flex-col h-screen overflow-hidden select-none">
  {@render children()}
</div>
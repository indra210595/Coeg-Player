<script lang="ts">
  import '../app.css';
  import { onMount } from 'svelte';
  import { check } from '@tauri-apps/plugin-updater';
  import SplashScreen from '$lib/components/SplashScreen.svelte';
  
  let { children } = $props();

  onMount(async () => {
    try {
      const update = await check();
      if (update?.available) {
        // Panggil modal / toast pemberitahuan di sini
        console.log(`Versi baru ${update.version} tersedia!`);
      }
    } catch (err) {
      console.error('Gagal cek update:', err);
    }
  });
</script>

<SplashScreen />

<div data-theme="dark" class="bg-base-300 text-base-content min-h-screen flex flex-col h-screen overflow-hidden select-none">
  {@render children()}
</div>
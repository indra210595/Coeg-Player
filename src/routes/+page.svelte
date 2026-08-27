<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { player, type Song } from '$lib/stores/player.svelte';

  import SidebarLeft from '$lib/components/SidebarLeft.svelte';
  import CenterStage from '$lib/components/CenterStage.svelte';
  import SynapsePanel from '$lib/components/SynapsePanel.svelte';

  let showSynapse = $state(true);

  async function loadSongs() {
    try {
      player.songs = await invoke<Song[]>('get_songs');
      player.restoreLastSession();
    } catch (err) {
      console.error('Error fetching songs:', err);
    }
  }

  onMount(() => {
    loadSongs();
  });
</script>

<div class="flex flex-col h-screen overflow-hidden bg-base-300">

  <!-- 3 COLUMN CONTAINER -->
  <div class="flex-1 flex overflow-hidden">
    <SidebarLeft />
    <CenterStage />
    <SynapsePanel bind:isOpen={showSynapse} />
  </div>
</div>
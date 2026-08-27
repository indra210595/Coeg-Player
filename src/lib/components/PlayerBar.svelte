<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { player } from '$lib/stores/player.svelte';
  import { 
      Play, Pause, SkipBack, SkipForward, 
      Shuffle, Repeat, Repeat1, Volume2, VolumeX, Volume1, Music, Heart
  } from 'lucide-svelte';

  let coverUrl = $state<string | null>(null);

  $effect(() => {
      const coverPath = player.currentSong?.cover_path;
      if (coverPath) {
      invoke<number[]>('get_cover_bytes', { path: coverPath })
          .then((bytes) => {
          const blob = new Blob([new Uint8Array(bytes)], { type: 'image/jpeg' });
          if (coverUrl) URL.revokeObjectURL(coverUrl);
              coverUrl = URL.createObjectURL(blob);
          })
          .catch(() => {
          if (coverUrl) URL.revokeObjectURL(coverUrl);
              coverUrl = null;
          });
      } else {
          if (coverUrl) URL.revokeObjectURL(coverUrl);
          coverUrl = null;
      }
  });

  function formatTime(seconds: number) {
    if (!seconds || isNaN(seconds)) return '0:00';
    const mins = Math.floor(seconds / 60);
    const secs = Math.floor(seconds % 60);
    return `${mins}:${secs < 10 ? '0' : ''}${secs}`;
  }

  function handleSeek(event: MouseEvent) {
    const target = event.currentTarget as HTMLElement;
    const rect = target.getBoundingClientRect();
    const clickX = event.clientX - rect.left;
    const percentage = Math.max(0, Math.min(1, clickX / rect.width));
    
    if (player.duration) {
      player.seek(percentage * player.duration);
    }
  }

  function handleVolumeChange(e: Event) {
    const target = e.target as HTMLInputElement;
    player.setVolume(parseFloat(target.value));
  }

  let waveformBars = $derived.by(() => {
    if (!player.currentSong?.waveform) {
      return Array(100).fill(30);
    }
    try {
      return JSON.parse(player.currentSong.waveform) as number[];
    } catch {
      return Array(100).fill(30);
    }
  });
</script>

<div class="w-full bg-base-200 border-t border-base-100 px-6 py-2.5 flex flex-col gap-2 h-[115px] flex-shrink-0 shadow-lg select-none">

    <!-- WAVEFORM INTERACTIVE SEEKBAR -->
    <div class="w-full flex items-center gap-3">
        <span class="text-[10px] font-mono text-base-content/50 w-8 text-right">{formatTime(player.currentTime)}</span>
    
        <button 
            type="button"
            class="flex-1 h-9 bg-base-300/60 hover:bg-base-300 rounded-lg px-2 py-1 flex items-end justify-between gap-[2px] cursor-pointer group border border-base-100/50 transition-all"
            onclick={handleSeek}
            title="Klik untuk loncat durasi"
        >
            {#each waveformBars as heightPercent, i}
              {@const barProgress = (i / waveformBars.length)}
              {@const currentProgress = player.duration ? (player.currentTime / player.duration) : 0}
              {@const isPlayed = barProgress <= currentProgress}

              <div 
                  class="flex-1 rounded-full transition-colors duration-150 {isPlayed ? 'bg-primary' : 'bg-base-content/20 group-hover:bg-base-content/30'}"
                  style="height: {heightPercent}%"
              ></div>
            {/each}
        </button>

        <span class="text-[10px] font-mono text-base-content/50 w-8">{formatTime(player.duration)}</span>
    </div>

    <!-- 3-COLUMN GRID LAYOUT -->
    <div class="grid grid-cols-3 items-center w-full">
        
        <!-- LEFT (COL 1): TRACK METADATA & HEART BUTTON -->
        <div class="flex items-center gap-3 truncate pr-4">
            <div class="w-10 h-10 rounded-lg bg-base-300 border border-base-100/60 flex items-center justify-center flex-shrink-0 overflow-hidden relative group">
                {#if coverUrl}
                    <img src={coverUrl} alt="Cover" class="w-full h-full object-cover" />
                {:else}
                    <Music class="w-5 h-5 text-base-content/30" />
                {/if}
            </div>
            <div class="truncate">
                <div class="font-bold text-primary text-sm truncate">{player.currentSong?.title || 'Tidak ada lagu'}</div>
                <div class="text-xs text-base-content/60 truncate">{player.currentSong?.artist || 'Tidak ada artis'}</div>
            </div>

            {#if player.currentSong}
              <button 
                type="button"
                class="btn btn-ghost btn-xs btn-circle ml-1 shrink-0"
                onclick={() => player.currentSong && player.toggleFavorite(player.currentSong)}
                title={player.currentSong.is_favorite ? 'Hapus dari Favorit' : 'Tambah ke Favorit'}
              >
                <Heart 
                  class="w-4 h-4 transition-all {player.currentSong.is_favorite ? 'fill-primary text-primary scale-110' : 'text-base-content/30 hover:text-primary'}" 
                />
              </button>
            {/if}
        </div>

        <!-- CENTER (COL 2): PLAYBACK CONTROLS -->
        <div class="flex items-center justify-center gap-2">
            <button 
                class="btn btn-circle btn-ghost btn-xs {player.isShuffle ? 'text-primary' : 'text-base-content/30'}"
                onclick={() => player.toggleShuffle()}
                disabled={!player.currentSong}
                title="Shuffle"
            >
                <Shuffle class="w-3.5 h-3.5" />
            </button>

            <button 
                class="btn btn-circle btn-ghost btn-xs text-base-content/80 hover:text-primary" 
                onclick={() => player.prev()}
                disabled={!player.currentSong}
            >
                <SkipBack class="w-4 h-4 fill-current" />
            </button>

            <button 
                class="btn btn-circle btn-primary btn-md shadow-lg shadow-primary/20 mx-1" 
                onclick={() => player.toggle()}
                disabled={!player.currentSong}
            >
                {#if player.isPlaying}
                  <Pause class="w-5 h-5 fill-current" />
                {:else}
                  <Play class="w-5 h-5 fill-current ml-0.5" />
                {/if}
            </button>

            <button 
                class="btn btn-circle btn-ghost btn-xs text-base-content/80 hover:text-primary" 
                onclick={() => player.next()}
                disabled={!player.currentSong}
            >
                <SkipForward class="w-4 h-4 fill-current" />
            </button>

            <button 
                class="btn btn-circle btn-ghost btn-xs {player.repeatMode !== 'off' ? 'text-primary' : 'text-base-content/30'}"
                onclick={() => player.toggleRepeat()}
                disabled={!player.currentSong}
                title="Repeat Mode"
            >
                {#if player.repeatMode === 'one'}
                  <Repeat1 class="w-3.5 h-3.5" />
                {:else}
                  <Repeat class="w-3.5 h-3.5" />
                {/if}
            </button>
        </div>

        <!-- RIGHT (COL 3): VOLUME CONTROL -->
        <div class="flex items-center justify-end gap-2 pl-4">
          <button class="btn btn-circle btn-ghost btn-xs text-base-content/60" onclick={() => player.toggleMute()}>
              {#if player.isMuted || player.volume === 0}
                  <VolumeX class="w-4 h-4 text-error" />
              {:else if player.volume < 0.5}
                  <Volume1 class="w-4 h-4" />
              {:else}
                  <Volume2 class="w-4 h-4" />
              {/if}
          </button>

          <input 
              type="range" 
              min="0" 
              max="1" 
              step="0.01" 
              value={player.isMuted ? 0 : player.volume}
              oninput={handleVolumeChange}
              class="range range-xs range-primary w-24 cursor-pointer" 
          />
        </div>

    </div>
</div>
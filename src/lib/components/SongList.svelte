<script lang="ts">
  import { createVirtualizer } from '@tanstack/svelte-virtual';
  import { player, type Song } from '$lib/stores/player.svelte';
  import { Play, Music2, Search, Clock, Volume2, Target, Heart } from 'lucide-svelte';
  import { untrack } from 'svelte';

  let scrollContainer = $state<HTMLDivElement | null>(null);
  let searchQuery = $state('');

  // Filter lagu instant
  let filteredSongs = $derived.by(() => {
    if (!searchQuery.trim()) return player.songs;
    const q = searchQuery.toLowerCase().trim();
    return player.songs.filter(
      (s) =>
        s.title.toLowerCase().includes(q) ||
        s.artist.toLowerCase().includes(q) ||
        (s.album && s.album.toLowerCase().includes(q))
    );
  });

  const virtualizer = createVirtualizer({
    count: 0,
    getScrollElement: () => scrollContainer,
    estimateSize: () => 50,
    overscan: 10
  });

  $effect(() => {
    const count = filteredSongs.length;
    const container = scrollContainer;

    if (container) {
      untrack(() => {
        $virtualizer.setOptions({
          count,
          getScrollElement: () => container,
          estimateSize: () => 50,
          overscan: 10
        });
        $virtualizer.measure();
      });
    }
  });

  let activeIndex = $derived.by(() => {
    if (!player.currentSong) return -1;
    return filteredSongs.findIndex((s) => s.id === player.currentSong?.id);
  });

  function scrollToActiveSong() {
    if (activeIndex !== -1) {
      $virtualizer.scrollToIndex(activeIndex, { align: 'center' });
    }
  }

  function formatTime(seconds: number) {
    if (!seconds || isNaN(seconds)) return '0:00';
    const mins = Math.floor(seconds / 60);
    const secs = Math.floor(seconds % 60);
    return `${mins}:${secs < 10 ? '0' : ''}${secs}`;
  }
</script>

<div class="h-full w-full flex flex-col bg-base-300 overflow-hidden">
  
  <!-- STICKY HEADER SEARCH & LOCATE BAR -->
  <div class="p-3 bg-base-200/80 border-b border-base-100 flex items-center justify-between gap-4 flex-shrink-0 select-none">
    <div class="relative flex-1 max-w-md">
      <Search class="w-4 h-4 absolute left-3 top-1/2 -translate-y-1/2 text-base-content/40" />
      <input 
        type="text" 
        placeholder="Cari judul, artis, atau album..."
        bind:value={searchQuery}
        class="input input-sm w-full pl-9 bg-base-300 border-base-100 focus:border-primary text-xs rounded-xl"
      />
      {#if searchQuery}
        <button 
          class="absolute right-3 top-1/2 -translate-y-1/2 text-xs text-base-content/40 hover:text-base-content"
          onclick={() => searchQuery = ''}
        >
          ✕
        </button>
      {/if}
    </div>

    <div class="flex items-center gap-2 text-xs font-mono">
      {#if activeIndex !== -1}
        <button 
          type="button"
          class="btn btn-xs btn-primary gap-1.5 font-bold shadow-md shadow-primary/20 animate-pulse hover:animate-none"
          onclick={scrollToActiveSong}
          title="Scroll ke lagu yang lagi diputar"
        >
          <Target class="w-3.5 h-3.5" />
          <span>#{activeIndex + 1}</span>
        </button>
      {/if}

      <span class="badge badge-primary badge-outline text-[10px] font-bold">
        {filteredSongs.length.toLocaleString()} SONGS
      </span>
    </div>
  </div>

  <!-- COLUMN HEADER TABLE (12-COL GRID PAS) -->
  <div class="grid grid-cols-12 gap-2 px-4 py-2 bg-base-200/40 border-b border-base-100/60 text-[10px] font-mono font-bold text-base-content/40 uppercase tracking-wider select-none flex-shrink-0">
    <div class="col-span-1 text-center">#</div>
    <div class="col-span-6 md:col-span-4">Title / Artist</div>
    <div class="col-span-3 hidden md:block">Album</div>
    <div class="col-span-2 hidden lg:block text-center">Format</div>
    <div class="col-span-5 md:col-span-2 text-right flex items-center justify-end gap-1">
      <Clock class="w-3 h-3" /> Duration
    </div>
  </div>

  <!-- VIRTUAL SCROLL CONTAINER -->
  <div 
    bind:this={scrollContainer} 
    class="flex-1 w-full overflow-y-auto scroll-smooth"
  >
    {#if filteredSongs.length === 0}
      <div class="text-center text-base-content/40 my-16 flex flex-col items-center gap-3 select-none">
        <Music2 class="w-12 h-12 opacity-20 animate-bounce" />
        <p class="text-sm font-semibold">Lagu tidak ditemukan</p>
        <p class="text-xs font-mono opacity-60">Coba kata kunci pencarian yang lain.</p>
      </div>
    {:else}
      <div style="height: {$virtualizer.getTotalSize()}px; width: 100%; position: relative;">
        {#each $virtualizer.getVirtualItems() as row (row.key)}
          {@const song = filteredSongs[row.index]}
          {@const isCurrent = player.currentSong?.id === song.id}
          {@const trackIndex = (row.index + 1) < 10 ? `0${row.index + 1}` : `${row.index + 1}`}
          
          <div 
            class="absolute top-0 left-0 w-full grid grid-cols-12 gap-2 px-4 items-center border-b border-base-100/30 transition-all cursor-pointer group hover:bg-base-200/60 {isCurrent ? 'bg-primary/10 border-l-4 border-l-primary' : ''}"
            style="height: {row.size}px; transform: translateY({row.start}px);"
            onclick={() => player.play(song, filteredSongs)}
            role="button"
            tabindex="0"
            onkeydown={(e) => e.key === 'Enter' && player.play(song, filteredSongs)}
          >
            <!-- COL 1: INDEX / EQUALIZER -->
            <div class="col-span-1 flex items-center justify-center font-mono text-xs">
              {#if isCurrent && player.isPlaying}
                <Volume2 class="w-4 h-4 text-primary animate-pulse" />
              {:else}
                <span class="text-base-content/30 group-hover:hidden">{trackIndex}</span>
                <Play class="w-3.5 h-3.5 text-primary hidden group-hover:block fill-current" />
              {/if}
            </div>

            <!-- COL 2: TITLE & ARTIST -->
            <div class="col-span-6 md:col-span-4 flex flex-col justify-center truncate pr-2">
              <div class="font-semibold text-xs text-base-content truncate group-hover:text-primary transition-colors {isCurrent ? 'text-primary font-bold' : ''}">
                {song.title}
              </div>
              <div class="text-[11px] text-base-content/50 truncate">
                {song.artist}
              </div>
            </div>

            <!-- COL 3: ALBUM -->
            <div class="col-span-3 hidden md:block text-xs text-base-content/60 truncate pr-2">
              {song.album || 'Unknown Album'}
            </div>

            <!-- COL 4: FORMAT / LOSSLESS -->
            <div class="col-span-2 hidden lg:flex items-center justify-center gap-1.5">
              <span class="badge badge-xs font-mono text-[9px] font-bold {song.is_lossless ? 'badge-success text-black' : 'badge-neutral text-base-content/60'}">
                {song.is_lossless ? 'LOSSLESS' : 'LOSSY'}
              </span>
              <span class="text-[10px] font-mono text-primary/80">
                {song.format || 'MP3'}
              </span>
            </div>

            <!-- COL 5: FAVORITE & DURATION -->
            <div class="col-span-5 md:col-span-2 flex items-center justify-end gap-3 font-mono text-xs text-base-content/50">
              <button 
                type="button"
                class="btn btn-ghost btn-xs btn-circle hover:bg-base-300 transition-colors"
                onclick={(e) => {
                  e.stopPropagation();
                  player.toggleFavorite(song);
                }}
                title={song.is_favorite ? 'Hapus dari Favorit' : 'Tambah ke Favorit'}
              >
                <Heart 
                  class="w-4 h-4 transition-all {song.is_favorite ? 'fill-primary text-primary scale-110' : 'text-base-content/30 hover:text-primary'}" 
                />
              </button>

              <span>{formatTime(song.duration)}</span>
            </div>
          </div>
        {/each}
      </div>
    {/if}
  </div>

</div>
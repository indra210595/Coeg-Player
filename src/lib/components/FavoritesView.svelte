<script lang="ts">
  import { player, type Song } from '$lib/stores/player.svelte';
  import AlbumThumb from '$lib/components/AlbumThumb.svelte';
  import { Play, Heart, Search, Music, Volume2 } from 'lucide-svelte';

  let searchQuery = $state('');

  // Filter lagu yang hanya berstatus is_favorite = true
  let favoriteSongs = $derived.by(() => {
    let list = player.songs.filter((s) => s.is_favorite);
    if (searchQuery.trim()) {
      const q = searchQuery.toLowerCase().trim();
      list = list.filter(
        (s) => s.title.toLowerCase().includes(q) || s.artist.toLowerCase().includes(q)
      );
    }
    return list;
  });

  function playFavoriteTrack(song: Song) {
    player.play(song, favoriteSongs);
  }

  function formatTime(seconds: number) {
    if (!seconds || isNaN(seconds)) return '0:00';
    const mins = Math.floor(seconds / 60);
    const secs = Math.floor(seconds % 60);
    return `${mins}:${secs < 10 ? '0' : ''}${secs}`;
  }
</script>

<div class="h-full w-full flex flex-col bg-base-300 overflow-hidden relative">
  
  <!-- HEADER FAVORITES VIEW -->
  <div class="p-4 bg-linear-to-b from-primary/10 via-base-200/80 to-base-300 border-b border-base-100 flex flex-col sm:flex-row items-center justify-between gap-4 shrink-0 select-none">
    <div class="flex items-center gap-3">
      <div class="w-12 h-12 rounded-2xl bg-primary/20 border border-primary/30 flex items-center justify-center text-primary shadow-lg shadow-primary/20">
        <Heart class="w-6 h-6 fill-current" />
      </div>
      <div>
        <h2 class="text-lg font-black text-base-content">Favorites</h2>
        <p class="text-xs text-base-content/50 font-mono">
          {favoriteSongs.length} Lagu Favorit
        </p>
      </div>
    </div>

    <!-- SEARCH BAR -->
    <div class="relative w-full sm:w-72">
      <Search class="w-4 h-4 absolute left-3 top-1/2 -translate-y-1/2 text-base-content/40" />
      <input 
        type="text" 
        placeholder="Cari lagu favorit..."
        bind:value={searchQuery}
        class="input input-sm w-full pl-9 bg-base-300 border-base-100 focus:border-primary text-xs rounded-xl"
      />
    </div>
  </div>

  <!-- TRACKLIST SONGS -->
  <div class="flex-1 overflow-y-auto p-4 scroll-smooth">
    {#if favoriteSongs.length === 0}
      <div class="text-center text-base-content/40 my-20 flex flex-col items-center gap-3 select-none">
        <Heart class="w-12 h-12 opacity-20" />
        <p class="text-sm font-semibold">Belum ada lagu favorit</p>
        <span class="text-xs">Klik ikon hati pada lagu untuk menyimpannya di sini.</span>
      </div>
    {:else}
      <div class="flex flex-col gap-1.5">
        {#each favoriteSongs as song, idx}
          {@const isCurrent = player.currentSong?.id === song.id}
          <div 
            class="grid grid-cols-12 gap-2 px-3 py-2 items-center rounded-2xl transition-all cursor-pointer group hover:bg-base-200/80 border border-transparent hover:border-base-100/50 {isCurrent ? 'bg-primary/10 border-primary/30' : ''}"
            onclick={() => playFavoriteTrack(song)}
            onkeydown={(e) => { if (e.key === 'Enter') playFavoriteTrack(song); }}
            role="button"
            tabindex="0"
          >
            <!-- INDEX & EQUALIZER -->
            <div class="col-span-1 flex items-center justify-center font-mono text-xs">
              {#if isCurrent && player.isPlaying}
                <Volume2 class="w-4 h-4 text-primary animate-pulse" />
              {:else}
                <span class="text-base-content/40 group-hover:hidden">{(idx + 1).toString().padStart(2, '0')}</span>
                <Play class="w-3.5 h-3.5 text-primary hidden group-hover:block fill-current" />
              {/if}
            </div>

            <!-- COVER & TITLE -->
            <div class="col-span-7 md:col-span-8 flex items-center gap-3 truncate pr-2">
              <div class="w-9 h-9 rounded-lg overflow-hidden shrink-0 border border-base-100">
                <AlbumThumb coverPath={song.cover_path} sizeClass="w-full h-full object-cover" />
              </div>
              <div class="flex flex-col truncate">
                <span class="text-xs font-bold text-base-content truncate group-hover:text-primary transition-colors {isCurrent ? 'text-primary' : ''}">
                  {song.title}
                </span>
                <span class="text-[10px] text-base-content/50 truncate">
                  {song.artist} • {song.album}
                </span>
              </div>
            </div>

            <!-- DURATION & UNFAVORITE BUTTON -->
            <div class="col-span-4 md:col-span-3 flex items-center justify-end gap-3 font-mono text-xs text-base-content/50">
              <button 
                type="button"
                class="btn btn-ghost btn-xs btn-circle text-primary hover:scale-110 transition-transform"
                onclick={(e) => {
                  e.stopPropagation();
                  player.toggleFavorite(song);
                }}
                title="Hapus dari Favorit"
              >
                <Heart class="w-4 h-4 fill-current" />
              </button>
              <span>{formatTime(song.duration)}</span>
            </div>
          </div>
        {/each}
      </div>
    {/if}
  </div>

</div>
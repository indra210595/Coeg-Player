<script lang="ts">
  import { player, type Song } from '$lib/stores/player.svelte';
  import AlbumThumb from '$lib/components/AlbumThumb.svelte';
  import { Play, Disc3, Search, Music, Shuffle, X, Volume2 } from 'lucide-svelte';
  import { untrack } from 'svelte';

  let searchQuery = $state('');
  let displayLimit = $state(40);
  let scrollContainer = $state<HTMLDivElement | null>(null);
  
  let selectedAlbumKey = $state<string | null>(null);

  interface AlbumGroup {
    key: string;
    album: string;
    artist: string;
    cover_path?: string | null;
    songs: Song[];
    totalDuration: number;
  }

  // Helper mengambil folder induk file
  function getParentDir(filePath: string): string {
    if (!filePath) return '';
    const normalized = filePath.replace(/\\/g, '/');
    const lastSlash = normalized.lastIndexOf('/');
    if (lastSlash === -1) return normalized;
    const parent = normalized.substring(0, lastSlash);
    return parent.replace(/\/(?:cd|disc)\s*\d+$/i, '');
  }

  // Helper mengekstrak Artis Utama (Lead Artist) sebelum koma, &, feat, ft, dll.
  function getLeadArtist(artistStr: string): string {
    if (!artistStr || artistStr.toLowerCase() === 'unknown artist') return 'unknown artist';
    const parts = artistStr.split(/\s*(?:,|&|;|\/|feat\.|ft\.|featuring|with|x)\s*/i);
    return parts[0].trim() || 'unknown artist';
  }

  // Helper memformat nama artis album secara cerdas
  function formatAlbumArtist(songs: Song[]): string {
    const rawArtists = songs.map((s) => s.artist?.trim() || 'Unknown Artist');
    const uniqueArtists = Array.from(new Set(rawArtists));

    if (uniqueArtists.length === 0) return 'Unknown Artist';
    if (uniqueArtists.length === 1) return uniqueArtists[0];

    const primaryArtists = songs.map((s) => getLeadArtist(s.artist || ''));
    const uniquePrimary = Array.from(
      new Set(primaryArtists.filter((a) => a && a.toLowerCase() !== 'unknown artist'))
    );

    if (uniquePrimary.length === 1) return uniquePrimary[0];
    if (uniquePrimary.length <= 3) return uniquePrimary.join(', ');
    return `${uniquePrimary.slice(0, 2).join(', ')} & others`;
  }

  // Smart Grouping: (Nama Album + Lead Artist)
  let albumGroups = $derived.by(() => {
    const map = new Map<string, AlbumGroup>();

    for (const song of player.songs) {
      const albumName = song.album?.trim() || 'Unknown Album';
      const artistName = song.artist?.trim() || 'Unknown Artist';
      const isUnknownAlbum = albumName.toLowerCase() === 'unknown album';
      const dirPath = getParentDir(song.file_path).toLowerCase();
      const leadArtist = getLeadArtist(artistName).toLowerCase();

      // KUNCI UTAMA: Untuk album bernama, kunci pake (Nama Album + Lead Artist)
      // Untuk Unknown Album, kunci pake (Unknown + Lead Artist + Path Folder)
      const key = isUnknownAlbum 
        ? `unknown___${leadArtist}___${dirPath}` 
        : `${albumName.toLowerCase()}___${leadArtist}`;

      if (!map.has(key)) {
        map.set(key, {
          key,
          album: albumName,
          artist: artistName,
          cover_path: song.cover_path,
          songs: [],
          totalDuration: 0
        });
      }

      const item = map.get(key)!;
      item.songs.push(song);
      item.totalDuration += song.duration || 0;

      if (!item.cover_path && song.cover_path) {
        item.cover_path = song.cover_path;
      }
    }

    const result = Array.from(map.values());

    for (const group of result) {
      group.artist = formatAlbumArtist(group.songs);
    }

    return result;
  });

  let selectedAlbum = $derived.by(() => {
    if (!selectedAlbumKey) return null;
    return albumGroups.find((a) => a.key === selectedAlbumKey) || null;
  });

  let filteredAlbums = $derived.by(() => {
    if (!searchQuery.trim()) return albumGroups;
    const q = searchQuery.toLowerCase().trim();
    return albumGroups.filter(
      (a) => a.album.toLowerCase().includes(q) || a.artist.toLowerCase().includes(q)
    );
  });

  $effect(() => {
    searchQuery;
    untrack(() => {
      displayLimit = 40;
    });
  });

  let visibleAlbums = $derived(filteredAlbums.slice(0, displayLimit));

  function handleScroll() {
    if (!scrollContainer) return;
    const { scrollTop, scrollHeight, clientHeight } = scrollContainer;
    
    if (scrollTop + clientHeight >= scrollHeight - 300) {
      if (displayLimit < filteredAlbums.length) {
        displayLimit += 40;
      }
    }
  }

  function playAlbum(album: AlbumGroup) {
    if (album.songs.length > 0) {
      player.play(album.songs[0], album.songs);
    }
  }

  function playAlbumTrack(album: AlbumGroup, song: Song) {
    player.play(song, album.songs);
  }

  function shufflePlayAlbum(album: AlbumGroup) {
    if (album.songs.length > 0) {
      if (!player.isShuffle) player.toggleShuffle();
      const randomIdx = Math.floor(Math.random() * album.songs.length);
      player.play(album.songs[randomIdx], album.songs);
    }
  }

  function formatTime(seconds: number) {
    if (!seconds || isNaN(seconds)) return '0:00';
    const mins = Math.floor(seconds / 60);
    const secs = Math.floor(seconds % 60);
    return `${mins}:${secs < 10 ? '0' : ''}${secs}`;
  }

  function formatTotalDuration(seconds: number) {
    const mins = Math.floor(seconds / 60);
    if (mins < 60) return `${mins} mins`;
    const hrs = Math.floor(mins / 60);
    const remMins = mins % 60;
    return `${hrs} hr ${remMins} mins`;
  }
</script>

<div class="h-full w-full flex flex-col bg-base-300 overflow-hidden relative">
  
  <!-- STICKY HEADER SEARCH BAR -->
  <div class="p-3 bg-base-200/80 border-b border-base-100 flex items-center justify-between gap-4 shrink-0 select-none">
    <div class="relative flex-1 max-w-md">
      <Search class="w-4 h-4 absolute left-3 top-1/2 -translate-y-1/2 text-base-content/40" />
      <input 
        type="text" 
        placeholder="Cari album atau artis..."
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

    <span class="badge badge-primary badge-outline text-[10px] font-mono font-bold">
      {filteredAlbums.length.toLocaleString()} ALBUMS
    </span>
  </div>

  <!-- GRID CONTAINERS ALBUM CARDS -->
  <div 
    bind:this={scrollContainer}
    onscroll={handleScroll}
    class="flex-1 overflow-y-auto p-4 scroll-smooth"
  >
    {#if filteredAlbums.length === 0}
      <div class="text-center text-base-content/40 my-20 flex flex-col items-center gap-3 select-none">
        <Disc3 class="w-12 h-12 opacity-20 animate-spin-slow" />
        <p class="text-sm font-semibold">Album tidak ditemukan</p>
      </div>
    {:else}
      <div class="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 xl:grid-cols-6 gap-4">
        {#each visibleAlbums as item}
          <div 
            class="group relative bg-base-200/50 hover:bg-base-200 border border-base-100/60 hover:border-primary/40 rounded-2xl p-3 flex flex-col gap-3 transition-all duration-300 hover:-translate-y-1 hover:shadow-xl hover:shadow-primary/10 cursor-pointer text-left w-full"
            onclick={() => selectedAlbumKey = item.key}
            onkeydown={(e) => { if (e.key === 'Enter' || e.key === ' ') selectedAlbumKey = item.key; }}
            role="button"
            tabindex="0"
          >
            <!-- COVER ART CONTAINER -->
            <div class="w-full aspect-square rounded-xl bg-base-100 border border-base-100/50 relative overflow-hidden flex items-center justify-center shadow-md">
              {#if item.cover_path}
                <AlbumThumb coverPath={item.cover_path} sizeClass="w-full h-full object-cover group-hover:scale-105 transition-transform duration-500" />
              {:else}
                <Music class="w-12 h-12 text-primary/30" />
              {/if}

              <!-- HOVER QUICK PLAY BUTTON -->
              <div class="absolute inset-0 bg-black/40 opacity-0 group-hover:opacity-100 transition-opacity flex items-center justify-center">
                <button 
                  type="button"
                  class="btn btn-circle btn-primary shadow-lg shadow-primary/40 translate-y-2 group-hover:translate-y-0 transition-all duration-300"
                  onclick={(e) => {
                    e.stopPropagation();
                    playAlbum(item);
                  }}
                  title="Putar Album Instan"
                >
                  <Play class="w-5 h-5 fill-current ml-0.5" />
                </button>
              </div>
            </div>

            <!-- METADATA ALBUM -->
            <div class="flex flex-col truncate w-full">
              <span class="text-xs font-bold text-base-content truncate group-hover:text-primary transition-colors" title={item.album}>
                {item.album}
              </span>
              <span class="text-[11px] text-base-content/50 truncate mt-0.5" title={item.artist}>
                {item.artist}
              </span>
              <span class="text-[9px] font-mono text-primary/80 mt-1">
                {item.songs.length} {item.songs.length > 1 ? 'TRACKS' : 'TRACK'}
              </span>
            </div>

          </div>
        {/each}
      </div>

      {#if displayLimit < filteredAlbums.length}
        <div class="text-center py-6 text-xs font-mono text-base-content/40">
          Memuat album lainnya... ({visibleAlbums.length} / {filteredAlbums.length})
        </div>
      {/if}
    {/if}
  </div>

  <!-- MODAL DETAIL ALBUM OVERLAY -->
  {#if selectedAlbum}
    <div 
      class="absolute inset-0 bg-black/75 backdrop-blur-md z-50 flex items-center justify-center p-4 md:p-6 animate-in fade-in duration-200"
      onclick={(e) => { if (e.target === e.currentTarget) selectedAlbumKey = null; }}
      onkeydown={(e) => { if (e.key === 'Escape') selectedAlbumKey = null; }}
      role="button"
      tabindex="0"
    >
      <div 
        class="bg-base-200 border border-base-100 rounded-3xl w-full max-w-2xl flex flex-col overflow-hidden shadow-2xl relative cursor-default"
      >
        <!-- CLOSE BUTTON -->
        <button 
          class="btn btn-sm btn-circle btn-ghost absolute top-3 right-3 z-10 text-base-content/60 hover:text-base-content hover:bg-base-300"
          onclick={() => selectedAlbumKey = null}
        >
          <X class="w-4 h-4" />
        </button>

        <!-- HEADER DETAIL ALBUM -->
        <div class="p-5 bg-linear-to-b from-base-100/60 to-base-200 border-b border-base-100 flex flex-col sm:flex-row gap-5 items-center sm:items-start shrink-0">
          <div class="w-28 h-28 md:w-32 md:h-32 rounded-2xl bg-base-300 border border-base-100 shadow-xl overflow-hidden shrink-0 flex items-center justify-center">
            {#if selectedAlbum.cover_path}
              <AlbumThumb coverPath={selectedAlbum.cover_path} sizeClass="w-full h-full object-cover" />
            {:else}
              <Music class="w-12 h-12 text-primary/30" />
            {/if}
          </div>

          <div class="flex-1 flex flex-col gap-2 text-center sm:text-left truncate w-full">
            <span class="badge badge-xs badge-primary font-bold self-center sm:self-start">ALBUM</span>
            <h2 class="text-lg md:text-xl font-bold text-base-content truncate" title={selectedAlbum.album}>
              {selectedAlbum.album}
            </h2>
            <p class="text-xs font-semibold text-primary truncate" title={selectedAlbum.artist}>
              {selectedAlbum.artist}
            </p>

            <div class="text-[11px] font-mono text-base-content/50 flex items-center justify-center sm:justify-start gap-2 mt-0.5">
              <span>{selectedAlbum.songs.length} Tracks</span>
              <span>•</span>
              <span>{formatTotalDuration(selectedAlbum.totalDuration)}</span>
            </div>

            <!-- ACTION BUTTONS -->
            <div class="flex items-center justify-center sm:justify-start gap-2.5 mt-1">
              <button 
                class="btn btn-xs btn-primary gap-1.5 font-bold shadow-md shadow-primary/20 rounded-lg px-3"
                onclick={() => selectedAlbum && playAlbum(selectedAlbum)}
              >
                <Play class="w-3.5 h-3.5 fill-current" /> Play All
              </button>
              <button 
                class="btn btn-xs btn-outline gap-1.5 font-medium rounded-lg px-3"
                onclick={() => selectedAlbum && shufflePlayAlbum(selectedAlbum)}
              >
                <Shuffle class="w-3.5 h-3.5" /> Shuffle
              </button>
            </div>
          </div>
        </div>

        <!-- TRACKLIST CONTAINER -->
        <div class="p-4 flex flex-col shrink-0">
          <div class="grid grid-cols-12 gap-2 px-3 py-1.5 text-[10px] font-mono font-bold text-base-content/40 uppercase tracking-wider border-b border-base-100/50 mb-1.5 shrink-0">
            <div class="col-span-1 text-center">#</div>
            <div class="col-span-8 md:col-span-9">Title / Artist</div>
            <div class="col-span-3 md:col-span-2 text-right">Duration</div>
          </div>

          <div class="max-h-[240px] overflow-y-auto scroll-smooth pr-1 flex flex-col gap-1">
            {#each selectedAlbum.songs as song, idx}
              {@const isCurrent = player.currentSong?.id === song.id}
              <button 
                type="button"
                class="grid grid-cols-12 gap-2 px-3 py-2 items-center rounded-xl transition-all cursor-pointer group hover:bg-base-300/80 w-full text-left {isCurrent ? 'bg-primary/10 border-l-4 border-l-primary' : ''}"
                onclick={() => selectedAlbum && playAlbumTrack(selectedAlbum, song)}
              >
                <!-- INDEX / EQUALIZER -->
                <div class="col-span-1 flex items-center justify-center font-mono text-xs">
                  {#if isCurrent && player.isPlaying}
                    <Volume2 class="w-4 h-4 text-primary animate-pulse" />
                  {:else}
                    <span class="text-base-content/40 group-hover:hidden">{(idx + 1).toString().padStart(2, '0')}</span>
                    <Play class="w-3.5 h-3.5 text-primary hidden group-hover:block fill-current" />
                  {/if}
                </div>

                <!-- TITLE & ARTIST SUBTITLE -->
                <div class="col-span-8 md:col-span-9 flex items-center gap-2 truncate pr-2">
                  <div class="flex flex-col truncate">
                    <span class="text-xs font-semibold text-base-content truncate group-hover:text-primary transition-colors {isCurrent ? 'text-primary font-bold' : ''}">
                      {song.title}
                    </span>
                    <span class="text-[10px] text-base-content/50 truncate">
                      {song.artist}
                    </span>
                  </div>
                  {#if song.is_lossless}
                    <span class="badge badge-xs badge-success text-black text-[9px] font-bold font-mono shrink-0">FLAC</span>
                  {/if}
                </div>

                <!-- DURATION -->
                <div class="col-span-3 md:col-span-2 text-right font-mono text-xs text-base-content/50">
                  {formatTime(song.duration)}
                </div>
              </button>
            {/each}
          </div>
        </div>

      </div>
    </div>
  {/if}

</div>
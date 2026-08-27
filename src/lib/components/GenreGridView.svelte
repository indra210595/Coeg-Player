<script lang="ts">
  import { player, type Song } from '$lib/stores/player.svelte';
  import { Play, Tag, Search, Shuffle, X, Volume2 } from 'lucide-svelte';
  import { untrack } from 'svelte';

  let searchQuery = $state('');
  let displayLimit = $state(40);
  let scrollContainer = $state<HTMLDivElement | null>(null);
  
  let selectedGenreKey = $state<string | null>(null);

  interface GenreGroup {
    key: string;
    genre: string;
    songs: Song[];
    totalDuration: number;
  }

  // 1. Grouping Lagu Berdasarkan Genre (Dipisah Pake Koma)
  let genreGroups = $derived.by(() => {
    const map = new Map<string, GenreGroup>();

    for (const song of player.songs) {
      const rawGenre = song.genre?.trim() || 'Unknown Genre';
      
      // Split genre berbasis koma & trim spasi
      const genreList = rawGenre
        .split(',')
        .map((g) => g.trim())
        .filter((g) => g.length > 0);

      const finalGenres = genreList.length > 0 ? genreList : ['Unknown Genre'];

      for (const genreName of finalGenres) {
        const key = genreName.toLowerCase();

        if (!map.has(key)) {
          map.set(key, {
            key,
            genre: genreName,
            songs: [],
            totalDuration: 0
          });
        }

        const item = map.get(key)!;
        // Hindari duplikasi jika lagu yang sama menuliskan genre ganda yang mirip
        if (!item.songs.some((s) => s.id === song.id)) {
          item.songs.push(song);
          item.totalDuration += song.duration || 0;
        }
      }
    }

    // Urutkan dari genre yang punya lagu terbanyak
    return Array.from(map.values()).sort((a, b) => b.songs.length - a.songs.length);
  });

  let selectedGenre = $derived.by(() => {
    if (!selectedGenreKey) return null;
    return genreGroups.find((g) => g.key === selectedGenreKey) || null;
  });

  let filteredGenres = $derived.by(() => {
    if (!searchQuery.trim()) return genreGroups;
    const q = searchQuery.toLowerCase().trim();
    return genreGroups.filter((g) => g.genre.toLowerCase().includes(q));
  });

  $effect(() => {
    searchQuery;
    untrack(() => {
      displayLimit = 40;
    });
  });

  let visibleGenres = $derived(filteredGenres.slice(0, displayLimit));

  function handleScroll() {
    if (!scrollContainer) return;
    const { scrollTop, scrollHeight, clientHeight } = scrollContainer;
    
    if (scrollTop + clientHeight >= scrollHeight - 300) {
      if (displayLimit < filteredGenres.length) {
        displayLimit += 40;
      }
    }
  }

  function playGenre(group: GenreGroup) {
    if (group.songs.length > 0) {
      player.play(group.songs[0], group.songs);
    }
  }

  function playGenreTrack(group: GenreGroup, song: Song) {
    player.play(song, group.songs);
  }

  function shufflePlayGenre(group: GenreGroup) {
    if (group.songs.length > 0) {
      if (!player.isShuffle) player.toggleShuffle();
      const randomIdx = Math.floor(Math.random() * group.songs.length);
      player.play(group.songs[randomIdx], group.songs);
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
        placeholder="Cari genre..."
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
      {filteredGenres.length.toLocaleString()} GENRES
    </span>
  </div>

  <!-- GRID CONTAINER GENRE CARDS (TEXT BASED / NO IMAGE) -->
  <div 
    bind:this={scrollContainer}
    onscroll={handleScroll}
    class="flex-1 overflow-y-auto p-4 scroll-smooth"
  >
    {#if filteredGenres.length === 0}
      <div class="text-center text-base-content/40 my-20 flex flex-col items-center gap-3 select-none">
        <Tag class="w-12 h-12 opacity-20" />
        <p class="text-sm font-semibold">Genre tidak ditemukan</p>
      </div>
    {:else}
      <div class="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 xl:grid-cols-6 gap-4">
        {#each visibleGenres as item}
          <div 
            class="group relative bg-linear-to-br from-base-200/80 to-base-200 border border-base-100/80 hover:border-primary/50 rounded-2xl p-4 flex flex-col justify-between aspect-square transition-all duration-300 hover:-translate-y-1 hover:shadow-xl hover:shadow-primary/10 cursor-pointer text-left w-full overflow-hidden"
            onclick={() => selectedGenreKey = item.key}
            onkeydown={(e) => { if (e.key === 'Enter' || e.key === ' ') selectedGenreKey = item.key; }}
            role="button"
            tabindex="0"
          >
            <!-- BACKGROUND DECORATIVE ICON -->
            <Tag class="w-24 h-24 absolute -right-6 -bottom-6 text-primary/5 group-hover:text-primary/10 transition-all duration-500 group-hover:scale-110 rotate-12" />

            <!-- CARD TOP BADGE -->
            <div class="flex items-center justify-between w-full z-10">
              <span class="badge badge-xs badge-primary font-bold font-mono text-[9px] uppercase tracking-wider">
                {item.songs.length} {item.songs.length > 1 ? 'TRACKS' : 'TRACK'}
              </span>

              <!-- HOVER QUICK PLAY BUTTON -->
              <button 
                type="button"
                class="btn btn-circle btn-xs btn-primary opacity-0 group-hover:opacity-100 transition-all duration-300 shadow-md shadow-primary/30"
                onclick={(e) => {
                  e.stopPropagation();
                  playGenre(item);
                }}
                title="Putar Genre Instan"
              >
                <Play class="w-3 h-3 fill-current ml-0.5" />
              </button>
            </div>

            <!-- CARD BOTTOM GENRE TITLE -->
            <div class="flex flex-col gap-0.5 z-10 mt-auto">
              <h3 class="text-base font-black text-base-content truncate group-hover:text-primary transition-colors capitalize" title={item.genre}>
                {item.genre}
              </h3>
              <span class="text-[10px] font-mono text-base-content/40">
                {formatTotalDuration(item.totalDuration)}
              </span>
            </div>

          </div>
        {/each}
      </div>

      {#if displayLimit < filteredGenres.length}
        <div class="text-center py-6 text-xs font-mono text-base-content/40">
          Memuat genre lainnya... ({visibleGenres.length} / {filteredGenres.length})
        </div>
      {/if}
    {/if}
  </div>

  <!-- MODAL DETAIL GENRE OVERLAY -->
  {#if selectedGenre}
    <div 
      class="absolute inset-0 bg-black/75 backdrop-blur-md z-50 flex items-center justify-center p-4 md:p-6 animate-in fade-in duration-200"
      onclick={(e) => { if (e.target === e.currentTarget) selectedGenreKey = null; }}
      onkeydown={(e) => { if (e.key === 'Escape') selectedGenreKey = null; }}
      role="button"
      tabindex="0"
    >
      <div 
        class="bg-base-200 border border-base-100 rounded-3xl w-full max-w-2xl flex flex-col overflow-hidden shadow-2xl relative cursor-default"
      >
        <!-- CLOSE BUTTON -->
        <button 
          class="btn btn-sm btn-circle btn-ghost absolute top-3 right-3 z-10 text-base-content/60 hover:text-base-content hover:bg-base-300"
          onclick={() => selectedGenreKey = null}
        >
          <X class="w-4 h-4" />
        </button>

        <!-- HEADER DETAIL GENRE -->
        <div class="p-5 bg-linear-to-b from-base-100/60 to-base-200 border-b border-base-100 flex flex-col sm:flex-row gap-5 items-center sm:items-start shrink-0">
          <div class="w-24 h-24 md:w-28 md:h-28 rounded-2xl bg-linear-to-br from-primary/20 to-primary/5 border border-primary/20 shadow-xl overflow-hidden shrink-0 flex flex-col items-center justify-center gap-1">
            <Tag class="w-8 h-8 text-primary" />
            <span class="text-[9px] font-mono font-bold text-primary/80 uppercase">GENRE</span>
          </div>

          <div class="flex-1 flex flex-col gap-1.5 text-center sm:text-left truncate w-full">
            <span class="badge badge-xs badge-primary font-bold self-center sm:self-start">GENRE CATEGORY</span>
            <h2 class="text-xl md:text-2xl font-black text-base-content truncate capitalize" title={selectedGenre.genre}>
              {selectedGenre.genre}
            </h2>

            <div class="text-[11px] font-mono text-base-content/50 flex items-center justify-center sm:justify-start gap-2 mt-0.5">
              <span>{selectedGenre.songs.length} Tracks</span>
              <span>•</span>
              <span>{formatTotalDuration(selectedGenre.totalDuration)}</span>
            </div>

            <!-- ACTION BUTTONS -->
            <div class="flex items-center justify-center sm:justify-start gap-2.5 mt-2">
              <button 
                class="btn btn-xs btn-primary gap-1.5 font-bold shadow-md shadow-primary/20 rounded-lg px-3"
                onclick={() => selectedGenre && playGenre(selectedGenre)}
              >
                <Play class="w-3.5 h-3.5 fill-current" /> Play All
              </button>
              <button 
                class="btn btn-xs btn-outline gap-1.5 font-medium rounded-lg px-3"
                onclick={() => selectedGenre && shufflePlayGenre(selectedGenre)}
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

          <!-- MAX HEIGHT RESTRICTED TO FIT ~5 SONGS -->
          <div class="max-h-[240px] overflow-y-auto scroll-smooth pr-1 flex flex-col gap-1">
            {#each selectedGenre.songs as song, idx}
              {@const isCurrent = player.currentSong?.id === song.id}
              <button 
                type="button"
                class="grid grid-cols-12 gap-2 px-3 py-2 items-center rounded-xl transition-all cursor-pointer group hover:bg-base-300/80 w-full text-left {isCurrent ? 'bg-primary/10 border-l-4 border-l-primary' : ''}"
                onclick={() => selectedGenre && playGenreTrack(selectedGenre, song)}
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
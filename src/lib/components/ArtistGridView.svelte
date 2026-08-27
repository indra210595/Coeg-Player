<script lang="ts">
  import { player, type Song } from '$lib/stores/player.svelte';
  import { Play, User, Search, Shuffle, X, Volume2, Mic2 } from 'lucide-svelte';
  import { untrack } from 'svelte';

  let searchQuery = $state('');
  let displayLimit = $state(40);
  let scrollContainer = $state<HTMLDivElement | null>(null);
  
  let selectedArtistKey = $state<string | null>(null);

  interface ArtistGroup {
    key: string;
    artist: string;
    songs: Song[];
    totalDuration: number;
  }

  // Daftar Band Legendaris yang Mengandung Koma / Ampersand biar GAK TERPOTONG
  const KNOWN_BAND_EXCEPTIONS = [
    'AC/DC',
    'earth, wind & fire',
    'earth, wind and fire',
    'crosby, stills, nash & young',
    'crosby, stills & nash',
    'tyler, the creator',
    'emerson, lake & palmer',
    'blood, sweat & tears',
    'sly & the family stone',
    'kool & the gang',
    'florence + the machine',
    'maroon 5',
    'duran duran'
  ];

  // Smart Multi-Artist Tokenizer
  function extractArtists(rawArtistStr: string): string[] {
    if (!rawArtistStr || rawArtistStr.trim() === '') return ['Unknown Artist'];
    
    let str = rawArtistStr.trim();
    
    // Hapus karakter aneh di ujung (contoh: "Avicii -" -> "Avicii")
    str = str.replace(/[\s\-\s]+$/, '').trim();
    if (!str) return ['Unknown Artist'];

    const foundExceptions: string[] = [];
    let tempStr = str;

    // 1. Amankan nama band dari daftar pengecualian
    KNOWN_BAND_EXCEPTIONS.forEach((band, idx) => {
      const regex = new RegExp(band.replace(/[.*+?^${}()|[\]\\]/g, '\\$&'), 'gi');
      if (regex.test(tempStr)) {
        const placeholder = `__BAND_EXCEPT_${idx}__`;
        // Format casing nama band dengan rapi
        const formattedBandName = band.replace(/\b\w/g, (l) => l.toUpperCase()).replace(/&/g, '&');
        foundExceptions[idx] = formattedBandName;
        tempStr = tempStr.replace(regex, placeholder);
      }
    });

    // 2. Split sisa string berdasarkan pemisah umum (feat, ft, ;, /, \, koma, &, ' x ')
    const parts = tempStr.split(/\s*(?:,|;|\\|\/|&|feat\.|ft\.|featuring|\bx\b)\s*/i);
    const result: string[] = [];

    for (const part of parts) {
      let cleaned = part.trim();
      if (!cleaned) continue;

      // Restore nama band dari placeholder
      if (cleaned.startsWith('__BAND_EXCEPT_') && cleaned.endsWith('__')) {
        const idx = parseInt(cleaned.replace('__BAND_EXCEPT_', '').replace('__', ''), 10);
        if (foundExceptions[idx]) {
          result.push(foundExceptions[idx]);
        }
      } else {
        // Hapus strip sisa di awal/akhir kata
        cleaned = cleaned.replace(/^[\s\-]+|[\s\-]+$/g, '').trim();
        if (cleaned.length > 0 && cleaned.toLowerCase() !== 'unknown artist') {
          result.push(cleaned);
        }
      }
    }

    return result.length > 0 ? Array.from(new Set(result)) : ['Unknown Artist'];
  }

  // Grouping 2.000+ lagu berdasarkan Artis Individu
  let artistGroups = $derived.by(() => {
    const map = new Map<string, ArtistGroup>();

    for (const song of player.songs) {
      const rawArtist = song.artist?.trim() || 'Unknown Artist';
      const parsedArtists = extractArtists(rawArtist);

      for (const artistName of parsedArtists) {
        const key = artistName.toLowerCase();

        if (!map.has(key)) {
          map.set(key, {
            key,
            artist: artistName,
            songs: [],
            totalDuration: 0
          });
        }

        const item = map.get(key)!;
        if (!item.songs.some((s) => s.id === song.id)) {
          item.songs.push(song);
          item.totalDuration += song.duration || 0;
        }
      }
    }

    // URUTKAN ALFABETIS (A-Z)
    return Array.from(map.values()).sort((a, b) => 
      a.artist.localeCompare(b.artist, undefined, { sensitivity: 'base', numeric: true })
    );
  });

  let selectedArtist = $derived.by(() => {
    if (!selectedArtistKey) return null;
    return artistGroups.find((a) => a.key === selectedArtistKey) || null;
  });

  let filteredArtists = $derived.by(() => {
    if (!searchQuery.trim()) return artistGroups;
    const q = searchQuery.toLowerCase().trim();
    return artistGroups.filter((a) => a.artist.toLowerCase().includes(q));
  });

  $effect(() => {
    searchQuery;
    untrack(() => {
      displayLimit = 40;
    });
  });

  let visibleArtists = $derived(filteredArtists.slice(0, displayLimit));

  function handleScroll() {
    if (!scrollContainer) return;
    const { scrollTop, scrollHeight, clientHeight } = scrollContainer;
    
    if (scrollTop + clientHeight >= scrollHeight - 300) {
      if (displayLimit < filteredArtists.length) {
        displayLimit += 40;
      }
    }
  }

  function playArtist(group: ArtistGroup) {
    if (group.songs.length > 0) {
      player.play(group.songs[0], group.songs);
    }
  }

  function playArtistTrack(group: ArtistGroup, song: Song) {
    player.play(song, group.songs);
  }

  function shufflePlayArtist(group: ArtistGroup) {
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
        placeholder="Cari artis..."
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
      {filteredArtists.length.toLocaleString()} ARTISTS
    </span>
  </div>

  <!-- GRID CONTAINER ARTIST CARDS -->
  <div 
    bind:this={scrollContainer}
    onscroll={handleScroll}
    class="flex-1 overflow-y-auto p-4 scroll-smooth"
  >
    {#if filteredArtists.length === 0}
      <div class="text-center text-base-content/40 my-20 flex flex-col items-center gap-3 select-none">
        <User class="w-12 h-12 opacity-20" />
        <p class="text-sm font-semibold">Artis tidak ditemukan</p>
      </div>
    {:else}
      <div class="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 xl:grid-cols-6 gap-4">
        {#each visibleArtists as item}
          <div 
            class="group relative bg-base-200/50 hover:bg-base-200 border border-base-100/60 hover:border-primary/40 rounded-2xl p-4 flex flex-col items-center text-center gap-3 transition-all duration-300 hover:-translate-y-1 hover:shadow-xl hover:shadow-primary/10 cursor-pointer w-full"
            onclick={() => selectedArtistKey = item.key}
            onkeydown={(e) => { if (e.key === 'Enter' || e.key === ' ') selectedArtistKey = item.key; }}
            role="button"
            tabindex="0"
          >
            <!-- CIRCULAR ARTIST AVATAR ICON -->
            <div class="w-20 h-20 rounded-full bg-linear-to-br from-primary/20 to-base-100 border border-primary/30 flex items-center justify-center relative shadow-md group-hover:scale-105 transition-transform duration-300">
              <Mic2 class="w-8 h-8 text-primary/80 group-hover:text-primary transition-colors" />

              <!-- HOVER QUICK PLAY BUTTON -->
              <div class="absolute inset-0 bg-black/40 rounded-full opacity-0 group-hover:opacity-100 transition-opacity flex items-center justify-center">
                <button 
                  type="button"
                  class="btn btn-circle btn-xs btn-primary shadow-lg shadow-primary/40"
                  onclick={(e) => {
                    e.stopPropagation();
                    playArtist(item);
                  }}
                  title="Putar Artis Instan"
                >
                  <Play class="w-3.5 h-3.5 fill-current ml-0.5" />
                </button>
              </div>
            </div>

            <!-- METADATA ARTIST -->
            <div class="flex flex-col truncate w-full">
              <span class="text-xs font-bold text-base-content truncate group-hover:text-primary transition-colors" title={item.artist}>
                {item.artist}
              </span>
              <span class="text-[9px] font-mono text-primary/80 mt-1">
                {item.songs.length} {item.songs.length > 1 ? 'TRACKS' : 'TRACK'}
              </span>
            </div>

          </div>
        {/each}
      </div>

      {#if displayLimit < filteredArtists.length}
        <div class="text-center py-6 text-xs font-mono text-base-content/40">
          Memuat artis lainnya... ({visibleArtists.length} / {filteredArtists.length})
        </div>
      {/if}
    {/if}
  </div>

  <!-- MODAL DETAIL ARTIST OVERLAY -->
  {#if selectedArtist}
    <div 
      class="absolute inset-0 bg-black/75 backdrop-blur-md z-50 flex items-center justify-center p-4 md:p-6 animate-in fade-in duration-200"
      onclick={(e) => { if (e.target === e.currentTarget) selectedArtistKey = null; }}
      onkeydown={(e) => { if (e.key === 'Escape') selectedArtistKey = null; }}
      role="button"
      tabindex="0"
    >
      <div 
        class="bg-base-200 border border-base-100 rounded-3xl w-full max-w-2xl flex flex-col overflow-hidden shadow-2xl relative cursor-default"
      >
        <!-- CLOSE BUTTON -->
        <button 
          class="btn btn-sm btn-circle btn-ghost absolute top-3 right-3 z-10 text-base-content/60 hover:text-base-content hover:bg-base-300"
          onclick={() => selectedArtistKey = null}
        >
          <X class="w-4 h-4" />
        </button>

        <!-- HEADER DETAIL ARTIST -->
        <div class="p-5 bg-linear-to-b from-base-100/60 to-base-200 border-b border-base-100 flex flex-col sm:flex-row gap-5 items-center sm:items-start shrink-0">
          <div class="w-24 h-24 md:w-28 md:h-28 rounded-full bg-linear-to-br from-primary/30 to-base-100 border border-primary/30 shadow-xl flex items-center justify-center shrink-0">
            <Mic2 class="w-10 h-10 text-primary" />
          </div>

          <div class="flex-1 flex flex-col gap-1.5 text-center sm:text-left truncate w-full">
            <span class="badge badge-xs badge-primary font-bold self-center sm:self-start">ARTIST PROFILE</span>
            <h2 class="text-xl md:text-2xl font-black text-base-content truncate" title={selectedArtist.artist}>
              {selectedArtist.artist}
            </h2>

            <div class="text-[11px] font-mono text-base-content/50 flex items-center justify-center sm:justify-start gap-2 mt-0.5">
              <span>{selectedArtist.songs.length} Tracks</span>
              <span>•</span>
              <span>{formatTotalDuration(selectedArtist.totalDuration)}</span>
            </div>

            <!-- ACTION BUTTONS -->
            <div class="flex items-center justify-center sm:justify-start gap-2.5 mt-2">
              <button 
                class="btn btn-xs btn-primary gap-1.5 font-bold shadow-md shadow-primary/20 rounded-lg px-3"
                onclick={() => selectedArtist && playArtist(selectedArtist)}
              >
                <Play class="w-3.5 h-3.5 fill-current" /> Play All
              </button>
              <button 
                class="btn btn-xs btn-outline gap-1.5 font-medium rounded-lg px-3"
                onclick={() => selectedArtist && shufflePlayArtist(selectedArtist)}
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
            <div class="col-span-8 md:col-span-9">Title / Album</div>
            <div class="col-span-3 md:col-span-2 text-right">Duration</div>
          </div>

          <div class="max-h-[240px] overflow-y-auto scroll-smooth pr-1 flex flex-col gap-1">
            {#each selectedArtist.songs as song, idx}
              {@const isCurrent = player.currentSong?.id === song.id}
              <button 
                type="button"
                class="grid grid-cols-12 gap-2 px-3 py-2 items-center rounded-xl transition-all cursor-pointer group hover:bg-base-300/80 w-full text-left {isCurrent ? 'bg-primary/10 border-l-4 border-l-primary' : ''}"
                onclick={() => selectedArtist && playArtistTrack(selectedArtist, song)}
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

                <!-- TITLE & ALBUM SUBTITLE -->
                <div class="col-span-8 md:col-span-9 flex items-center gap-2 truncate pr-2">
                  <div class="flex flex-col truncate">
                    <span class="text-xs font-semibold text-base-content truncate group-hover:text-primary transition-colors {isCurrent ? 'text-primary font-bold' : ''}">
                      {song.title}
                    </span>
                    <span class="text-[10px] text-base-content/50 truncate">
                      {song.album || 'Unknown Album'}
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
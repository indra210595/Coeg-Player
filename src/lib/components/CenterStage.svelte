<script lang="ts">
    import { onMount, onDestroy } from 'svelte';
    import { player } from '$lib/stores/player.svelte';
    import { untrack } from 'svelte';
    import { viewStore } from '$lib/stores/view.svelte';
    import SongList from '$lib/components/SongList.svelte';
    import PlayerBar from '$lib/components/PlayerBar.svelte';
    import AlbumThumb from '$lib/components/AlbumThumb.svelte';
    import FavoritesView from '$lib/components/FavoritesView.svelte';
    import ArtistGridView from '$lib/components/ArtistGridView.svelte';
    import AlbumGridView from '$lib/components/AlbumGridView.svelte';
    import GenreGridView from '$lib/components/GenreGridView.svelte';
    import PlaylistGridView from '$lib/components/PlaylistGridView.svelte';
    import { Disc3, ListFilter, AlignLeft, Music2, Heart, Disc, ListMusic, Mic2, Radio } from 'lucide-svelte';

    let activeTab = $state<'now-playing' | 'library'>('library');

    // Labels dinamis untuk tombol header tab sesuai menu sidebar yang aktif
    let currentViewLabel = $derived.by(() => {
        switch (viewStore.current) {
            case 'favorites': return 'Favorites';
            case 'artists': return 'Artists';
            case 'albums': return 'Albums';
            case 'genres': return 'Genres';
            case 'playlists': return 'Playlists';
            default: return 'Library List';
        }
    });

    // 1. REF & LOGIC LIVE FFT SPECTRUM VISUALIZER
    let barContainer = $state<HTMLDivElement | null>(null);
    let animFrameId: number;
    
    const getInitialSensitivity = (): number => {
      if (typeof window === 'undefined') return 0.9;
      const saved = localStorage.getItem('coeg_fft_sensitivity');
      return saved !== null ? parseFloat(saved) : 0.9;
    };

    let sensitivity = $state(getInitialSensitivity());

    function renderFftSpectrum() {
        if (player.analyser && barContainer && player.isPlaying) {
            const bufferLength = player.analyser.frequencyBinCount; // 32 bins
            const dataArray = new Uint8Array(bufferLength);
            player.analyser.getByteFrequencyData(dataArray);

            const bars = barContainer.children;
            for (let i = 0; i < bufferLength && i < bars.length; i++) {
                const bar = bars[i] as HTMLElement;
                const val = dataArray[i];

                const freqBoost = 1 + (i / bufferLength) * 0.95;
                const boostedVal = val * sensitivity * freqBoost;
                const heightPercent = Math.min(100, Math.max(8, (boostedVal / 255) * 100));
                
                bar.style.height = `${heightPercent}%`;
            }
        } else if (barContainer && !player.isPlaying) {
            const bars = barContainer.children;
            for (let i = 0; i < bars.length; i++) {
                (bars[i] as HTMLElement).style.height = '8%';
            }
        }

        animFrameId = requestAnimationFrame(renderFftSpectrum);
    }

    onMount(() => {
        animFrameId = requestAnimationFrame(renderFftSpectrum);
    });

    onDestroy(() => {
        if (animFrameId) cancelAnimationFrame(animFrameId);
    });

    // 2. PARSER & SYNCED LYRICS LOGIC (.LRC FORMAT)
    interface LyricLine {
        time: number;
        text: string;
    }

    let parsedLyrics = $derived.by(() => {
        const raw = player.currentSong?.lyrics;
        if (!raw) return [];

        const lines = raw.split('\n');
        const result: LyricLine[] = [];
        const timeRegex = /\[(\d{2}):(\d{2})\.(\d{2,3})\]/;

        for (const line of lines) {
            const match = timeRegex.exec(line);
            if (match) {
                const min = parseInt(match[1], 10);
                const sec = parseInt(match[2], 10);
                const ms = parseInt(match[3].padEnd(3, '0'), 10);
                const time = min * 60 + sec + ms / 1000;
                const text = line.replace(/\[.*?\]/g, '').trim();
                if (text) result.push({ time, text });
            } else {
                const text = line.trim();
                if (text && !text.startsWith('[')) {
                    result.push({ time: -1, text });
                }
            }
        }
        return result.sort((a, b) => a.time - b.time);
    });

    let activeIndex = $derived.by(() => {
        if (parsedLyrics.length === 0) return -1;
        const curTime = player.currentTime;
        
        for (let i = parsedLyrics.length - 1; i >= 0; i--) {
            if (parsedLyrics[i].time !== -1 && parsedLyrics[i].time <= curTime) {
                return i;
            }
        }
        return 0;
    });

    let lyricsContainer = $state<HTMLDivElement | null>(null);

    $effect(() => {
        if (activeIndex !== -1 && lyricsContainer) {
            const activeEl = lyricsContainer.children[activeIndex] as HTMLElement;
            if (activeEl) {
                activeEl.scrollIntoView({ behavior: 'smooth', block: 'center' });
            }
        }
    });
    $effect(() => {
      const currentView = viewStore.current;
      untrack(() => {
        activeTab = 'library';
      });
    });
    $effect(() => {
      if (typeof window !== 'undefined') {
        localStorage.setItem('coeg_fft_sensitivity', sensitivity.toString());
      }
    });
</script>

<main class="flex-1 flex flex-col bg-base-300 overflow-hidden relative">
  <!-- VIEW SWITCHER HEADER -->
  <div class="p-3 border-b border-base-100 flex items-center justify-between bg-base-200/50 flex-shrink-0 select-none">
    <div class="join bg-base-100 p-0.5 rounded-lg">
      <button 
        class="join-item btn btn-xs border-none {activeTab === 'now-playing' ? 'btn-primary' : 'btn-ghost text-base-content/60'}"
        onclick={() => activeTab = 'now-playing'}
      >
        <Disc3 class="w-3.5 h-3.5" /> Now Playing
      </button>
      <button 
        class="join-item btn btn-xs border-none {activeTab === 'library' ? 'btn-primary' : 'btn-ghost'}"
        onclick={() => activeTab = 'library'}
      >
        <ListFilter class="w-3.5 h-3.5" /> {currentViewLabel}
      </button>
    </div>
  </div>

  <!-- TAB CONTENT CONTAINER -->
  <div class="flex-1 flex flex-col min-h-0 overflow-hidden relative w-full">
    
    <!-- 1. NOW PLAYING VIEW (SPECTUM + LYRICS) -->
    <div class="h-full w-full overflow-y-auto p-4 flex flex-col gap-6 {activeTab === 'now-playing' ? 'flex' : 'hidden'}">
      
      <!-- TOP ROW: COVER ART & METADATA -->
      <div class="flex flex-col lg:flex-row gap-6 items-center lg:items-start flex-shrink-0">
        <div class="w-50 h-50 md:w-64 md:h-64 rounded-2xl bg-base-100 border border-base-100 shadow-2xl flex items-center justify-center flex-shrink-0 relative overflow-hidden group">
            {#if player.currentSong?.cover_path}
                <AlbumThumb coverPath={player.currentSong.cover_path} sizeClass="w-full h-full" />
            {:else}
                <Disc3 class="w-24 h-24 text-primary/30 animate-spin-slow" />
            {/if}
            <div class="absolute inset-0 bg-gradient-to-t from-black/60 via-transparent to-transparent opacity-40"></div>
        </div>

        <div class="flex-1 w-full flex flex-col gap-4">
          <div>
            <div class="flex gap-2 mb-2">
              <span class="badge badge-xs badge-primary font-bold">NOW PLAYING</span>
              <span class="badge badge-xs badge-outline">{player.currentSong?.genre || 'MUSIC'}</span>
            </div>
            <h2 class="text-2xl font-bold text-base-content">{player.currentSong?.title || 'Pilih Lagu'}</h2>
            <p class="text-sm text-primary font-medium mt-1">
              {player.currentSong?.artist || 'Unknown Artist'} — 
              <span class="text-base-content/50">{player.currentSong?.album || 'Unknown Album'}</span>
            </p>

            {#if player.currentSong}
              <div class="flex items-center gap-2 mt-3 flex-wrap">
                <span class="badge badge-sm font-mono font-bold {player.currentSong.is_lossless ? 'badge-success text-black' : 'badge-neutral text-base-content/70'}">
                  {player.currentSong.is_lossless ? 'LOSSLESS' : 'LOSSY'}
                </span>
                <span class="badge badge-sm badge-outline font-mono font-semibold text-primary">
                  {player.currentSong.format || 'MP3'}
                </span>
                <span class="text-xs font-mono text-base-content/60 bg-base-200 px-2 py-0.5 rounded-md border border-base-100">
                  {#if player.currentSong.is_lossless}
                    {player.currentSong.bit_depth || 16}bit / {(player.currentSong.sample_rate ? player.currentSong.sample_rate / 1000 : 44.1).toFixed(1)}kHz
                  {:else}
                    {player.currentSong.bitrate || 320} kbps • {(player.currentSong.sample_rate ? player.currentSong.sample_rate / 1000 : 44.1).toFixed(1)}kHz
                  {/if}
                </span>
              </div>
            {/if}
          </div>

          <!-- REALTIME LIVE FFT SPECTRUM -->
          <div class="p-4 bg-base-200/60 rounded-2xl border border-base-100 flex flex-col gap-2">
            <div class="flex items-center justify-between">
              <span class="text-[10px] font-mono font-bold text-primary uppercase tracking-wider flex items-center gap-1.5">
                <span class="w-1.5 h-1.5 rounded-full {player.isPlaying ? 'bg-success animate-pulse' : 'bg-base-content/30'}"></span>
                FFT (32 Bins)
              </span>

              <div class="flex items-center gap-2 bg-base-100/60 px-2 py-1 rounded-lg border border-base-100">
                <span class="text-[9px] font-mono text-base-content/60 font-semibold">
                  SENS: {sensitivity.toFixed(1)}x
                </span>
                <input 
                  type="range" 
                  min="0.5" 
                  max="3.0" 
                  step="0.1" 
                  bind:value={sensitivity} 
                  class="range range-xs range-primary w-20 h-1"
                  title="Atur sensitivitas ayunan visualizer"
                />
              </div>
            </div>

            <div bind:this={barContainer} class="h-20 flex items-end justify-between gap-[3px] pt-2 px-1">
              {#each Array(32) as _, i}
                <div 
                  class="flex-1 bg-gradient-to-t from-primary via-accent to-secondary rounded-t-sm transition-[height] duration-75 ease-out"
                  style="height: 8%"
                ></div>
              {/each}
            </div>
          </div>
        </div>
      </div>

      <!-- BOTTOM ROW: SYNCED LYRICS KARAOKE DISPLAY -->
      <div class="p-4 bg-base-200/60 rounded-2xl border border-base-100 flex flex-col gap-3 flex-1 min-h-[240px]">
        <div class="flex items-center justify-between border-b border-base-100/40 pb-2">
          <span class="text-[10px] font-mono font-bold text-primary uppercase tracking-wider flex items-center gap-1.5">
            <AlignLeft class="w-3.5 h-3.5" /> Lyrics
          </span>
        </div>

        <div bind:this={lyricsContainer} class="flex-1 overflow-y-auto max-h-64 flex flex-col items-center gap-4 py-12 px-2 scroll-smooth select-none">
          {#if parsedLyrics.length > 0}
            {#each parsedLyrics as line, i}
              {@const isActive = i === activeIndex}
              <button 
                type="button"
                class="text-center transition-all duration-300 hover:scale-105 cursor-pointer {isActive ? 'text-primary font-bold text-lg md:text-xl drop-shadow-[0_0_12px_rgba(168,85,247,0.5)]' : 'text-base-content/30 text-sm md:text-base font-medium'}"
                onclick={() => line.time !== -1 && player.seek(line.time)}
              >
                {line.text}
              </button>
            {/each}
          {:else}
            <div class="flex flex-col items-center justify-center my-auto text-base-content/30 gap-2 py-8">
              <Music2 class="w-8 h-8 animate-bounce" />
              <span class="text-xs font-mono">Belum ada lirik untuk lagu ini</span>
            </div>
          {/if}
        </div>
      </div>

    </div>

    <!-- 2. DYNAMIC CONTENT VIEW (BERDASARKAN SIDEBAR MENU) -->
    <div class="flex-1 flex flex-col min-h-0 overflow-hidden w-full {activeTab === 'library' ? '' : 'hidden'}">
      {#if viewStore.current === 'library'}
        <SongList />

      {:else if viewStore.current === 'favorites'}
        <FavoritesView />

      {:else if viewStore.current === 'artists'}
        <ArtistGridView />

      {:else if viewStore.current === 'albums'}
        <AlbumGridView />

      {:else if viewStore.current === 'genres'}
        <GenreGridView />

      {:else if viewStore.current === 'playlists'}
        <PlaylistGridView />
      {/if}
    </div>

  </div>

  <!-- PLAYERBAR EMBEDDED -->
  <PlayerBar />
</main>
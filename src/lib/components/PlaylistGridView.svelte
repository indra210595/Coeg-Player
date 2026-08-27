<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { player, type Song } from '$lib/stores/player.svelte';
  import AlbumThumb from '$lib/components/AlbumThumb.svelte';
  import { Play, ListMusic, Plus, Search, Trash2, X, Volume2, Music, Shuffle } from 'lucide-svelte';

  interface Playlist {
    id: number;
    name: string;
    created_at: string;
    song_count: number;
  }

  let playlists = $state<Playlist[]>([]);
  let searchQuery = $state('');
  let newPlaylistName = $state('');
  let isCreateModalOpen = $state(false);

  let selectedPlaylist = $state<Playlist | null>(null);
  let playlistSongs = $state<Song[]>([]);

  async function loadPlaylists() {
    try {
      playlists = await invoke<Playlist[]>('get_playlists');
    } catch (err) {
      console.error('Gagal ambil playlists:', err);
    }
  }

  $effect(() => {
    loadPlaylists();
  });

  async function handleCreatePlaylist() {
    if (!newPlaylistName.trim()) return;
    try {
      await invoke('create_playlist', { name: newPlaylistName });
      newPlaylistName = '';
      isCreateModalOpen = false;
      await loadPlaylists();
    } catch (err) {
      console.error('Gagal buat playlist:', err);
    }
  }

  async function handleDeletePlaylist(id: number) {
    try {
      await invoke('delete_playlist', { playlistId: id });
      if (selectedPlaylist?.id === id) selectedPlaylist = null;
      await loadPlaylists();
    } catch (err) {
      console.error('Gagal hapus playlist:', err);
    }
  }

  async function openPlaylistDetail(pl: Playlist) {
    selectedPlaylist = pl;
    try {
      playlistSongs = await invoke<Song[]>('get_playlist_songs', { playlistId: pl.id });
    } catch (err) {
      console.error('Gagal load isi playlist:', err);
    }
  }

  async function removeSong(songId: number) {
    if (!selectedPlaylist) return;
    try {
      await invoke('remove_song_from_playlist', { playlistId: selectedPlaylist.id, songId });
      playlistSongs = playlistSongs.filter((s) => s.id !== songId);
      await loadPlaylists();
    } catch (err) {
      console.error('Gagal hapus lagu dari playlist:', err);
    }
  }

  function playPlaylist(songs: Song[]) {
    if (songs.length > 0) player.play(songs[0], songs);
  }

  function shufflePlay(songs: Song[]) {
    if (songs.length > 0) {
      if (!player.isShuffle) player.toggleShuffle();
      const rand = Math.floor(Math.random() * songs.length);
      player.play(songs[rand], songs);
    }
  }

  function formatTime(seconds: number) {
    if (!seconds || isNaN(seconds)) return '0:00';
    const mins = Math.floor(seconds / 60);
    const secs = Math.floor(seconds % 60);
    return `${mins}:${secs < 10 ? '0' : ''}${secs}`;
  }

  let filteredPlaylists = $derived.by(() => {
    if (!searchQuery.trim()) return playlists;
    const q = searchQuery.toLowerCase().trim();
    return playlists.filter((p) => p.name.toLowerCase().includes(q));
  });
</script>

<div class="h-full w-full flex flex-col bg-base-300 overflow-hidden relative">
  
  <!-- HEADER -->
  <div class="p-4 bg-base-200/80 border-b border-base-100 flex items-center justify-between gap-4 shrink-0 select-none">
    <div class="relative flex-1 max-w-md">
      <Search class="w-4 h-4 absolute left-3 top-1/2 -translate-y-1/2 text-base-content/40" />
      <input 
        type="text" 
        placeholder="Cari playlist..."
        bind:value={searchQuery}
        class="input input-sm w-full pl-9 bg-base-300 border-base-100 focus:border-primary text-xs rounded-xl"
      />
    </div>

    <button 
      class="btn btn-sm btn-primary font-bold gap-2 rounded-xl shadow-md shadow-primary/20"
      onclick={() => isCreateModalOpen = true}
    >
      <Plus class="w-4 h-4" /> Playlist Baru
    </button>
  </div>

  <!-- GRID PLAYLIST -->
  <div class="flex-1 overflow-y-auto p-4 scroll-smooth">
    {#if filteredPlaylists.length === 0}
      <div class="text-center text-base-content/40 my-20 flex flex-col items-center gap-3 select-none">
        <ListMusic class="w-12 h-12 opacity-20" />
        <p class="text-sm font-semibold">Belum ada playlist</p>
      </div>
    {:else}
      <div class="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 xl:grid-cols-6 gap-4">
        {#each filteredPlaylists as item}
          <div 
            class="group relative bg-base-200/50 hover:bg-base-200 border border-base-100/60 hover:border-primary/40 rounded-2xl p-4 flex flex-col justify-between aspect-square transition-all duration-300 hover:-translate-y-1 hover:shadow-xl hover:shadow-primary/10 cursor-pointer w-full overflow-hidden"
            onclick={() => openPlaylistDetail(item)}
            onkeydown={(e) => { if (e.key === 'Enter') openPlaylistDetail(item); }}
            role="button"
            tabindex="0"
          >
            <ListMusic class="w-20 h-20 absolute -right-4 -bottom-4 text-primary/10 group-hover:text-primary/20 transition-all rotate-12" />

            <div class="flex justify-between items-center w-full z-10">
              <span class="badge badge-xs badge-primary font-bold font-mono">
                {item.song_count} TRACKS
              </span>
              <button 
                type="button"
                class="btn btn-ghost btn-xs btn-circle text-error/60 hover:text-error opacity-0 group-hover:opacity-100 transition-opacity"
                onclick={(e) => {
                  e.stopPropagation();
                  handleDeletePlaylist(item.id);
                }}
                title="Hapus Playlist"
              >
                <Trash2 class="w-3.5 h-3.5" />
              </button>
            </div>

            <div class="z-10 mt-auto truncate">
              <h3 class="text-base font-bold text-base-content truncate group-hover:text-primary transition-colors">
                {item.name}
              </h3>
            </div>
          </div>
        {/each}
      </div>
    {/if}
  </div>

  <!-- MODAL DETAIL PLAYLIST -->
  {#if selectedPlaylist}
    <div 
      class="absolute inset-0 bg-black/75 backdrop-blur-md z-50 flex items-center justify-center p-4 md:p-6 animate-in fade-in duration-200"
      onclick={(e) => { if (e.target === e.currentTarget) selectedPlaylist = null; }}
      onkeydown={(e) => { if (e.key === 'Escape') selectedPlaylist = null; }}
      role="button"
      tabindex="0"
    >
      <div class="bg-base-200 border border-base-100 rounded-3xl w-full max-w-2xl flex flex-col overflow-hidden shadow-2xl relative cursor-default">
        <button 
          class="btn btn-sm btn-circle btn-ghost absolute top-3 right-3 z-10 text-base-content/60 hover:text-base-content"
          onclick={() => selectedPlaylist = null}
        >
          <X class="w-4 h-4" />
        </button>

        <div class="p-5 bg-linear-to-b from-base-100/60 to-base-200 border-b border-base-100 flex gap-5 items-center shrink-0">
          <div class="w-24 h-24 rounded-2xl bg-primary/10 border border-primary/20 flex flex-col items-center justify-center text-primary shrink-0">
            <ListMusic class="w-8 h-8" />
          </div>

          <div class="flex-1 truncate">
            <span class="badge badge-xs badge-primary font-bold">PLAYLIST</span>
            <h2 class="text-xl font-bold text-base-content truncate mt-1">{selectedPlaylist.name}</h2>
            <p class="text-xs text-base-content/50 font-mono mt-0.5">{playlistSongs.length} Songs</p>

            <div class="flex gap-2 mt-3">
              <button class="btn btn-xs btn-primary gap-1.5 font-bold" onclick={() => playPlaylist(playlistSongs)}>
                <Play class="w-3.5 h-3.5 fill-current" /> Play All
              </button>
              <button class="btn btn-xs btn-outline gap-1.5" onclick={() => shufflePlay(playlistSongs)}>
                <Shuffle class="w-3.5 h-3.5" /> Shuffle
              </button>
            </div>
          </div>
        </div>

        <div class="p-4 flex flex-col shrink-0">
          <div class="max-h-[240px] overflow-y-auto scroll-smooth pr-1 flex flex-col gap-1">
            {#each playlistSongs as song, idx}
              {@const isCurrent = player.currentSong?.id === song.id}
              <div 
                class="grid grid-cols-12 gap-2 px-3 py-2 items-center rounded-xl hover:bg-base-300/80 transition-all cursor-pointer group {isCurrent ? 'bg-primary/10 border-l-4 border-l-primary' : ''}"
                onclick={() => player.play(song, playlistSongs)}
                onkeydown={(e) => { if (e.key === 'Enter') player.play(song, playlistSongs); }}
                role="button"
                tabindex="0"
              >
                <div class="col-span-1 flex items-center justify-center font-mono text-xs">
                  {#if isCurrent && player.isPlaying}
                    <Volume2 class="w-4 h-4 text-primary animate-pulse" />
                  {:else}
                    <span class="text-base-content/40 group-hover:hidden">{(idx + 1).toString().padStart(2, '0')}</span>
                    <Play class="w-3.5 h-3.5 text-primary hidden group-hover:block fill-current" />
                  {/if}
                </div>

                <div class="col-span-8 flex flex-col truncate">
                  <span class="text-xs font-semibold text-base-content truncate group-hover:text-primary {isCurrent ? 'text-primary font-bold' : ''}">
                    {song.title}
                  </span>
                  <span class="text-[10px] text-base-content/50 truncate">{song.artist}</span>
                </div>

                <div class="col-span-3 flex items-center justify-end gap-2 font-mono text-xs text-base-content/50">
                  <button 
                    type="button"
                    class="btn btn-ghost btn-xs btn-circle text-error/60 hover:text-error opacity-0 group-hover:opacity-100"
                    onclick={(e) => {
                      e.stopPropagation();
                      removeSong(song.id);
                    }}
                    title="Hapus dari playlist"
                  >
                    <X class="w-3.5 h-3.5" />
                  </button>
                  <span>{formatTime(song.duration)}</span>
                </div>
              </div>
            {/each}
          </div>
        </div>
      </div>
    </div>
  {/if}

  <!-- MODAL BUAT PLAYLIST -->
  {#if isCreateModalOpen}
    <div class="absolute inset-0 bg-black/75 backdrop-blur-md z-50 flex items-center justify-center p-4">
      <div class="bg-base-200 border border-base-100 p-6 rounded-3xl w-full max-w-sm flex flex-col gap-4 shadow-2xl">
        <h3 class="text-base font-bold text-base-content">Buat Playlist Baru</h3>
        <input 
          type="text" 
          placeholder="Nama playlist..." 
          bind:value={newPlaylistName}
          class="input input-sm bg-base-300 border-base-100 focus:border-primary w-full text-xs rounded-xl"
        />
        <div class="flex justify-end gap-2">
          <button class="btn btn-xs btn-ghost" onclick={() => isCreateModalOpen = false}>Batal</button>
          <button class="btn btn-xs btn-primary font-bold" onclick={handleCreatePlaylist}>Simpan</button>
        </div>
      </div>
    </div>
  {/if}

</div>
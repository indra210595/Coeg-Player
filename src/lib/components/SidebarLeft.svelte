<script lang="ts">
    import { ask } from '@tauri-apps/plugin-dialog';
    import { onMount } from 'svelte';
    import { invoke } from '@tauri-apps/api/core';
    import { open } from '@tauri-apps/plugin-dialog';
    import { listen } from '@tauri-apps/api/event';
    import { 
        Library, ListMusic, Heart, History, 
        Folder, FolderPlus, CheckCircle2, Loader2, 
        RefreshCw, Trash2, Music, Zap, Play, Music2, Disc, Mic2, Radio
    } from 'lucide-svelte';
    import { viewStore, type ViewType } from '$lib/stores/view.svelte';
    import { player, type Song } from '$lib/stores/player.svelte';
    import AlbumThumb from '$lib/components/AlbumThumb.svelte';

    const menuItems: { id: ViewType; label: string; icon: any }[] = [
      { id: 'library', label: 'Music Library', icon: Music2 },
      { id: 'favorites', label: 'Favorites', icon: Heart },
      { id: 'artists', label: 'Artists', icon: Mic2 },
      { id: 'albums', label: 'Albums', icon: Disc },
      { id: 'genres', label: 'Genres', icon: Radio },
      { id: 'playlists', label: 'Playlists', icon: ListMusic },
    ];

    interface FolderItem {
      id: number;
      path: string;
      created_at: string;
      file_count: number;
      total_bytes: number;
      is_syncing?: boolean;
      progress?: number;
    }

  interface ScanProgressPayload {
    folder_id: number;
    current: number;
    total: number;
    percentage: number;
    status: string;
  }

  let folders = $state<FolderItem[]>([]);

  // Utility Format Bytes ke GB/MB
  function formatBytes(bytes: number) {
    if (!bytes || bytes === 0) return '0 B';
    const gb = bytes / (1024 * 1024 * 1024);
    if (gb >= 1) return `${gb.toFixed(1)} GB`;
    const mb = bytes / (1024 * 1024);
    return `${mb.toFixed(1)} MB`;
  }

  // Fetch daftar folder dari SQLite via Rust
  async function loadFolders() {
    try {
      const res = await invoke<FolderItem[]>('get_folders');
      folders = res.map((f) => {
        const existing = folders.find((old) => old.id === f.id);
        return {
          ...f,
          is_syncing: existing ? existing.is_syncing : false,
          progress: existing ? existing.progress : 100
        };
      });
    } catch (err) {
      console.error('Error fetching folders:', err);
    }
  }

  // Fetch daftar lagu dari SQLite via Rust
  async function refreshSongs() {
    try {
      player.songs = await invoke<Song[]>('get_songs');
      player.refreshQuickPicks();
    } catch (err) {
      console.error('Error fetching songs:', err);
    }
  }

  // 1. TAMBAH FOLDER BARU
  async function handleAddFolder() {
    try {
      const selected = await open({
        directory: true,
        multiple: false,
        title: 'Pilih Folder Musik'
      });

      console.log("Folder dipilih:", selected);

      if (selected) {
        const folderId = await invoke<number>('add_folder', { folderPath: selected });
        console.log("Folder ID dibuat:", folderId);
        await loadFolders();
        
        const target = folders.find((f) => f.id === folderId);
        if (target) {
          target.is_syncing = true;
          target.progress = 0;
        }
      }
    } catch (err) {
      console.error('Error adding folder:', err);
      alert('Gagal memilih folder: ' + err);
    }
  }

  // 2. SYNC / RESCAN PER FOLDER
  async function handleSyncFolder(folderId: number) {
    try {
      const target = folders.find((f) => f.id === folderId);
      if (target) {
        target.is_syncing = true;
        target.progress = 0;
      }
      await invoke('sync_folder', { folderId });
    } catch (err) {
      console.error('Error syncing folder:', err);
    }
  }

  // 3. HAPUS FOLDER DARI APP (ON DELETE CASCADE)
  async function handleDeleteFolder(folderId: number, path: string) {
    const confirmed = await ask(`Hapus folder "${path}" dari daftar sync?\nLagu di dalamnya akan dikeluarkan dari library.`, {
        title: 'Konfirmasi Hapus Folder',
        kind: 'warning'
    });

    if (confirmed) {
        try {
            await invoke('delete_folder', { folderId });
            await loadFolders();
            await refreshSongs();
        } catch (err) {
            console.error('Error deleting folder:', err);
        }
    }
  }

  // Derived Values (Kalkulasi Otomatis)
  let totalFiles = $derived(folders.reduce((acc, f) => acc + f.file_count, 0));
  let totalSizeFormatted = $derived(formatBytes(folders.reduce((acc, f) => acc + f.total_bytes, 0)));
  let isAnySyncing = $derived(folders.some((f) => f.is_syncing));

  onMount(() => {
    loadFolders();
    refreshSongs();

    // Event Listener Progress Realtime dari Rust
    const unlistenProgress = listen<ScanProgressPayload>('scan-progress', (event) => {
      const { folder_id, percentage, status } = event.payload;
      const target = folders.find((f) => f.id === folder_id);
      if (target) {
        target.progress = percentage;
        target.is_syncing = status === 'syncing';
      }
    });

    // Event Listener Selesai Scan
    const unlistenFinished = listen<number>('scan-finished', async () => {
      await loadFolders();
      await refreshSongs();
    });

    return () => {
      unlistenProgress.then((u) => u());
      unlistenFinished.then((u) => u());
    };
  });
</script>

<aside class="w-16 md:w-64 bg-base-200 border-r border-base-100 flex flex-col justify-between p-3 select-none flex-shrink-0 transition-all">
  
  <!-- TOP SECTION -->
  <div class="flex flex-col gap-4">
    <!-- BRAND REBRANDING (COEG PLAYER) -->
    <div class="flex items-center gap-2.5 px-2 py-1">
      <div class="p-2 bg-primary/10 text-primary rounded-xl">
        <Music class="w-5 h-5" />
      </div>
      <div class="hidden md:flex flex-col">
        <span class="font-bold text-sm tracking-wide text-primary">COEG PLAYER</span>
        <span class="text-[10px] text-base-content/50 tracking-widest font-mono">NEON EDITION</span>
      </div>
    </div>

    <!-- NAVIGATION MENU -->
    <nav class="flex flex-col gap-1">
      <span class="hidden md:block text-[10px] font-bold text-base-content/40 px-2 my-1 uppercase">Browse</span>
      
      {#each menuItems as item}
        {@const Icon = item.icon}
        <button
          type="button"
          class="flex items-center gap-3 px-3 py-2.5 rounded-xl text-xs font-medium transition-all cursor-pointer {viewStore.current === item.id ? 'bg-primary text-primary-content font-bold shadow-lg shadow-primary/20' : 'text-base-content/70 hover:bg-base-300 hover:text-base-content'}"
          onclick={() => viewStore.set(item.id)}
        >
          <Icon class="w-4 h-4" />
          <span>{item.label}</span>
        </button>
      {/each}
    </nav>
  </div>

  <!-- BOTTOM SECTION (QUICK PICKS + LOCAL SYNC) -->
  <div class="flex flex-col gap-3">
    
    <!-- QUICK PICKS -->
    {#if player.quickPicks.length > 0}
      <div class="hidden md:flex flex-col gap-1.5">
        <div class="flex items-center justify-between px-1">
            <span class="text-[10px] font-bold text-base-content/40 uppercase flex items-center gap-1">
                <Zap class="w-3 h-3 text-warning" /> Quick Picks
            </span>

            <button 
                type="button"
                class="btn btn-ghost btn-xs p-1 text-base-content/40 hover:text-primary rounded-lg transition-colors"
                onclick={() => player.refreshQuickPicks()}
                title="Acak Quick Picks"
            >
                <RefreshCw class="w-3 h-3" />
            </button>
        </div>

        <div class="flex flex-col gap-1">
          {#each player.quickPicks as song}
            <button 
            class="flex items-center justify-between p-1.5 rounded-xl bg-base-300/40 hover:bg-base-300 border border-base-100/50 text-left transition-all group"
            onclick={() => player.play(song)}
            >
                <div class="flex items-center gap-2.5 truncate">
                    <!-- MINI THUMBNAIL -->
                    <AlbumThumb coverPath={song.cover_path} sizeClass="w-8 h-8" />

                    <div class="truncate">
                        <div class="text-[11px] font-semibold text-base-content truncate group-hover:text-primary">{song.title}</div>
                        <div class="text-[9px] text-base-content/50 truncate">{song.artist}</div>
                    </div>
                </div>
                <Play class="w-3.5 h-3.5 text-primary opacity-0 group-hover:opacity-100 flex-shrink-0 transition-opacity mr-1" />
            </button>
          {/each}
        </div>
      </div>
    {/if}

    <!-- PANEL LOCAL SYNC MULTI-FOLDER -->
    <div class="hidden md:flex flex-col gap-3 p-3.5 bg-base-300/80 rounded-2xl border border-base-100">
      
      <!-- HEADER STATUS + ADD BUTTON -->
      <div class="flex items-center justify-between">
        <div class="flex items-center gap-1.5">
          <div class="w-2 h-2 rounded-full {isAnySyncing ? 'bg-warning animate-pulse' : 'bg-success'}"></div>
          <span class="text-[10px] font-mono font-bold {isAnySyncing ? 'text-warning' : 'text-success'}">
            {isAnySyncing ? 'SYNCING' : 'LOCAL SYNC'}
          </span>
        </div>

        <button 
          class="btn btn-ghost btn-xs p-1 text-primary hover:bg-primary/10 rounded-lg"
          onclick={handleAddFolder}
          title="Tambah Folder Musik"
        >
          <FolderPlus class="w-4 h-4" />
        </button>
      </div>

      <!-- LIST FOLDER -->
      <div class="flex flex-col gap-2.5 max-h-40 overflow-y-auto pr-1">
        {#if folders.length === 0}
          <div class="text-center py-4 text-base-content/40 text-xs">
            Belum ada folder.<br />
            <button class="btn btn-link btn-xs text-primary p-0 h-auto" onclick={handleAddFolder}>
              + Tambah Folder
            </button>
          </div>
        {:else}
          {#each folders as folder (folder.id)}
            <div class="flex items-center justify-between gap-2 group">
              
              <!-- LEFT: ICON & PATH -->
              <div class="flex items-center gap-2 truncate flex-1">
                <Folder class="w-4 h-4 text-warning flex-shrink-0" />
                <div class="truncate">
                  <div class="text-xs font-semibold text-base-content/90 truncate" title={folder.path}>
                    {folder.path}
                  </div>
                  
                  {#if folder.is_syncing}
                    <div class="text-[10px] font-mono text-warning font-semibold">
                      Syncing... {folder.progress}%
                    </div>
                  {:else}
                    <div class="text-[10px] font-mono text-base-content/40">
                      {folder.file_count.toLocaleString()} files
                    </div>
                  {/if}
                </div>
              </div>

              <!-- RIGHT: ACTION BUTTONS (SYNC & DELETE) -->
              <div class="flex items-center gap-1 flex-shrink-0">
                {#if folder.is_syncing}
                  <Loader2 class="w-3.5 h-3.5 text-warning animate-spin" />
                {:else}
                  <button 
                    class="btn btn-ghost btn-xs p-1 text-base-content/40 hover:text-primary opacity-0 group-hover:opacity-100 transition-opacity"
                    onclick={() => handleSyncFolder(folder.id)}
                    title="Rescan / Sync Folder"
                  >
                    <RefreshCw class="w-3 h-3" />
                  </button>

                  <button 
                    class="btn btn-ghost btn-xs p-1 text-base-content/40 hover:text-error opacity-0 group-hover:opacity-100 transition-opacity"
                    onclick={() => handleDeleteFolder(folder.id, folder.path)}
                    title="Hapus Folder"
                  >
                    <Trash2 class="w-3 h-3" />
                  </button>
                {/if}
              </div>

            </div>
          {/each}
        {/if}
      </div>

      <div class="divider my-0 opacity-20"></div>

      <!-- FOOTER TOTALS -->
      <div class="flex items-center justify-between text-[11px] font-mono text-base-content/50">
        <span>Total: {totalFiles.toLocaleString()} files</span>
        <span class="font-bold text-base-content/80">{totalSizeFormatted}</span>
      </div>

    </div>

  </div>

</aside>
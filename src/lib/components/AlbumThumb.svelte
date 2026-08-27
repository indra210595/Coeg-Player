<script lang="ts">
    import { convertFileSrc } from '@tauri-apps/api/core';
    import { Music } from 'lucide-svelte';

    let { coverPath = null, sizeClass = 'w-9 h-9' } = $props<{
        coverPath?: string | null;
        sizeClass?: string;
    }>();

    let coverUrl = $derived(coverPath ? convertFileSrc(coverPath) : null);
</script>

<div class="{sizeClass} rounded-lg bg-base-300/80 border border-base-100/60 flex items-center justify-center shrink-0 overflow-hidden relative">
    {#if coverUrl}
        <img 
            src={coverUrl} 
            alt="Cover" 
            class="w-full h-full object-cover" 
            loading="lazy"
            decoding="async"
        />
    {:else}
        <Music class="w-4 h-4 text-base-content/30" />
    {/if}
</div>
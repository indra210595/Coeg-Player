<script lang="ts">
  import { RefreshCw, Cpu, Music, Sparkles } from 'lucide-svelte';
  import { player } from '$lib/stores/player.svelte';

  let { isOpen = $bindable(true) } = $props<{ isOpen: boolean }>();

  let averageMatch = $derived.by(() => {
    if (player.aiQueue.length === 0) return 95;
    const sum = player.aiQueue.reduce((acc, item) => acc + item.matchScore, 0);
    return Math.round(sum / player.aiQueue.length);
  });

  function getScoreTheme(score: number) {
    if (score >= 90) {
      return {
        text: 'text-emerald-400',
        bar: 'bg-emerald-500',
        border: 'border-emerald-500/20',
        leftBorder: 'border-l-emerald-500'
      };
    }
    if (score >= 80) {
      return {
        text: 'text-amber-500',
        bar: 'bg-amber-500',
        border: 'border-amber-500/20',
        leftBorder: 'border-l-amber-500'
      };
    }
    return {
      text: 'text-rose-400',
      bar: 'bg-rose-500',
      border: 'border-rose-500/20',
      leftBorder: 'border-l-rose-500'
    };
  }
</script>

{#if isOpen}
  <aside class="w-80 bg-base-200 border-l border-base-100 flex flex-col justify-between p-4 flex-shrink-0 select-none hidden xl:flex h-full overflow-hidden">
    
    <!-- TOP SECTION: HEADER + STATS CARDS -->
    <div class="flex flex-col gap-3 flex-shrink-0">
      <div class="flex items-center justify-between">
        <div class="flex items-center gap-2.5">
          <div class="p-2 rounded-xl bg-emerald-500/10 border border-emerald-500/20 text-emerald-400 shadow-[0_0_12px_rgba(16,185,129,0.2)]">
            <Cpu class="w-5 h-5 animate-pulse" />
          </div>
          <div class="flex flex-col">
            <span class="font-bold text-sm tracking-wide text-base-content">Recommendation</span>
            <span class="text-[9px] font-mono font-bold text-emerald-400 tracking-wider">PREDICTIVE QUEUE</span>
          </div>
        </div>
      </div>

      <div class="grid grid-cols-3 gap-2 my-1">
        <div class="bg-base-300/60 rounded-xl p-2 border border-base-100/60 flex flex-col items-center justify-center text-center">
          <span class="text-base font-mono font-bold text-amber-500">{averageMatch}%</span>
          <span class="text-[9px] font-mono text-base-content/40">Accuracy</span>
        </div>
        <div class="bg-base-300/60 rounded-xl p-2 border border-base-100/60 flex flex-col items-center justify-center text-center">
          <span class="text-base font-mono font-bold text-emerald-400">{player.songs.length * 2 || 847}</span>
          <span class="text-[9px] font-mono text-base-content/40">Patterns</span>
        </div>
        <div class="bg-base-300/60 rounded-xl p-2 border border-base-100/60 flex flex-col items-center justify-center text-center">
          <span class="text-base font-mono font-bold text-base-content/80">{player.aiQueue.length}</span>
          <span class="text-[9px] font-mono text-base-content/40">In Queue</span>
        </div>
      </div>

      <div class="divider my-0 opacity-10"></div>
    </div>

    <!-- MIDDLE SECTION: SCROLLABLE QUEUE LIST -->
    <div class="flex-1 overflow-y-auto pr-1 flex flex-col gap-2.5 my-2 scroll-smooth">
      {#if player.aiQueue.length > 0}
        {#each player.aiQueue as item, i}
          {@const theme = getScoreTheme(item.matchScore)}
          {@const indexNum = (i + 1) < 10 ? `0${i + 1}` : `${i + 1}`}

          <!-- PEMBATAS TERPISAH + TOMBOL REROLL SEBELUM LAGU KE-08 (MERAH) -->
          {#if i === 7}
            <div class="flex items-center justify-between pt-2 pb-1 px-1 border-t border-base-100/40 mt-1">
              <span class="text-[10px] font-mono font-bold text-rose-400 uppercase tracking-wider flex items-center gap-1">
                <Sparkles class="w-3 h-3" /> Wildcards
              </span>
              <button 
                type="button"
                class="btn btn-ghost btn-xs p-1 h-auto text-rose-400 hover:bg-rose-500/10 rounded-lg flex items-center gap-1 font-mono text-[10px]"
                onclick={() => player.refreshWildcards()}
                title="Reroll lagu discovery"
              >
                <RefreshCw class="w-3 h-3" />
                <span>REROLL</span>
              </button>
            </div>
          {/if}

          <button 
            type="button"
            class="w-full bg-base-300/40 hover:bg-base-300 border border-base-100/60 border-l-2 {theme.leftBorder} rounded-xl p-2.5 flex flex-col gap-2 transition-all group cursor-pointer text-left"
            onclick={() => player.play(item.song)}
          >
            <div class="flex items-center gap-2.5 truncate">
              <span class="text-[10px] font-mono font-bold text-base-content/30 w-4 text-center flex-shrink-0">
                {indexNum}
              </span>
              <div class="truncate flex-1">
                <div class="text-xs font-semibold text-base-content truncate group-hover:text-primary transition-colors">
                  {item.song.title}
                </div>
                <div class="text-[10px] text-base-content/50 truncate mt-0.5">
                  {item.song.artist} — <span class="opacity-70">{item.song.album || 'Single'}</span>
                </div>
              </div>
            </div>

            <div class="flex items-center gap-2 pl-6">
              <span class="badge badge-xs bg-emerald-500/10 text-emerald-400 border-emerald-500/20 font-mono text-[9px] truncate max-w-[110px]">
                {item.song.genre || 'Music'}
              </span>
              <span class="text-[9px] font-mono text-base-content/40 truncate">
                {item.reason}
              </span>
            </div>

            <div class="flex items-center gap-2.5 pl-6 pt-0.5">
              <div class="flex-1 h-1.5 rounded-full bg-base-100 overflow-hidden border border-base-100/50">
                <div 
                  class="h-full {theme.bar} transition-all duration-500 rounded-full"
                  style="width: {item.matchScore}%"
                ></div>
              </div>
              <span class="text-[10px] font-mono font-bold {theme.text} w-7 text-right">
                {item.matchScore}%
              </span>
            </div>
          </button>
        {/each}
      {:else}
        <div class="flex flex-col items-center justify-center my-auto py-12 text-center text-base-content/30 gap-2">
          <Music class="w-8 h-8 opacity-40 animate-bounce" />
          <span class="text-xs font-mono">Putar lagu untuk mengaktifkan AI Queue</span>
        </div>
      {/if}
    </div>
  </aside>
{/if}
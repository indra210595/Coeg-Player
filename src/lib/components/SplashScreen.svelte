<script lang="ts">
  import { onMount } from 'svelte';

  let { onFinish } = $props<{ onFinish?: () => void }>();
  let isFading = $state(false);
  let isVisible = $state(true);

  onMount(() => {
    // Splash tampil 1.8 detik lalu fade-out 500ms
    const timer = setTimeout(() => {
      isFading = true;
      setTimeout(() => {
        isVisible = false;
        if (onFinish) onFinish();
      }, 500);
    }, 1800);

    return () => clearTimeout(timer);
  });
</script>

{#if isVisible}
  <div 
    class="fixed inset-0 z-999 bg-[#0d0e15] flex flex-col items-center justify-center transition-opacity duration-500 select-none"
    class:opacity-0={isFading}
    class:pointer-events-none={isFading}
  >
    <!-- BACKGROUND GLOW NEON -->
    <div class="absolute w-64 h-64 bg-primary/20 rounded-full blur-3xl animate-pulse"></div>
    <div class="absolute w-48 h-48 bg-secondary/20 rounded-full blur-2xl animate-pulse delay-300"></div>

    <div class="relative flex flex-col items-center gap-6">
      <!-- LOGO ANIMATION -->
      <img 
        src="/coeg_player_logo.jpg" 
        alt="Coeg Player" 
        class="w-36 h-36 object-contain drop-shadow-[0_0_30px_rgba(168,85,247,0.6)] animate-bounce"
      />

      <!-- TYPOGRAPHY -->
      <div class="flex flex-col items-center gap-1.5">
        <h1 class="text-2xl font-black tracking-widest text-transparent bg-clip-text bg-linear-to-r from-cyan-400 via-primary to-secondary">
          COEG PLAYER
        </h1>
        <span class="text-[10px] font-mono text-primary/70 tracking-[0.3em] uppercase animate-pulse">
          Loading Audio Engine...
        </span>
      </div>
    </div>
  </div>
{/if}
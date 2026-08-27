<script lang="ts">
  import '../app.css';
  import SplashScreen from '$lib/components/SplashScreen.svelte';
  
  let { children } = $props();
</script>

<SplashScreen />

<div data-theme="dark" class="bg-base-300 text-base-content min-h-screen flex flex-col h-screen overflow-hidden select-none">
  {@render children()}
</div>
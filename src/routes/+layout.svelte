<script lang="ts">
  import '../app.css';
  
  let { children } = $props();
</script>

<div data-theme="dark" class="bg-base-300 text-base-content min-h-screen flex flex-col h-screen overflow-hidden select-none">
  {@render children()}
</div>
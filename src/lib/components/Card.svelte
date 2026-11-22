<script lang="ts">
  import type { Snippet } from "svelte";

  let { featured = false, children }: { featured?: boolean; children: Snippet } = $props();
</script>

<article class="card" class:featured>
  {@render children()}
</article>

<style>
  .card {
    position: relative;
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
    padding: var(--space-5);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-lg);
    background: var(--color-bg);
  }
  .card.featured {
    background: linear-gradient(160deg, #3a3f5c, #1f2238);
    border-color: var(--color-border-strong);
  }
</style>

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
    max-height: 190px;
    overflow: hidden;
    padding: var(--space-4) var(--space-5);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-lg);
    background: var(--game-bg-mid, var(--color-bg));
  }
  .card.featured {
    background: linear-gradient(160deg, var(--game-card-top, var(--card-top)), var(--game-bg-mid, var(--card-bottom)));
    border-color: var(--color-border-strong);
  }
</style>

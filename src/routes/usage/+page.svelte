<script lang="ts">
  import { onMount } from "svelte";
  import { goto } from "$app/navigation";

  onMount(() => {
    goto("/chat", { replaceState: true });
  });
</script>

<div class="flex h-full items-center justify-center text-sm text-muted-foreground">Mofu App</div>

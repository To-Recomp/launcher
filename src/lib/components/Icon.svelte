<script module lang="ts">
  export type IconName =
    | "download" | "settings" | "play" | "chevron-down" | "chevron-right" | "folder"
    | "info" | "minimize" | "maximize" | "close" | "display" | "graphics" | "audio"
    | "controls" | "saves" | "game" | "about";
</script>

<script lang="ts">
  let { name, size = 20 }: { name: IconName; size?: number } = $props();

  const paths: Record<IconName, string> = {
    download: "M12 3v12m0 0l-4-4m4 4l4-4M4 17v2a2 2 0 002 2h12a2 2 0 002-2v-2",
    settings: "M4 7h9m4 0h3M4 17h3m4 0h9M15 4v6M9 14v6",
    play: "M7 4.5v15l12-7.5z",
    "chevron-down": "M6 9l6 6 6-6",
    "chevron-right": "M9 6l6 6-6 6",
    folder: "M3 7a2 2 0 012-2h4l2 2h8a2 2 0 012 2v8a2 2 0 01-2 2H5a2 2 0 01-2-2z",
    info: "M12 11v5m0-8.5v.01M12 21a9 9 0 100-18 9 9 0 000 18z",
    minimize: "M5 12h14",
    maximize: "M5 5h14v14H5z",
    close: "M6 6l12 12M18 6L6 18",
    display: "M3 5h18v11H3zM8 20h8M12 16v4",
    graphics: "M4 20L16 8m-2-4l1 2 2 1-2 1-1 2-1-2-2-1 2-1zM18 14l.7 1.3L20 16l-1.3.7L18 18l-.7-1.3L16 16l1.3-.7z",
    audio: "M4 9v6h4l5 4V5L8 9zM16.5 8.5a5 5 0 010 7",
    controls: "M6 8h12a3 3 0 013 3v3a3 3 0 01-5.2 2L14 14h-4l-1.8 2A3 3 0 013 14v-3a3 3 0 013-3zM8 10.5v3M6.5 12h3",
    saves: "M5 4h11l3 3v13H5zM8 4v5h7V4M8 20v-6h8v6",
    game: "M4 8h16M4 16h16M9 5v6M15 13v6",
    about: "M12 11v5m0-8.5v.01M12 21a9 9 0 100-18 9 9 0 000 18z",
  };
</script>

<svg
  width={size}
  height={size}
  viewBox="0 0 24 24"
  fill="none"
  stroke="currentColor"
  stroke-width="1.8"
  stroke-linecap="round"
  stroke-linejoin="round"
  aria-hidden="true"
>
  <path d={paths[name]} />
</svg>

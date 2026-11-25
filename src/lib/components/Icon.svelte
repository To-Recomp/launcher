<script module lang="ts">
  import Check from "@lucide/svelte/icons/check";
  import ChevronDown from "@lucide/svelte/icons/chevron-down";
  import ChevronRight from "@lucide/svelte/icons/chevron-right";
  import Download from "@lucide/svelte/icons/download";
  import Folder from "@lucide/svelte/icons/folder";
  import Gamepad2 from "@lucide/svelte/icons/gamepad-2";
  import Info from "@lucide/svelte/icons/info";
  import Minus from "@lucide/svelte/icons/minus";
  import Monitor from "@lucide/svelte/icons/monitor";
  import Play from "@lucide/svelte/icons/play";
  import Save from "@lucide/svelte/icons/save";
  import Settings from "@lucide/svelte/icons/settings";
  import SlidersHorizontal from "@lucide/svelte/icons/sliders-horizontal";
  import Square from "@lucide/svelte/icons/square";
  import Trash from "@lucide/svelte/icons/trash";
  import TriangleAlert from "@lucide/svelte/icons/triangle-exclamation-point";
  import Volume2 from "@lucide/svelte/icons/volume-2";
  import WandSparkles from "@lucide/svelte/icons/wand-sparkles";
  import X from "@lucide/svelte/icons/x";

  /** The icons the launcher uses, by name: each one is a Lucide icon (https://lucide.dev, ISC license). */
  const icons = {
    download: Download,
    settings: SlidersHorizontal,
    trash: Trash,
    play: Play,
    "chevron-down": ChevronDown,
    "chevron-right": ChevronRight,
    folder: Folder,
    info: Info,
    minimize: Minus,
    maximize: Square,
    close: X,
    display: Monitor,
    graphics: WandSparkles,
    audio: Volume2,
    controls: Gamepad2,
    saves: Save,
    game: SlidersHorizontal,
    about: Info,
    gear: Settings,
    check: Check,
    alert: TriangleAlert,
  } as const;

  export type IconName = keyof typeof icons;
</script>

<script lang="ts">
  let { name, size = 20 }: { name: IconName; size?: number } = $props();

  const Glyph = $derived(icons[name]);
</script>

<!-- aria-hidden: an icon never speaks for itself, its button carries the label. -->
<Glyph {size} strokeWidth={1.8} aria-hidden="true" />

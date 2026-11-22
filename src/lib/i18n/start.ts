import { launcherLanguage } from "../launcher/api";
import { detectLocale, setLocale } from "./index.svelte";

/**
 * Sets the interface language before anything is shown: the one chosen in the launcher settings,
 * else the shipped language closest to the system's, else English.
 */
export async function startLanguage(preferred: readonly string[] = navigator.languages): Promise<void> {
  let chosen: string | null = null;
  try {
    chosen = await launcherLanguage();
  } catch {
    // No Tauri runtime (plain browser) or an unreadable file: the system's language decides.
  }
  const detected = detectLocale(chosen ? [chosen] : preferred);
  setLocale(detected);
}

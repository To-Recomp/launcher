import type { Game } from "./catalog/games";
import type { Location } from "./settings/types";

/**
 * Where a game's release and data folder live on this machine, or `null` while it isn't
 * installed. Installations and instances arrive with #3; until then no game has one.
 */
export function locationFor(_game: Game): Location | null {
  return null;
}

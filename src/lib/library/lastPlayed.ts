const key = (gameId: string) => `lastPlayed.${gameId}`;

/** The console the game was last played on, if it ever was. */
export function lastPlayedPlatform(gameId: string): string | null {
  try {
    return localStorage.getItem(key(gameId));
  } catch {
    return null; // storage can be unavailable; the game then opens on its default console
  }
}

export function rememberPlayed(gameId: string, platform: string) {
  try {
    localStorage.setItem(key(gameId), platform);
  } catch {
    // Not remembering is harmless.
  }
}

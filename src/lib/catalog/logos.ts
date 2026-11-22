const files = import.meta.glob("../../assets/logos/*.png", { eager: true, query: "?url", import: "default" }) as Record<
  string,
  string
>;

/** "../../assets/logos/Destiny2.png" -> "destiny2". */
const logos = new Map(
  Object.entries(files).map(([path, url]) => [path.split("/").pop()!.replace(/\.png$/, "").toLowerCase(), url]),
);

/**
 * "Tales of Destiny 2" -> "destiny2", "Tales of the Abyss" -> "abyss": the logo files are named
 * after the game with the "Tales of (the)" prefix and the spaces removed.
 */
export function logoKey(name: string): string {
  return name
    .replace(/^Tales of (the )?/i, "")
    .replace(/\s+/g, "")
    .toLowerCase();
}

/** The bundled logo of a game, or undefined when it has none (the title is shown as text). */
export function logoFor(name: string): string | undefined {
  return logos.get(logoKey(name));
}

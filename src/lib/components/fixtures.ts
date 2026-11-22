import type { Game } from "../catalog/games";

export const eternia: Game = {
  id: "toe",
  icon: "E",
  name: "Tales of Eternia",
  repository: "To-Recomp/toe",
  year: 2000,
  state: "not-installed",
  platforms: [{ platform: "PS1", regions: ["USA"], state: "playable", maturity: "alpha" }],
};

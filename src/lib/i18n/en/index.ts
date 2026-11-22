import { game } from "./game";
import { launcher } from "./launcher";
import { shell } from "./shell";

/**
 * The English messages, split by area of the interface. Keys are `<area>.<where>.<what>`; the
 * area prefix keeps the parts from colliding.
 */
export const en = { ...shell, ...game, ...launcher } as const;

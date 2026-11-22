import { checkLauncherUpdate, updateLauncher, updaterAvailable, updatePreferences, type UpdateInfo } from "../updater";

/** Whether the launcher can update itself, and the newer release it has found, for the screens. */
export class LauncherUpdates {
  available = $state(false);
  found = $state<UpdateInfo | null>(null);

  /** Learns whether this installation can update itself, then checks if the settings ask for it. */
  async start() {
    this.available = await updaterAvailable();
    if (!this.available) return;
    try {
      if ((await updatePreferences()).auto) await this.check();
    } catch {
      // Offline at startup is not worth a message; "Check now" reports failures.
    }
  }

  /** Looks for a newer release; rejects with the reason when it can't. */
  async check(): Promise<UpdateInfo | null> {
    this.found = await checkLauncherUpdate();
    return this.found;
  }

  /** Installs the newer release and restarts into it; resolves when there is nothing newer. */
  install() {
    return updateLauncher();
  }

  /** The channel changed: what was found on the other one no longer counts. */
  forget() {
    this.found = null;
  }
}

export const launcherUpdates = new LauncherUpdates();

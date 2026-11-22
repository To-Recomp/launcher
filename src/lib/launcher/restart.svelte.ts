/** Settings that only apply on the next start mark it here; the settings page offers the restart. */
class RestartNeeded {
  needed = $state(false);

  mark() {
    this.needed = true;
  }
}

export const restartNeeded = new RestartNeeded();

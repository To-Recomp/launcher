import { defineConfig } from "vitest/config";
import { svelte } from "@sveltejs/vite-plugin-svelte";
import { version } from "./package.json";

// The version the screens show is package.json's: no second copy to keep in step at each release.

// Tauri expects a fixed dev port and must not clear the terminal.
export default defineConfig(({ mode }) => ({
  plugins: [svelte()],
  define: { __APP_VERSION__: JSON.stringify(version) },
  clearScreen: false,
  server: { port: 1420, strictPort: true },
  resolve: mode === "test" ? { conditions: ["browser"] } : undefined,
  test: {
    environment: "jsdom",
    setupFiles: ["./src/test-setup.ts"],
    include: ["src/**/*.test.ts", "scripts/**/*.test.ts"],
    coverage: {
      provider: "v8",
      include: ["src/**/*.{ts,svelte}"],
      exclude: ["src/**/*.test.ts", "src/test-setup.ts", "src/main.ts"],
      reporter: ["text", "lcov"],
      thresholds: { lines: 90, functions: 90, statements: 90, branches: 65 },
    },
  },
}));

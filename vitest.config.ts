import path from "node:path";
import { defineConfig } from "vitest/config";
import react from "@vitejs/plugin-react";

export default defineConfig({
  plugins: [react()],
  resolve: {
    alias: {
      "@": path.resolve(__dirname, "./src"),
    },
  },
  test: {
    environment: "jsdom",
    setupFiles: ["./tests/setupGlobals.ts", "./tests/setupTests.ts"],
    globals: true,
    include: ["tests/**/*.test.{ts,tsx}"],
    exclude: ["cc-switch/**", "cc-notify-cli/**", "code-notify/**", "node_modules/**"],
    coverage: {
      reporter: ["text", "lcov"],
    },
  },
});

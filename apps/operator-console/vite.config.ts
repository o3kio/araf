import "vitest/config";
import react from "@vitejs/plugin-react";
import { defineConfig } from "vite";

// Operator Console dev server. Runs on a separate origin/port from the
// Tenant Console to mirror the deployment-level separation (ADR 0001).
export default defineConfig({
  plugins: [react()],
  server: {
    port: 5174,
    strictPort: true,
    proxy: {
      "/api": { target: "http://127.0.0.1:8081", changeOrigin: true },
    },
  },
  test: {
    environment: "jsdom",
    globals: true,
    setupFiles: ["./src/test/setup.ts"],
    css: false,
  },
});

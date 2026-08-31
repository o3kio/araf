import "vitest/config";
import react from "@vitejs/plugin-react";
import { defineConfig } from "vite";

// Tenant Console dev server. The Operator Console uses a different port so
// both surfaces can run side by side locally.
export default defineConfig({
  plugins: [react()],
  server: {
    port: 5173,
    strictPort: true,
  },
  test: {
    environment: "jsdom",
    globals: true,
    setupFiles: ["./src/test/setup.ts"],
    css: false,
  },
});

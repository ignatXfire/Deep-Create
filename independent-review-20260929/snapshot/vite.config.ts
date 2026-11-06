import { fileURLToPath } from "node:url";
import react from "@vitejs/plugin-react";
import { defineConfig } from "vitest/config";

export default defineConfig({
  root: fileURLToPath(new URL("./ui", import.meta.url)),
  base: "./",
  plugins: [react()],
  clearScreen: false,
  server: { host: "127.0.0.1", port: 1420, strictPort: true },
  preview: { host: "127.0.0.1", port: 1421, strictPort: true },
  build: {
    outDir: "dist",
    emptyOutDir: true,
    target: "es2022",
    sourcemap: false,
    reportCompressedSize: true
  },
  test: { environment: "node", include: ["tests/**/*.test.ts", "tests/**/*.test.tsx"] }
});

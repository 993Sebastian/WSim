/// <reference types="vitest/config" />
import yaml from "@modyfi/vite-plugin-yaml";
import react from "@vitejs/plugin-react";
import { defineConfig } from "vite";

export default defineConfig({
  plugins: [react(), yaml()],
  clearScreen: false,
  server: {
    // Fixed port expected by Tauri (app/src-tauri/tauri.conf.json → devUrl).
    port: 1420,
    strictPort: true,
    // Texts live in ../data/texte, outside the UI package.
    fs: { allow: [".."] },
  },
  build: {
    target: "es2022",
  },
  test: {
    environment: "jsdom",
    include: ["src/**/*.test.{ts,tsx}"],
  },
});

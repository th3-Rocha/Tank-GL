import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

// O design system mora em ../design e é importado direto de lá.
export default defineConfig({
  plugins: [react()],
  clearScreen: false,
  server: { port: 1420, strictPort: true, fs: { allow: [".."] } },
  build: { target: "es2022", outDir: "dist" },
});

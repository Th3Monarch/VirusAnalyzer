import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import tailwindcss from "@tailwindcss/vite";
import { basePath } from "./src/config";

export default defineConfig({
  base: basePath,
  plugins: [react(), tailwindcss()],
});

import { defineConfig } from "vite";
import { svelte } from "@sveltejs/vite-plugin-svelte";

import wasm from "vite-plugin-wasm";
import topLevelAwait from "vite-plugin-top-level-await";

import tailwindcss from "@tailwindcss/vite";
import path from "path";

// https://vite.dev/config/
export default defineConfig({

  plugins: [svelte(), wasm(), topLevelAwait(), tailwindcss()],
  resolve: {
    alias: {
      $lib: path.resolve("./src/lib"),
      $wasm: path.resolve("./wasm/pkg/wasm"),
    },
  },
  base: "./",
});

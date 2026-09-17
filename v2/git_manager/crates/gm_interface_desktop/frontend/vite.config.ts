import { defineConfig } from 'vite';
import { svelte, vitePreprocess } from '@sveltejs/vite-plugin-svelte';

// https://vitejs.dev/config/
export default defineConfig({
  plugins: [svelte({ preprocess: vitePreprocess() })],

  // Vite serves on this port; tauri.conf.json devPath must match.
  server: {
    port:       5173,
    strictPort: true,
    // Tauri expects an explicit host for the IPC to work correctly.
    host:       '127.0.0.1',
  },

  // Tells Vite to read TAURI_* env vars from the shell — these control
  // debug vs release builds in the Tauri CLI.
  envPrefix: ['VITE_', 'TAURI_'],

  build: {
    // Tauri supports es2021 and the modern Chrome embedded in WebKit.
    target:    ['es2021', 'chrome100', 'safari13'],
    // Disable minification for debug builds so stack traces are readable.
    minify:    !process.env.TAURI_DEBUG ? 'esbuild' : false,
    sourcemap: !!process.env.TAURI_DEBUG,
  },
});
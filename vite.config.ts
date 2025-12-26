import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';
import path from 'path';

// https://vitejs.dev/config/
export default defineConfig({
  plugins: [react()],

  // Tauri expects a fixed port, fail if that port is not available
  server: {
    port: 5174,  // DigiCode本体(5173)と競合しないように変更
    strictPort: true,
  },

  // to make use of `TAURI_DEBUG` and other env variables
  // https://tauri.app/start/frontend/vite/
  envPrefix: ['VITE_', 'TAURI_'],

  build: {
    // Tauri supports es2021
    target:
      process.env.TAURI_ENV_PLATFORM === 'windows' ? 'chrome105' : 'safari13',
    // don't minify for debug builds
    minify: !process.env.TAURI_ENV_DEBUG ? 'esbuild' : false,
    // produce sourcemaps for debug builds
    sourcemap: !!process.env.TAURI_ENV_DEBUG,
  },

  resolve: {
    alias: {
      '@': path.resolve(__dirname, 'src'),
    },
  },
});

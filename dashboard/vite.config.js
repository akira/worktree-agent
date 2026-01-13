import { defineConfig } from 'vite';
import { svelte } from '@sveltejs/vite-plugin-svelte';

export default defineConfig({
  plugins: [svelte()],
  build: {
    outDir: 'dist',
  },
  server: {
    proxy: {
      '/api': {
        target: 'http://localhost:3847',
        changeOrigin: true,
      },
    },
  },
});

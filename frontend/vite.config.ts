import { defineConfig } from 'vite'
import { svelte } from '@sveltejs/vite-plugin-svelte'

// https://vitejs.dev/config/
export default defineConfig({
  plugins: [svelte()],
  server: {
    proxy: {
      '/api': 'http://localhost:3000',
      '/budget': 'http://localhost:3000',
      '/categories': 'http://localhost:3000',
      '/cards': 'http://localhost:3000',
    }
  }
})

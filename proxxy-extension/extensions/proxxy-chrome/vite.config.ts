import { defineConfig } from 'vite'
import { resolve } from 'path'

export default defineConfig(({ mode }) => ({
  build: {
    outDir: './dist',
    rollupOptions: {
      input: {
        'background': resolve(__dirname, 'src/background/index.ts'),
        'devtools': resolve(__dirname, 'src/devtools/devtools.ts'),
        'devtools-panel': resolve(__dirname, 'src/devtools/panel.ts'),
        'popup': resolve(__dirname, 'src/popup/popup.ts'),
        'options': resolve(__dirname, 'src/options/options.ts')
      },
      output: {
        entryFileNames: '[name].js',
        chunkFileNames: '[name].js',
        assetFileNames: '[name].[ext]'
      }
    },
    watch: mode === 'development' ? {
      include: ['src/**/*']
    } : undefined
  },
  resolve: {
    alias: {
      '@': resolve(__dirname, 'src')
    }
  },
  server: {
    port: 3001,
    strictPort: true
  },
  plugins: mode === 'development' ? [
    {
      name: 'hot-reload',
      handleHotUpdate() {
        browser.runtime.reload()
      }
    }
  ] : []
}))
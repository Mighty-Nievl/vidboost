import tailwindcss from '@tailwindcss/vite'

export default defineNuxtConfig({
  devtools: { enabled: false },
  css: ['~/assets/css/main.css'],
  vite: {
    plugins: [tailwindcss()],
    server: {
      allowedHosts: ['.trycloudflare.com'],
      proxy: {
        '/api': { target: 'http://127.0.0.1:8080', changeOrigin: true },
      },
    },
  },
  runtimeConfig: {
    public: {
      apiBase: process.env.NUXT_PUBLIC_API_BASE || '',
    },
  },
  nitro: {
    routeRules: {
      '/api/**': { cors: true },
    },
  },
  app: {
    head: {
      title: 'VidBoost — Video Enhancer',
      meta: [
        { name: 'description', content: 'Enhance video lokal: warna, sharpen, denoise, upscale, kompresi, watermark, crop & rotate. Berbasis ffmpeg.' },
        { name: 'viewport', content: 'width=device-width, initial-scale=1, viewport-fit=cover' },
      ],
      link: [{ rel: 'icon', type: 'image/svg+xml', href: '/favicon.svg' }],
      script: [{
        innerHTML: "try{var t=localStorage.getItem('vidboost:theme');document.documentElement.dataset.theme=(t==='clay-dark'||t==='clay-light')?t:'clay-light'}catch(e){document.documentElement.dataset.theme='clay-light'}",
      }],
    },
  },
})

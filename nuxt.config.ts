// https://nuxt.com/docs/api/configuration/nuxt-config
export default defineNuxtConfig({
  compatibilityDate: '2025-07-15',
  devtools: { enabled: false }, // 禁用 devtools 避免 fork pool 问题

  // 全局 head 配置 - 禁用 referrer 防止B站图片防盗链
  app: {
    head: {
      meta: [
        { name: 'referrer', content: 'no-referrer' }
      ]
    }
  },

  // Pinia 状态管理
  modules: ['@pinia/nuxt'],

  // Naive UI 自动导入
  build: {
    transpile: ['naive-ui', '@juggle/resize-observer'],
  },

  // 禁用 SSR (Tauri 桌面应用)
  ssr: false,

  // Vite 配置
  vite: {
    optimizeDeps: {
      include: ['naive-ui'],
    },
  },
})

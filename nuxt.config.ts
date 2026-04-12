// https://nuxt.com/docs/api/configuration/nuxt-config
export default defineNuxtConfig({
  modules: [
    '@nuxt/eslint',
    '@nuxt/ui'
  ],

  ssr: false,
  devtools: {
    enabled: true
  },

  css: ['~/assets/css/main.css'],

  devServer: {
    port: 3124
  },

  compatibilityDate: '2025-01-15',

  nitro: {
    preset: 'static',
    prerender: {
      routes: ['/']
    }
  },

  eslint: {
    config: {
      stylistic: {
        commaDangle: 'never',
        braceStyle: '1tbs'
      }
    }
  }
})

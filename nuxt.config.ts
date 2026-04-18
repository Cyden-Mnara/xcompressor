// https://nuxt.com/docs/api/configuration/nuxt-config
declare const process: {
  env: Record<string, string | undefined>
}

export default defineNuxtConfig({

  modules: [
    '@nuxt/eslint',
    '@nuxt/ui'
  ],

  ssr: false,
  devtools: {
    enabled: true
  },
  app: {
    baseURL: process.env.NUXT_APP_BASE_URL || '/'
  },

  css: ['~/assets/css/main.css'],

  devServer: {
    port: 3124
  },

  compatibilityDate: '2025-01-15',

  nitro: {
    preset: 'static',
    prerender: {
      routes: ['/', '/app', '/credits', '/support']
    }
  },

  eslint: {
    config: {
      stylistic: {
        commaDangle: 'never',
        braceStyle: '1tbs'
      }
    }
  },

  icon: {
    provider: 'none',
    clientBundle: {
      icons: [
        'lucide:arrow-down',
        'lucide:arrow-left',
        'lucide:arrow-right',
        'lucide:arrow-up',
        'lucide:badge-check',
        'lucide:check',
        'lucide:check-circle-2',
        'lucide:chevron-down',
        'lucide:chevron-left',
        'lucide:chevron-right',
        'lucide:chevron-up',
        'lucide:circle-alert',
        'lucide:circle-check',
        'lucide:clapperboard',
        'lucide:download',
        'lucide:external-link',
        'lucide:film',
        'lucide:folder-open',
        'lucide:folder-output',
        'lucide:folder-plus',
        'lucide:git-commit-horizontal',
        'lucide:heart-handshake',
        'lucide:image',
        'lucide:info',
        'lucide:layers-3',
        'lucide:layout-dashboard',
        'lucide:list-checks',
        'lucide:monitor',
        'lucide:monitor-play',
        'lucide:moon',
        'lucide:music',
        'lucide:play',
        'lucide:plus',
        'lucide:refresh-ccw',
        'lucide:refresh-cw',
        'lucide:scroll-text',
        'lucide:skip-back',
        'lucide:square',
        'lucide:sun',
        'lucide:terminal',
        'lucide:trash-2',
        'lucide:users',
        'lucide:video',
        'lucide:x',
        'simple-icons:apple',
        'simple-icons:linux',
        'simple-icons:windows'
      ],
      scan: true,
      sizeLimitKb: 256
    }
  }
})

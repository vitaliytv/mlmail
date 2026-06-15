import { defineConfig, mergeConfig } from 'vitest/config'
import viteConfig from './vite.config.js'

// Frontend variant (n-test / n-vue canon): reuse the Vite plugins (Vue, VueMacros,
// AutoImport, Quasar) so .vue SFC compilation, auto-imports and Quasar transforms
// work natively under vitest — no bun-test preload hacks needed. happy-dom supplies
// the DOM for @vue/test-utils component mounting.
// vite.config.js exports a callback (env-based), so resolve it per configEnv before merge.
export default defineConfig(configEnv =>
  mergeConfig(typeof viteConfig === 'function' ? viteConfig(configEnv) : viteConfig, {
    test: {
      include: ['**/*.test.{js,mjs}', 'tests/**/*.test.{js,mjs}'],
      exclude: ['**/node_modules/**', '**/dist/**', '**/reports/stryker/**'],
      environment: 'happy-dom',
      // pool: 'forks' — defense-in-depth process isolation between test files (test.mdc).
      pool: 'forks',
      coverage: { provider: 'v8', reporter: ['lcov', 'text-summary'] }
    }
  })
)

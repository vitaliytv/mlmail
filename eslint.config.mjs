import { getConfig } from '@nitra/eslint-config'

export default [
  {
    ignores: [
      '**/auto-imports.d.ts',
      '**/reports/stryker/**',
      '.claude/worktrees/**',
      'app/src-tauri/**',
      'app/dist/**',
      'docs/**'
    ]
  },
  ...getConfig({
    node: ['scripts'],
    vue: ['app']
  }),
  {
    // Build/test tool configs run under Node, not the browser — override the
    // `vue: ['app']` browser globals above for just these files.
    files: [
      'app/vite.config.mjs',
      'app/vitest.config.mjs',
      'app/stryker.config.mjs',
      'app/stryker-vue-macros-ignorer.mjs'
    ],
    languageOptions: {
      globals: { process: 'readonly', URL: 'readonly' }
    }
  }
]

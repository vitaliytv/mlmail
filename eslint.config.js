import { getConfig } from '@nitra/eslint-config'

export default [
  {
    ignores: ['**/auto-imports.d.ts', 'src-tauri/**', 'dist/**']
  },
  ...getConfig({
    vue: ['.']
  })
]

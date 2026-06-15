/** @type {import('@stryker-mutator/core').PartialStrykerOptions} */
export default {
  testRunner: 'vitest',
  vitest: { configFile: 'vitest.config.mjs' },
  // perTest: Stryker runs only the tests covering a mutated line — the main speedup
  // over the command runner. vitest-runner AST-patches mutants in memory (no sandbox
  // node_modules copy), so inPlace is no longer needed.
  coverageAnalysis: 'perTest',
  mutate: [
    'src/services/auth-store.js',
    'src/i18n/auth-errors.js',
    'src/tool/*.js',
    'src/views/*.vue',
    'src/App.vue',
    '!src/**/*.test.js'
  ],
  // incremental: зберігає результати між запусками, відновлює після краш/kill.
  incremental: true,
  incrementalFile: 'reports/stryker/incremental.json',
  reporters: ['json', 'clear-text'],
  jsonReporter: { fileName: 'reports/stryker/mutation.json' },
  tempDirName: 'reports/stryker/.tmp',
  // Local plugin: skip mutating Vue <script setup> macros (defineProps/Emits/Model/
  // Slots/Expose/Options); else Stryker wraps macro args in a coverage ternary that
  // @vue/compiler-sfc cannot statically analyze and SFC compilation fails.
  plugins: ['@stryker-mutator/vitest-runner', './stryker-vue-macros-ignorer.mjs'],
  ignorers: ['vue-macros']
}

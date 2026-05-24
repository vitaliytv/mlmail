/** @type {import('@stryker-mutator/core').PartialStrykerOptions} */
export default {
  testRunner: 'command',
  commandRunner: {
    command: 'bun test --preload ./test/happy-dom.preload.js src'
  },
  // inPlace avoids hoisted-node_modules issues in a Bun monorepo sandbox
  inPlace: true,
  mutate: [
    'src/services/auth-store.js',
    'src/i18n/auth-errors.js',
    'src/views/*.vue',
    'src/App.vue',
    '!src/**/*.test.js'
  ],
  // concurrency:1 prevents parallel workers from competing over inPlace source files
  concurrency: 1,
  // 60 s per mutant — bun test + happy-dom SFC compilation needs headroom
  timeoutMS: 60000,
  reporters: ['json', 'clear-text'],
  jsonReporter: { fileName: 'reports/stryker/mutation.json' },
  tempDirName: 'reports/stryker/.tmp',
  coverageAnalysis: 'off'
}

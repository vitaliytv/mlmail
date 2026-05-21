import { parseArgs } from 'node:util'

/**
 * Parse docs-regen CLI arguments.
 * @param {string[]} argv Raw argument list (without the runtime and script entries).
 * @returns {{projection: string|undefined, all: boolean, dry: boolean, noMark: boolean, check: boolean}} Parsed options.
 */
export function parseCliArgs(argv) {
  const { values } = parseArgs({
    args: argv,
    options: {
      projection: { type: 'string' },
      all: { type: 'boolean', default: false },
      dry: { type: 'boolean', default: false },
      'no-mark': { type: 'boolean', default: false },
      check: { type: 'boolean', default: false }
    },
    strict: true,
    allowPositionals: false
  })
  return {
    projection: values.projection,
    all: values.all,
    dry: values.dry,
    noMark: values['no-mark'],
    check: values.check
  }
}

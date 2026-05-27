/* global Bun */
// @ts-nocheck

import { GlobalRegistrator } from '@happy-dom/global-registrator'
import { mock } from 'bun:test'

// happy-dom must register before Vue's runtime-dom is imported — runtime-dom
// captures `document` at module-eval time. Everything Vue-related below is
// therefore loaded via dynamic import, after register().
GlobalRegistrator.register()

// --- Vue SFC loader -------------------------------------------------------
// `bun test` has no Vite transform, so `.vue` files would load as raw text.
// Compile them on the fly with @vue/compiler-sfc (script setup + inline
// template; <style> blocks are irrelevant to behavior assertions).
const { parse, compileScript } = await import('@vue/compiler-sfc')

const VUE_FILE_RE = /\.vue$/

Bun.plugin({
  name: 'vue-sfc',
  setup(build) {
    build.onLoad({ filter: VUE_FILE_RE }, async args => {
      const source = await Bun.file(args.path).text()
      const { descriptor, errors } = parse(source, { filename: args.path })
      if (errors.length > 0) {
        throw new Error(`Vue SFC parse failed (${args.path}):\n${errors.join('\n')}`)
      }
      const id = Bun.hash(args.path).toString(36)
      const { content } = compileScript(descriptor, { id, inlineTemplate: true })
      return { contents: content, loader: 'js' }
    })
  }
})

// `bun test` has no unplugin-auto-import either, so expose the auto-imported
// Vue / Vue Router APIs (vite.config.js AutoImport presets) as globals — source
// files use bare `ref`, `computed`, `h`, `onMounted`, … without imports.
for (const mod of [await import('vue'), await import('vue-router')]) {
  for (const [name, value] of Object.entries(mod)) {
    if (name !== 'default' && !(name in globalThis)) globalThis[name] = value
  }
}

// `import 'quasar'` resolves to the SSR build under the `node` condition that
// `bun test` activates; force the browser/client build for component mounting.
const quasarClient = await import('quasar/dist/quasar.client.js')
mock.module('quasar', () => quasarClient)

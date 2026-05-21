import { mount } from '@vue/test-utils'
import * as Quasar from 'quasar'

// `bun test` has no @quasar/vite-plugin to auto-import components per file,
// so register every Quasar component (QBtn, QPage, …) globally.
const quasarComponents = Object.fromEntries(Object.entries(Quasar).filter(([name]) => /^Q[A-Z]/.test(name)))

/**
 * @param {object} component Vue component
 * @param {object} [options] mount options (forwarded)
 * @returns {object} test wrapper
 */
export function mountWithQuasar(component, options = {}) {
  const userGlobal = options.global || {}
  const userPlugins = userGlobal.plugins || []
  const wrapper = {
    render() {
      return h(Quasar.QLayout, { view: 'hHh lpR fFf' }, () => h(Quasar.QPageContainer, () => h(component)))
    }
  }
  return mount(wrapper, {
    ...options,
    global: {
      ...userGlobal,
      components: { ...quasarComponents, ...userGlobal.components },
      plugins: [...userPlugins, [Quasar.Quasar, { config: { dark: false } }]]
    }
  })
}

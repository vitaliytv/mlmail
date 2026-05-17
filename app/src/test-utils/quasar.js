import { mount } from '@vue/test-utils'
import { QLayout, QPageContainer, Quasar } from 'quasar'

/**
 * @param {object} component Vue component
 * @param {object} [options] mount options (forwarded)
 * @returns {object} test wrapper
 */
export function mountWithQuasar(component, options = {}) {
  const userPlugins = (options.global && options.global.plugins) || []
  const wrapper = {
    components: { QLayout, QPageContainer, Subject: component },
    render() {
      return h(QLayout, { view: 'hHh lpR fFf' }, () => h(QPageContainer, () => h(component)))
    }
  }
  return mount(wrapper, {
    ...options,
    global: {
      ...options.global,
      plugins: [...userPlugins, [Quasar, { config: { dark: false } }]]
    }
  })
}

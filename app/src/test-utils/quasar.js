import { mount } from '@vue/test-utils'
import { Quasar } from 'quasar'

export function mountWithQuasar(component, options = {}) {
  return mount(component, {
    ...options,
    global: {
      ...(options.global ?? {}),
      plugins: [[Quasar], ...(options.global?.plugins ?? [])],
    },
  })
}

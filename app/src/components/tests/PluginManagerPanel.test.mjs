import { flushPromises } from '@vue/test-utils'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { mountQuasar } from '../../test-utils/quasar.js'

const mocks = vi.hoisted(() => ({
  invoke: vi.fn(),
  open: vi.fn()
}))

vi.mock('@tauri-apps/api/core', () => ({ invoke: (...arguments_) => mocks.invoke(...arguments_) }))
vi.mock('@tauri-apps/plugin-dialog', () => ({ open: (...arguments_) => mocks.open(...arguments_) }))

const pluginManagerPanelModule = await import('../PluginManagerPanel.vue')
const PluginManagerPanel = pluginManagerPanelModule.default

const draftRelease = {
  package: 'vitaliytv:draft-helper',
  version: '1.2.3',
  digest: `sha256:${'a'.repeat(64)}`
}
const bookingRelease = {
  package: 'vitaliytv:booking-finder',
  version: '2.0.0',
  digest: `sha256:${'b'.repeat(64)}`
}
const selectedComponentPath = '/Users/test/draft-helper.n-plugin'
const installedPlugins = [
  {
    release: draftRelease,
    triggers: ['nitra:gmail/draft-helper@0.1.0'],
    actions: [
      {
        kind: 'draft-helper-create',
        label: 'Create draft',
        trigger: 'nitra:gmail/draft-helper@0.1.0'
      }
    ],
    enabled: true,
    lifecycle: { desiredState: 'enabled', effectiveState: 'active', disabledBy: null },
    activationGeneration: 7
  },
  {
    release: bookingRelease,
    triggers: ['nitra:gmail/booking-finder@0.1.0'],
    actions: [
      {
        kind: 'booking-finder-find',
        label: 'Find bookings',
        trigger: 'nitra:gmail/booking-finder@0.1.0'
      }
    ],
    enabled: true,
    lifecycle: { desiredState: 'enabled', effectiveState: 'active', disabledBy: null },
    activationGeneration: 8
  }
]
const preview = {
  previewId: `sha256:${'c'.repeat(64)}`,
  contractFingerprint: `sha256:${'d'.repeat(64)}`,
  release: draftRelease,
  supportedTriggers: ['nitra:gmail/draft-helper@0.1.0'],
  actions: installedPlugins[0].actions,
  dependencies: [
    {
      name: 'formatter',
      package: 'vitaliytv:mail-formatter',
      requirement: '=1.0.0',
      imports: ['vitaliytv:mail/format@1.0.0']
    }
  ],
  requiredCapabilities: [
    {
      requirementId: `sha256:${'e'.repeat(64)}`,
      capability: 'mail:draft.create',
      hostInterface: 'nitra:gmail/drafts@0.1.0',
      accountScope: 'current-account',
      accountId: 'person@example.com'
    }
  ],
  compatible: true,
  reason: null
}

/**
 * Mounts the dialog with Quasar and a local QDialog stub.
 * @returns {object} Vue Test Utils wrapper
 */
function mountPanel() {
  return mountQuasar(PluginManagerPanel, {
    props: { modelValue: true },
    global: {
      stubs: {
        QDialog: { template: '<div><slot /></div>' }
      }
    }
  })
}

/**
 * Finds one rendered button by its visible label.
 * @param {object} wrapper Vue Test Utils wrapper
 * @param {string} label visible button label
 * @returns {object | undefined} matching button wrapper
 */
function button(wrapper, label) {
  return wrapper.findAll('button').find(candidate => candidate.text().includes(label))
}

/** Flushes the immediate open watcher and its list request. */
async function openPanel() {
  await flushPromises()
}

beforeEach(() => {
  mocks.invoke.mockReset()
  mocks.open.mockReset()
  mocks.invoke.mockImplementation(command => {
    if (command === 'plugin_manager_list') return Promise.resolve(installedPlugins)
    return Promise.resolve(null)
  })
  mocks.open.mockResolvedValue(selectedComponentPath)
})

describe('PluginManagerPanel installation consent', () => {
  it('shows a preview for the selected Component and waits for explicit consent before activation', async () => {
    mocks.invoke.mockImplementation(command => {
      if (command === 'plugin_manager_list') return Promise.resolve(installedPlugins)
      if (command === 'plugin_manager_preflight') return Promise.resolve(preview)
      return Promise.resolve(null)
    })
    const wrapper = mountPanel()
    await openPanel()

    await button(wrapper, 'Встановити .n-plugin').trigger('click')
    await flushPromises()

    expect(mocks.invoke).toHaveBeenCalledWith('plugin_manager_preflight', {
      path: selectedComponentPath
    })
    expect(wrapper.text()).toContain('mail:draft.create')
    expect(wrapper.text()).toContain('person@example.com')
    expect(wrapper.text()).toContain('Create draft')
    expect(wrapper.text()).toContain('vitaliytv:mail-formatter =1.0.0')
    expect(mocks.invoke).not.toHaveBeenCalledWith('plugin_manager_confirm_install', expect.anything())
  })

  it('sends only exact preview identity and opaque consent decisions on confirm', async () => {
    mocks.invoke.mockImplementation(command => {
      if (command === 'plugin_manager_list') return Promise.resolve(installedPlugins)
      if (command === 'plugin_manager_preflight') return Promise.resolve(preview)
      if (command === 'plugin_manager_confirm_install') return Promise.resolve(installedPlugins[0])
      return Promise.resolve(null)
    })
    const wrapper = mountPanel()
    await openPanel()
    await button(wrapper, 'Встановити .n-plugin').trigger('click')
    await flushPromises()

    const consent = wrapper.find('.q-checkbox')
    await consent.trigger('click')
    await button(wrapper, 'Підтвердити встановлення').trigger('click')
    await flushPromises()

    expect(mocks.invoke).toHaveBeenCalledWith('plugin_manager_confirm_install', {
      confirmation: {
        path: selectedComponentPath,
        previewId: preview.previewId,
        expectedRelease: draftRelease,
        grants: [{ requirementId: preview.requiredCapabilities[0].requirementId, allow: true }]
      }
    })
    expect(wrapper.text()).not.toContain('Підтвердити встановлення')
  })

  it('shows compatibility errors and cancel never activates the candidate', async () => {
    const incompatible = {
      ...preview,
      compatible: false,
      reason: 'application did not register host interface `other:mail/raw@1.0.0`'
    }
    mocks.invoke.mockImplementation(command => {
      if (command === 'plugin_manager_list') return Promise.resolve(installedPlugins)
      if (command === 'plugin_manager_preflight') return Promise.resolve(incompatible)
      return Promise.resolve(null)
    })
    const wrapper = mountPanel()
    await openPanel()
    await button(wrapper, 'Встановити .n-plugin').trigger('click')
    await flushPromises()

    expect(wrapper.text()).toContain(incompatible.reason)
    expect(button(wrapper, 'Підтвердити встановлення').attributes('disabled')).toBeDefined()
    await button(wrapper, 'Скасувати').trigger('click')

    expect(mocks.invoke).not.toHaveBeenCalledWith('plugin_manager_confirm_install', expect.anything())
  })
})

describe('PluginManagerPanel exact installed actions', () => {
  it('renders backend projection and uses separate typed commands with exact releases', async () => {
    mocks.invoke.mockImplementation((command, arguments_) => {
      if (command === 'plugin_manager_list') return Promise.resolve(installedPlugins)
      if (command === 'plugin_draft_helper_create') {
        return Promise.resolve({ draftId: 'draft-1', release: arguments_.target, generation: 7 })
      }
      if (command === 'plugin_booking_finder_find') {
        return Promise.resolve({ query: 'from:(booking.com)', messages: [], release: arguments_.target, generation: 8 })
      }
      return Promise.resolve(null)
    })
    const wrapper = mountPanel()
    await openPanel()

    expect(mocks.invoke).toHaveBeenCalledWith('plugin_manager_list')
    expect(wrapper.text()).toContain(draftRelease.package)
    expect(wrapper.text()).toContain(draftRelease.version)
    expect(wrapper.text()).toContain(draftRelease.digest)
    expect(wrapper.text()).toContain('active')

    await button(wrapper, 'Create draft').trigger('click')
    await button(wrapper, 'Find bookings').trigger('click')
    await flushPromises()

    expect(mocks.invoke).toHaveBeenCalledWith('plugin_draft_helper_create', {
      target: draftRelease
    })
    expect(mocks.invoke).toHaveBeenCalledWith('plugin_booking_finder_find', {
      target: bookingRelease
    })
  })

  it('targets only the selected exact release for lifecycle changes', async () => {
    const wrapper = mountPanel()
    await openPanel()
    const rows = wrapper.findAll('[data-plugin-row]')

    await button(rows[0], 'Вимкнути').trigger('click')
    await button(rows[1], 'Видалити').trigger('click')
    await flushPromises()

    expect(mocks.invoke).toHaveBeenCalledWith('plugin_manager_set_disabled', {
      target: draftRelease,
      disabled: true
    })
    expect(mocks.invoke).toHaveBeenCalledWith('plugin_manager_uninstall', {
      target: bookingRelease
    })
  })

  it('preserves an operation error after the following successful list reload', async () => {
    mocks.invoke.mockImplementation(command => {
      if (command === 'plugin_manager_list') return Promise.resolve(installedPlugins)
      if (command === 'plugin_manager_set_disabled') return Promise.reject(new Error('lifecycle denied'))
      return Promise.resolve(null)
    })
    const wrapper = mountPanel()
    await openPanel()

    await button(wrapper.findAll('[data-plugin-row]')[0], 'Вимкнути').trigger('click')
    await flushPromises()

    expect(wrapper.text()).toContain('lifecycle denied')
    expect(mocks.invoke.mock.calls.filter(([command]) => command === 'plugin_manager_list')).toHaveLength(2)

    await button(wrapper, 'Оновити').trigger('click')
    await flushPromises()

    expect(wrapper.text()).toContain('lifecycle denied')
    expect(mocks.invoke.mock.calls.filter(([command]) => command === 'plugin_manager_list')).toHaveLength(3)
  })
})

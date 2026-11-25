import assert from 'node:assert/strict'
import { readFileSync } from 'node:fs'
import test from 'node:test'
import { fileURLToPath } from 'node:url'
import vm from 'node:vm'

const SOURCE = readFileSync(fileURLToPath(new URL('../client.js', import.meta.url)), 'utf8')

const React = {
  createElement(type, props, ...children) {
    return { type, props: props ?? {}, children: children.flat(Infinity) }
  },
  useState(initial) {
    return [initial, () => {}]
  },
  useEffect() {},
}

function load() {
  let module
  const window = { __ModuleLoader__: { load: (loaded) => (module = loaded) } }
  vm.runInNewContext(SOURCE, { window, console })
  assert.equal(module.id, 'rhobuilder-dsh-ui')
  const slots = new Map()
  const ctx = {
    slots: {
      inject(_name, register) {
        register()
      },
      register(meta, component) {
        slots.set(meta.name, { meta, component })
        return meta
      },
    },
  }
  module.factory(() => React).apply(ctx)
  return slots
}

function expand(node) {
  if (typeof node === 'function') return expand(node({}))
  if (Array.isArray(node)) return node.map(expand)
  if (!node || typeof node !== 'object') return node
  if (typeof node.type === 'function') {
    const children = node.children ?? []
    return expand(
      node.type({
        ...(node.props ?? {}),
        children: children.length === 1 ? children[0] : children,
      }),
    )
  }
  return { ...node, children: (node.children ?? []).map(expand) }
}

function text(node) {
  if (typeof node === 'string' || typeof node === 'number') return String(node)
  if (Array.isArray(node)) return node.map(text).join(' ')
  if (!node || typeof node !== 'object') return ''
  return text(node.children)
}

function buttons(node, found = []) {
  if (Array.isArray(node)) {
    node.forEach((child) => buttons(child, found))
  } else if (node && typeof node === 'object') {
    if (node.type === 'button') found.push(node)
    buttons(node.children, found)
  }
  return found
}

test('registers the rhobuilder surface', () => {
  const slots = load()
  assert.deepEqual([...slots.keys()], [
    'conversation.hero.frame',
    'conversation.hero.options',
    'sidebar.brand.mark',
    'sidebar.brand.name',
    'sidebar.panellist',
    'main',
    'settings.section',
  ])
  assert.equal(slots.get('main').meta.key, 'rhobuilder')
  assert.equal(slots.get('sidebar.panellist').meta.label, 'Rhobuilder')
})

test('the hero names rhobuilder, the primitive graph and the key boundary', () => {
  const hero = load().get('conversation.hero.frame').component
  const tree = expand(hero({ hero: true, children: React.createElement('div', null, 'conversation') }))
  const words = text(tree)
  assert.match(words, /rhobuilder/)
  assert.match(words, /rholo dApp studio/)
  assert.match(words, /r-cash, r-sure, r-gen, r-share and r-print/)
  assert.match(words, /never holds a key/)
  assert.match(words, /P5 devnet measured/)
  assert.match(words, /conversation/)
})

test('the hero is inert outside the blank home', () => {
  const hero = load().get('conversation.hero.frame').component
  assert.equal(hero({ hero: false, children: 'conversation' }), 'conversation')
})

test('the starters fill and submit the composer', () => {
  const starters = load().get('conversation.hero.options').component
  const actions = { draft: '', submits: 0, setDraft(value) { this.draft = value }, submit() { this.submits += 1 } }
  const tree = starters({ inputActions: actions })
  const found = buttons(tree)
  assert.equal(found.length, 4)
  found[0].props.onClick()
  assert.match(actions.draft, /create a design.json/)
  assert.equal(actions.submits, 1)
})

test('the dashboard exposes readiness, primitive and CLI sections', () => {
  const dashboard = load().get('main').component
  const words = text(expand(dashboard({})))
  assert.match(words, /Read-only studio/)
  assert.match(words, /Readiness/)
  assert.match(words, /Primitive graph/)
  assert.match(words, /CLI handoff/)
  assert.match(words, /Signed, funded and live actions stay in the terminal/)
  assert.match(words, /Truthful status/)
})

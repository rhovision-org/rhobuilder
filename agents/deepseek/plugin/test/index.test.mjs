import assert from 'node:assert/strict'
import test from 'node:test'

import { apply } from '../index.js'

function harness() {
  const tools = new Map()
  return {
    ctx: { tools: { register: (tool) => tools.set(tool.name, tool) } },
    tools,
  }
}

test('apply registers the six rhobuilder tools', () => {
  const { ctx, tools } = harness()
  apply(ctx, { command: '/usr/bin/rhobuilder' })
  assert.deepEqual([...tools.keys()], [
    'rhobuilder_registry',
    'rhobuilder_design',
    'rhobuilder_scaffold',
    'rhobuilder_check',
    'rhobuilder_plan',
    'rhobuilder_chain',
  ])
  for (const tool of tools.values()) {
    assert.equal(tool.parameters.type, 'object')
    assert.equal(tool.parameters.additionalProperties, false)
    assert.equal(tool.output.schema.type, 'object')
    assert.equal(typeof tool.output.render, 'function')
    assert.ok(tool.timeoutMs > 0)
  }
})

test('apply refuses a missing CLI command', () => {
  const { ctx } = harness()
  assert.throws(() => apply(ctx, {}), /config.command is required/)
})

test('the registry tool validates its action before spawning', async () => {
  const { ctx, tools } = harness()
  apply(ctx, { command: '/usr/bin/rhobuilder' })
  const registry = tools.get('rhobuilder_registry')
  await assert.rejects(registry.execute({}, {}), /`action` must be a non-empty string/)
  await assert.rejects(registry.execute({ action: 'nope' }, {}), /must be one of/)
  await assert.rejects(registry.execute({ action: 'get' }, {}), /`id` must be a non-empty string/)
  await assert.rejects(
    registry.execute({ action: 'verify' }, {}),
    /`network` must be a non-empty string/,
  )
  await assert.rejects(
    registry.execute({ action: 'drift' }, {}),
    /`sourceRoot` must be a non-empty string/,
  )
})

test('the design and chain tools require their payloads', async () => {
  const { ctx, tools } = harness()
  apply(ctx, { command: '/usr/bin/rhobuilder' })
  await assert.rejects(
    tools.get('rhobuilder_design').execute({ action: 'validate' }, {}),
    /`path` must be a non-empty string/,
  )
  await assert.rejects(
    tools.get('rhobuilder_chain').execute({ action: 'explore', network: 'testnet' }, {}),
    /must be one of/,
  )
  await assert.rejects(
    tools.get('rhobuilder_chain').execute({ action: 'deploy-status', network: 'testnet' }, {}),
    /`deployId` must be a non-empty string/,
  )
})

test('the plan renderer names the network and deploy actions', () => {
  const { ctx, tools } = harness()
  apply(ctx, { command: '/usr/bin/rhobuilder' })
  const blocks = tools.get('rhobuilder_plan').output.render(
    {},
    {
      schema: 'rhobuilder.deploy-plan/v1',
      ok: true,
      network: 'testnet',
      deploys: [
        { name: 'r-cash', action: 'reuse' },
        { name: 'panel-selection', action: 'deploy' },
      ],
    },
  )
  const text = blocks.map((block) => block.text).join('\n')
  assert.match(text, /deploy plan testnet: 2 step\(s\)/)
  assert.match(text, /r-cash: reuse/)
  assert.match(text, /panel-selection: deploy/)
})

test('the check renderer reports failures', () => {
  const { ctx, tools } = harness()
  apply(ctx, { command: '/usr/bin/rhobuilder' })
  const blocks = tools.get('rhobuilder_check').output.render(
    {},
    {
      ok: false,
      profile: 'static',
      gates: [
        { name: 'parse', status: 'pass' },
        { name: 'reuse', status: 'fail' },
      ],
    },
  )
  const text = blocks.map((block) => block.text).join('\n')
  assert.match(text, /check static: failed/)
  assert.match(text, /reuse: fail/)
  assert.match(text, /1 gate\(s\) failed/)
})

test('renderers are total on unexpected values', () => {
  const { ctx, tools } = harness()
  apply(ctx, { command: '/usr/bin/rhobuilder' })
  for (const tool of tools.values()) {
    const blocks = tool.output.render({}, null)
    assert.ok(Array.isArray(blocks))
    assert.equal(typeof blocks[0].text, 'string')
  }
})

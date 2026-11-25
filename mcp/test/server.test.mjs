import assert from 'node:assert/strict'
import { chmodSync, mkdtempSync, writeFileSync } from 'node:fs'
import { tmpdir } from 'node:os'
import { join } from 'node:path'
import test from 'node:test'

import { TOOLS, handle, parseArgs } from '../server.mjs'

const CFG = { command: '/usr/bin/rhobuilder', args: [], cwd: process.cwd() }

test('initialize answers the legacy lane', async () => {
  const response = await handle(
    {
      jsonrpc: '2.0',
      id: 1,
      method: 'initialize',
      params: { protocolVersion: '2025-11-25', capabilities: {} },
    },
    CFG,
  )
  assert.equal(response.result.protocolVersion, '2025-11-25')
  assert.equal(response.result.serverInfo.name, 'rhobuilder')
  assert.ok(response.result.capabilities.tools)
})

test('server/discover answers the modern lane with metadata', async () => {
  const response = await handle(
    { jsonrpc: '2.0', id: 2, method: 'server/discover', params: {} },
    CFG,
  )
  assert.equal(response.result.protocolVersion, '2026-07-28')
  assert.equal(response.result.resultType, 'complete')
  assert.equal(
    response.result._meta['io.modelcontextprotocol/protocolVersion'],
    '2026-07-28',
  )
})

test('tools/list exposes the read-only surface', async () => {
  const response = await handle(
    { jsonrpc: '2.0', id: 3, method: 'tools/list', params: {} },
    CFG,
  )
  const names = response.result.tools.map((tool) => tool.name)
  assert.deepEqual(names, [
    'rhobuilder_registry_list',
    'rhobuilder_registry_get',
    'rhobuilder_registry_graph',
    'rhobuilder_registry_drift',
    'rhobuilder_design_check',
    'rhobuilder_scaffold_plan',
    'rhobuilder_chain_probe',
    'rhobuilder_deploy_plan',
    'rhobuilder_baseline_plan',
    'rhobuilder_baseline_verify',
    'rhobuilder_check',
    'rhobuilder_formal_check',
  ])
  assert.equal(names.length, TOOLS.length)
  for (const tool of response.result.tools) {
    assert.equal(tool.inputSchema.type, 'object')
  }
})

test('modern tools/list carries the modern metadata', async () => {
  const response = await handle(
    {
      jsonrpc: '2.0',
      id: 4,
      method: 'tools/list',
      params: {},
      _meta: { 'io.modelcontextprotocol/protocolVersion': '2026-07-28' },
    },
    CFG,
  )
  assert.equal(response.result.resultType, 'complete')
  assert.equal(
    response.result._meta['io.modelcontextprotocol/protocolVersion'],
    '2026-07-28',
  )
})

test('an unknown tool is a tool error, not a crash', async () => {
  const response = await handle(
    {
      jsonrpc: '2.0',
      id: 5,
      method: 'tools/call',
      params: { name: 'nope', arguments: {} },
    },
    CFG,
  )
  assert.equal(response.result.isError, true)
})

test('a tool call runs the CLI bridge', async () => {
  const directory = mkdtempSync(join(tmpdir(), 'rhobuilder-mcp-'))
  const script = join(directory, 'fake.sh')
  writeFileSync(
    script,
    '#!/bin/sh\nprintf \'{"schema":"rhobuilder.registry-list/v1","ok":true,"count":0,"items":[]}\'\n',
  )
  chmodSync(script, 0o755)
  const response = await handle(
    {
      jsonrpc: '2.0',
      id: 6,
      method: 'tools/call',
      params: { name: 'rhobuilder_registry_list', arguments: {} },
    },
    { command: script, args: [], cwd: process.cwd() },
  )
  assert.equal(response.result.isError, false)
  assert.match(response.result.content[0].text, /"ok": true/)
})

test('parseArgs takes the command and its leading arguments', () => {
  const cfg = parseArgs(['--binary', 'cargo', 'run', '-q', '-p', 'rhobuilder-cli', '--'])
  assert.equal(cfg.command, 'cargo')
  assert.deepEqual(cfg.args, ['run', '-q', '-p', 'rhobuilder-cli', '--'])
})

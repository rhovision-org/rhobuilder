import assert from 'node:assert/strict'
import { chmodSync, mkdtempSync, writeFileSync } from 'node:fs'
import { tmpdir } from 'node:os'
import { join } from 'node:path'
import test from 'node:test'

import { buildInvocation, parseBridgeStdout, runBridge } from '../bridge.js'

function fakeScript(body) {
  const directory = mkdtempSync(join(tmpdir(), 'rhobuilder-bridge-'))
  const path = join(directory, 'fake.sh')
  writeFileSync(path, `#!/bin/sh\n${body}\n`)
  chmodSync(path, 0o755)
  return path
}

const CONFIG = { command: '/usr/bin/rhobuilder', args: [], cwd: process.cwd() }

test('version puts --json before the subcommand', () => {
  const invocation = buildInvocation('version', {}, CONFIG)
  assert.equal(invocation.command, '/usr/bin/rhobuilder')
  assert.deepEqual(invocation.args, ['--json', 'version'])
})

test('a cargo fallback keeps --json after the -- separator', () => {
  const invocation = buildInvocation(
    'version',
    {},
    { command: 'cargo', args: ['run', '-q', '-p', 'rhobuilder-cli', '--'] },
  )
  assert.deepEqual(invocation.args, [
    'run',
    '-q',
    '-p',
    'rhobuilder-cli',
    '--',
    '--json',
    'version',
  ])
})

test('registry get names the action and id', () => {
  const invocation = buildInvocation(
    'registry',
    { action: 'get', id: 'rcash.account-factory', network: 'testnet' },
    CONFIG,
  )
  assert.deepEqual(invocation.args, [
    '--json',
    'registry',
    'get',
    '--network',
    'testnet',
    '--id',
    'rcash.account-factory',
  ])
})

test('design validate names the action and path', () => {
  const invocation = buildInvocation(
    'design',
    { action: 'validate', path: 'design.json' },
    CONFIG,
  )
  assert.deepEqual(invocation.args, [
    '--json',
    'design',
    'validate',
    '--path',
    'design.json',
  ])
})

test('registry drift carries the source root and strict flag', () => {
  const invocation = buildInvocation(
    'registry',
    { action: 'drift', sourceRoot: '/tmp/rhobuilder-siblings', strict: true },
    CONFIG,
  )
  assert.deepEqual(invocation.args, [
    '--json',
    'registry',
    'drift',
    '--source-root',
    '/tmp/rhobuilder-siblings',
    '--strict',
  ])
})

test('registry verify is read-only by construction', () => {
  const invocation = buildInvocation(
    'registry',
    { action: 'verify', network: 'testnet' },
    CONFIG,
  )
  assert.deepEqual(invocation.args, [
    '--json',
    'registry',
    'verify',
    '--network',
    'testnet',
  ])
})

test('scaffold plans unless confirm is true', () => {
  const plan = buildInvocation(
    'scaffold',
    { target: '/tmp/app', design: 'design.json', template: 'headless' },
    CONFIG,
  )
  assert.deepEqual(plan.args, [
    '--json',
    'new',
    '/tmp/app',
    '--design',
    'design.json',
    '--template',
    'headless',
    '--plan',
  ])
  const apply = buildInvocation(
    'scaffold',
    { target: '/tmp/app', design: 'design.json', confirm: true },
    CONFIG,
  )
  assert.ok(apply.args.includes('--apply'))
  assert.ok(!apply.args.includes('--plan'))
})

test('chain status carries the network', () => {
  const invocation = buildInvocation(
    'chain',
    { action: 'status', network: 'testnet' },
    CONFIG,
  )
  assert.deepEqual(invocation.args, [
    '--json',
    'chain',
    'status',
    '--network',
    'testnet',
  ])
})

test('baseline plan and verify map to read-only commands', () => {
  const plan = buildInvocation(
    'baseline',
    { action: 'plan', network: 'testnet' },
    CONFIG,
  )
  assert.deepEqual(plan.args, [
    '--json',
    'baseline',
    'plan',
    '--network',
    'testnet',
  ])
  const verify = buildInvocation(
    'baseline',
    { action: 'verify', network: 'devnet' },
    CONFIG,
  )
  assert.deepEqual(verify.args, [
    '--json',
    'baseline',
    'verify',
    '--network',
    'devnet',
  ])
})

test('check maps to the project gate profile', () => {
  const invocation = buildInvocation(
    'check',
    { project: '/tmp/app', profile: 'formal' },
    CONFIG,
  )
  assert.deepEqual(invocation.args, [
    '--json',
    'check',
    '--project',
    '/tmp/app',
    '--profile',
    'formal',
  ])
})

test('plan maps to the read-only deploy plan', () => {
  const invocation = buildInvocation(
    'plan',
    { project: '/tmp/app', network: 'testnet' },
    CONFIG,
  )
  assert.deepEqual(invocation.args, [
    '--json',
    'deploy',
    'plan',
    '--project',
    '/tmp/app',
    '--network',
    'testnet',
  ])
})

test('an error envelope becomes a typed error with its hint', () => {
  assert.throws(
    () =>
      parseBridgeStdout(
        '{"kind":"error","error":{"type":"IdentityMismatch","status":"identity_mismatch","field":"network","message":"wrong genesis","hint":"Re-probe before planning."}}',
      ),
    (error) =>
      error.type === 'IdentityMismatch' &&
      error.status === 'identity_mismatch' &&
      error.field === 'network' &&
      error.hint === 'Re-probe before planning.' &&
      /wrong genesis/.test(error.message) &&
      /Re-probe before planning/.test(error.message),
  )
})

test('ok:false is also an error', () => {
  assert.throws(
    () => parseBridgeStdout('{"schema":"x","ok":false,"error":{"type":"Refused","message":"no"}}'),
    (error) => error.type === 'Refused',
  )
})

test('runBridge returns the parsed envelope', async () => {
  const script = fakeScript('printf \'{"schema":"rhobuilder.version/v1","ok":true,"version":"0.1.0"}\'')
  const document = await runBridge('version', {}, { command: script, args: [], cwd: process.cwd() })
  assert.equal(document.ok, true)
  assert.equal(document.version, '0.1.0')
})

test('runBridge surfaces a non-JSON failure', async () => {
  const script = fakeScript('printf "not json" >&2; exit 3')
  await assert.rejects(
    runBridge('version', {}, { command: script, args: [], cwd: process.cwd() }),
    /exited 3/,
  )
})

test('project status maps to the read-only CLI', () => {
  const invocation = buildInvocation(
    'project',
    { action: 'status', project: '/tmp/app' },
    CONFIG,
  )
  assert.deepEqual(invocation.args, [
    '--json',
    'project',
    'status',
    '--project',
    '/tmp/app',
  ])
})

test('design stdin uses --stdin and no path', () => {
  const invocation = buildInvocation(
    'design',
    { action: 'validate', stdin: true },
    CONFIG,
  )
  assert.deepEqual(invocation.args, ['--json', 'design', 'validate', '--stdin'])
})

test('a known error kind leads with actionable prose', () => {
  assert.throws(
    () =>
      parseBridgeStdout(
        '{"kind":"error","error":{"type":"IdentityMismatch","status":"identity_mismatch","message":"wrong genesis","hint":"Re-probe."}}',
      ),
    (error) =>
      /node is not the network/.test(error.message) &&
      error.prose &&
      /wrong genesis/.test(error.message) &&
      /Re-probe/.test(error.message),
  )
})

test('acceptRefusal returns a structured refusal document', () => {
  const document = parseBridgeStdout('{"schema":"x","ok":false,"verdict":"refuse","owed":["panel"]}', {
    acceptRefusal: true,
  })
  assert.equal(document.ok, false)
  assert.equal(document.verdict, 'refuse')
  assert.equal(document.owed[0], 'panel')
})

test('runBridge can feed stdin and return a refusal', async () => {
  const script = fakeScript(
    'cat >/dev/null; printf \'{"schema":"x","ok":false,"verdict":"refuse","owed":["panel"]}\'; exit 2',
  )
  const document = await runBridge(
    'design',
    { action: 'validate', stdin: true },
    { command: script, args: [], cwd: process.cwd() },
    { input: '{"schema":"rhobuilder.design/v1"}', acceptRefusal: true },
  )
  assert.equal(document.verdict, 'refuse')
  assert.equal(document.owed[0], 'panel')
})

test('buildInvocation refuses an unknown tool', () => {
  assert.throws(() => buildInvocation('nope', {}, CONFIG), /unknown tool/)
})

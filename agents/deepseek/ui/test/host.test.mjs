import assert from 'node:assert/strict'
import {
  chmodSync,
  existsSync,
  mkdirSync,
  mkdtempSync,
  readFileSync,
  symlinkSync,
  writeFileSync,
} from 'node:fs'
import { tmpdir } from 'node:os'
import { join } from 'node:path'
import { Readable } from 'node:stream'
import test from 'node:test'

import { apply } from '../index.js'

const FAKE_CLI = `#!/usr/bin/env node
let input = ''
process.stdin.on('data', (chunk) => { input += chunk })
process.stdin.on('end', () => {
  const argv = process.argv.join(' ')
  if (argv.includes('baseline') && argv.includes('preflight')) {
    process.stdout.write(JSON.stringify({
      schema: 'rhobuilder.baseline-preflight/v1',
      ok: false,
      blockers: [{ kind: 'missing-root', detail: 'r-gen is missing', hint: 'resolve the campaign' }],
    }))
    process.exit(0)
  }
  const bad = (argv + ' ' + input).includes('duplicate-escrow')
  const value = {
    schema: 'rhobuilder.design-check/v1',
    ok: !bad,
    verdict: bad ? 'refuse' : 'pass',
    app: { name: 'Draft', slug: bad ? 'duplicate-escrow' : 'bookable-service' },
    validation: { schema: 'rhobuilder.design-validation/v1', ok: !bad, checks: [] },
    reuse: {
      schema: 'rhobuilder.reuse-report/v1',
      verdict: bad ? 'refuse' : 'pass',
      ok: !bad,
      checks: [],
      conflicts: bad ? [{ component: 'escrow', primitive: 'rsure.policy', detail: 'already registered' }] : [],
      missing: [],
      reuse: [],
      extend: [],
      new: [],
    },
    owed: [],
    writes: false,
    source: 'stdin',
  }
  process.stdout.write(JSON.stringify(value))
  process.exit(bad ? 2 : 0)
})
`

function fakeCli() {
  const directory = mkdtempSync(join(tmpdir(), 'rhobuilder-ui-cli-'))
  const path = join(directory, 'rhobuilder')
  writeFileSync(path, FAKE_CLI, { mode: 0o755 })
  chmodSync(path, 0o755)
  return path
}

function harness({ admit = () => ({ rejection: 401 }), workspace = process.cwd(), command = '/usr/bin/rhobuilder' } = {}) {
  const routes = []
  let tap
  const ctx = {
    webServer: {
      tapIndex(transform) {
        tap = transform
        return () => {}
      },
      register(route) {
        routes.push(route)
        return () => {}
      },
    },
    connection: { admit },
  }
  apply(ctx, {
    command,
    args: [],
    cwd: process.cwd(),
    registry: new URL('../../../../registry', import.meta.url).pathname,
    workspace,
  })
  return { tap, routes, ctx }
}

function response() {
  return {
    status: 0,
    headers: {},
    body: '',
    writeHead(status, headers) {
      this.status = status
      this.headers = headers ?? {}
    },
    end(body) {
      this.body = body ?? ''
      this.done?.()
    },
  }
}

function call(route, url, { method = 'GET', headers = {}, body = '' } = {}) {
  return new Promise((resolve) => {
    const res = response()
    res.done = () => resolve(res)
    const request = Readable.from(body ? [body] : [])
    request.url = url
    request.method = method
    request.headers = headers
    route.handler(request, res)
  })
}

test('the index tap sets the rhobuilder title, favicon and description', () => {
  const { tap } = harness()
  const html = tap('<!doctype html><html><head><title>DeepSeek Harness</title><link rel="icon" href="old.svg"></head><body></body></html>')
  assert.match(html, /<title>rhobuilder — rholo dApp studio<\/title>/)
  assert.match(html, /rhobuilder-favicon\.svg/)
  assert.match(html, /rhobuilder — design, reuse, scaffold and test/)
  assert.doesNotMatch(html, /DeepSeek Harness<\/title>/)
})

test('static brand assets are served', async () => {
  const { routes } = harness()
  const favicon = routes.find((route) => route.path.endsWith('rhobuilder-favicon.svg'))
  const mark = routes.find((route) => route.path.endsWith('rhobuilder-mark.svg'))
  assert.ok(favicon && mark)
  const body = await favicon.resource()
  assert.match(await body.text(), /<svg/)
  assert.equal(body.headers.get('content-type'), 'image/svg+xml')
})

test('the API prefix requires admission and returns summary data', async () => {
  let admitted = false
  const { routes } = harness({
    admit: () => (admitted ? { peer: {} } : { rejection: 401 }),
  })
  const api = routes.find((route) => route.path === '/rhobuilder/api')
  assert.ok(api)

  const denied = await call(api, '/rhobuilder/api/summary')
  assert.equal(denied.status, 401)

  admitted = true
  const allowed = await call(api, '/rhobuilder/api/summary')
  assert.equal(allowed.status, 200)
  const value = JSON.parse(allowed.body)
  assert.equal(value.schema, 'rhobuilder.ui-summary/v1')
  assert.equal(value.safety.signing, 'cli-only')
  assert.ok(value.registry.primitives >= 1)
})

test('unknown API paths are 404 after admission', async () => {
  const { routes } = harness({ admit: () => ({ peer: {} }) })
  const api = routes.find((route) => route.path === '/rhobuilder/api')
  const result = await call(api, '/rhobuilder/api/nope')
  assert.equal(result.status, 404)
})

test('scaffold-plan refuses traversal before invoking the CLI', async () => {
  const { routes } = harness({ admit: () => ({ peer: {} }) })
  const api = routes.find((route) => route.path === '/rhobuilder/api')
  const result = await call(
    api,
    '/rhobuilder/api/scaffold-plan?design=../escape.json&template=headless',
  )
  assert.equal(result.status, 400)
})

const BOOKING = JSON.parse(
  readFileSync(new URL('../../../../fixtures/designs/booking.json', import.meta.url), 'utf8'),
)

function designApi(options = {}) {
  const workspace = options.workspace ?? mkdtempSync(join(tmpdir(), 'rhobuilder-ui-save-'))
  const { routes } = harness({
    admit: () => ({ peer: {} }),
    workspace,
    command: fakeCli(),
    ...options,
  })
  return { api: routes.find((route) => route.path === '/rhobuilder/api'), workspace }
}

function post(api, path, value) {
  return call(api, path, {
    method: 'POST',
    headers: { 'content-type': 'application/json' },
    body: typeof value === 'string' ? value : JSON.stringify(value),
  })
}

test('design validation is POST-only and requires admission', async () => {
  const denied = harness()
  const deniedApi = denied.routes.find((route) => route.path === '/rhobuilder/api')
  const unauthorized = await post(deniedApi, '/rhobuilder/api/design-validate', BOOKING)
  assert.equal(unauthorized.status, 401)

  const { api } = designApi()
  const wrongMethod = await call(api, '/rhobuilder/api/design-validate')
  assert.equal(wrongMethod.status, 405)
  assert.equal(wrongMethod.headers.allow, 'POST')
})

test('design validation returns a structured pass and refusal', async () => {
  const { api } = designApi()
  const passed = await post(api, '/rhobuilder/api/design-validate', BOOKING)
  assert.equal(passed.status, 200)
  const passValue = JSON.parse(passed.body)
  assert.equal(passValue.ok, true)
  assert.equal(passValue.writes, false)
  assert.equal(passValue.source, 'browser-draft')

  const refused = await post(api, '/rhobuilder/api/design-validate', {
    ...BOOKING,
    app: { ...BOOKING.app, slug: 'duplicate-escrow' },
  })
  assert.equal(refused.status, 200)
  const refusedValue = JSON.parse(refused.body)
  assert.equal(refusedValue.ok, false)
  assert.equal(refusedValue.reuse.conflicts.length, 1)
})

test('design validation rejects malformed and oversized bodies', async () => {
  const { api } = designApi()
  const malformed = await post(api, '/rhobuilder/api/design-validate', '{not json')
  assert.equal(malformed.status, 400)
  const oversized = await post(api, '/rhobuilder/api/design-validate', 'x'.repeat(70_000))
  assert.equal(oversized.status, 413)
})

test('design save writes a contained design.json and needs overwrite', async () => {
  const { api, workspace } = designApi()
  const first = await post(api, '/rhobuilder/api/design-save', { design: BOOKING })
  assert.equal(first.status, 200)
  const firstValue = JSON.parse(first.body)
  assert.equal(firstValue.ok, true)
  assert.equal(firstValue.path, 'designs/bookable-service.json')
  const target = join(workspace, 'designs', 'bookable-service.json')
  assert.ok(existsSync(target))
  assert.match(readFileSync(target, 'utf8'), /"schema": "rhobuilder.design\/v1"/)

  const second = await post(api, '/rhobuilder/api/design-save', { design: BOOKING })
  assert.equal(second.status, 409)
  assert.equal(JSON.parse(second.body).exists, true)

  const overwrite = await post(api, '/rhobuilder/api/design-save', {
    design: BOOKING,
    overwrite: true,
  })
  assert.equal(overwrite.status, 200)
  assert.equal(JSON.parse(overwrite.body).overwrite, true)

  const status = await call(api, '/rhobuilder/api/project-status')
  const value = JSON.parse(status.body)
  const saved = value.projects.find((row) => row.source === 'workspace')
  assert.ok(saved)
  assert.deepEqual(
    saved.badges.map((badge) => badge.label),
    ['Draft'],
  )
})

test('design save refuses path escapes and projects without a marker', async () => {
  const workspace = mkdtempSync(join(tmpdir(), 'rhobuilder-ui-save-'))
  const outside = mkdtempSync(join(tmpdir(), 'rhobuilder-ui-outside-'))
  symlinkSync(outside, join(workspace, 'escape'))
  const { api } = designApi({ workspace })

  const traversal = await post(api, '/rhobuilder/api/design-save', {
    design: BOOKING,
    project: '../escape',
  })
  assert.equal(traversal.status, 400)

  const absolute = await post(api, '/rhobuilder/api/design-save', {
    design: BOOKING,
    project: outside,
  })
  assert.equal(absolute.status, 400)

  const symlinked = await post(api, '/rhobuilder/api/design-save', {
    design: BOOKING,
    project: 'escape',
  })
  assert.equal(symlinked.status, 400)

  const project = join(workspace, 'project')
  mkdirSync(project)
  const missingMarker = await post(api, '/rhobuilder/api/design-save', {
    design: BOOKING,
    project: 'project',
  })
  assert.equal(missingMarker.status, 400)

  const good = join(workspace, 'good')
  mkdirSync(good)
  writeFileSync(join(good, '.rhobuilder.json'), '{}')
  const saved = await post(api, '/rhobuilder/api/design-save', {
    design: BOOKING,
    project: 'good',
  })
  assert.equal(saved.status, 200)
  assert.equal(JSON.parse(saved.body).path, 'good/design.json')
  assert.ok(existsSync(join(good, 'design.json')))
})

test('project-status derives fixture badges without a browser path', async () => {
  const { api } = designApi()
  const result = await call(api, '/rhobuilder/api/project-status')
  assert.equal(result.status, 200)
  const value = JSON.parse(result.body)
  const booking = value.projects.find((row) => row.slug === 'bookable-service')
  assert.ok(booking)
  assert.ok(booking.badges.some((badge) => badge.label === 'Scaffold'))
  const refused = value.projects.find((row) => row.slug === 'duplicate-escrow')
  assert.ok(refused.badges.some((badge) => badge.label === 'Refused'))
})

test('preflight is testnet-only and returns blockers', async () => {
  const { api } = designApi()
  const wrong = await call(api, '/rhobuilder/api/preflight?network=devnet')
  assert.equal(wrong.status, 400)
  const result = await call(api, '/rhobuilder/api/preflight?network=testnet')
  assert.equal(result.status, 200)
  const value = JSON.parse(result.body)
  assert.equal(value.schema, 'rhobuilder.baseline-preflight/v1')
  assert.equal(value.ok, false)
  assert.equal(value.blockers[0].kind, 'missing-root')
})

test('a refused design is never written', async () => {
  const { api, workspace } = designApi()
  const refused = await post(api, '/rhobuilder/api/design-save', {
    design: { ...BOOKING, app: { ...BOOKING.app, slug: 'duplicate-escrow' } },
  })
  assert.equal(refused.status, 200)
  assert.equal(JSON.parse(refused.body).writes, false)
  assert.equal(existsSync(join(workspace, 'designs', 'duplicate-escrow.json')), false)
})


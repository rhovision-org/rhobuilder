import assert from 'node:assert/strict'
import test from 'node:test'

import { apply } from '../index.js'

function harness({ admit = () => ({ rejection: 401 }) } = {}) {
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
    command: '/usr/bin/rhobuilder',
    args: [],
    cwd: process.cwd(),
    registry: new URL('../../../registry', import.meta.url).pathname,
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

function call(route, url) {
  return new Promise((resolve) => {
    const res = response()
    res.done = () => resolve(res)
    route.handler({ url, headers: {} }, res)
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

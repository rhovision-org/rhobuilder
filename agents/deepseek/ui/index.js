// Rhobuilder host half: brand the DSH index and expose read-only dashboard data.
//
// The browser half is client.js. This file runs in Node as a DSH plugin row. It never
// signs, funds or reads a key; it only shells to the fixed read-only CLI surfaces and
// serves local registry/evidence files behind DSH's browser-token fence.

import { execFile } from 'node:child_process'
import { readFileSync, readdirSync, existsSync } from 'node:fs'
import { dirname, join, resolve } from 'node:path'
import { fileURLToPath } from 'node:url'
import { promisify } from 'node:util'

import { runBridge } from '../plugin/bridge.js'

const HERE = dirname(fileURLToPath(import.meta.url))
const ROOT = resolve(HERE, '../../..')
const TITLE = 'rhobuilder — rholo dApp studio'
const API = '/rhobuilder/api'
const execFileAsync = promisify(execFile)

export const name = 'rhobuilder-ui'
export const inject = ['webServer', 'connection']

let readinessCache = { at: 0, value: null }
let summaryCache = { at: 0, value: null }
const networkCache = new Map()

function readAsset(name) {
  return readFileSync(join(HERE, 'assets', name), 'utf8')
}

function readJson(path, fallback = null) {
  try {
    return JSON.parse(readFileSync(path, 'utf8'))
  } catch {
    return fallback
  }
}

function sanitize(value) {
  if (Array.isArray(value)) return value.map(sanitize)
  if (!value || typeof value !== 'object') return value
  const out = {}
  for (const [key, item] of Object.entries(value)) {
    if (/secret|private.?key|keystore|passphrase/i.test(key)) continue
    out[key] = sanitize(item)
  }
  return out
}

function bridgeConfig(config) {
  const command = config?.command || join(ROOT, 'target', 'release', 'rhobuilder')
  return {
    command,
    args: Array.isArray(config?.args) ? config.args : [],
    cwd: config?.cwd || ROOT,
    timeoutMs: 20_000,
    maxOutputBytes: 1_000_000,
    registry: config?.registry || join(ROOT, 'registry'),
  }
}

async function readiness() {
  if (Date.now() - readinessCache.at < 20_000) return readinessCache.value
  const script = join(ROOT, 'tools', 'launch_dsh.py')
  let value
  if (!existsSync(script)) {
    value = {
      schema: 'rhobuilder.dsh-check/v1',
      ok: false,
      status: 'not_run',
      checks: [],
      note: 'tools/launch_dsh.py is missing from this installation',
    }
  } else {
    const python = process.env.RHOBUILDER_PYTHON || 'python3'
    try {
      const { stdout } = await execFileAsync(
        python,
        [script, '--check', '--json'],
        { cwd: ROOT, timeout: 45_000, maxBuffer: 1_000_000, env: process.env },
      )
      value = JSON.parse(stdout)
    } catch (error) {
      const stdout = typeof error?.stdout === 'string' ? error.stdout.trim() : ''
      if (stdout.startsWith('{')) {
        try {
          value = JSON.parse(stdout)
        } catch {
          value = null
        }
      }
      if (!value) {
        value = {
          schema: 'rhobuilder.dsh-check/v1',
          ok: false,
          status: 'not_run',
          checks: [],
          note: error?.message || 'readiness check did not run',
        }
      }
    }
  }
  readinessCache = { at: Date.now(), value }
  return value
}

function summary() {
  if (Date.now() - summaryCache.at < 5_000) return summaryCache.value
  const registry = readJson(join(ROOT, 'registry', 'primitives.json'), {})
  const networks = {}
  for (const network of ['devnet', 'testnet', 'mainnet']) {
    const profile = readJson(join(ROOT, 'registry', 'networks', `${network}.json`), {})
    networks[network] = {
      status: profile.status,
      deploy: profile.deploy,
      networkId: profile.networkId,
      shardId: profile.shardId,
      endpoint: profile.endpoint,
      roots: profile.roots ?? {},
      observed: profile.observed ?? null,
    }
  }
  const evidence = []
  const evidenceDir = join(ROOT, 'docs', 'src', 'evidence')
  if (existsSync(evidenceDir)) {
    for (const file of readdirSync(evidenceDir).filter((name) => name.endsWith('.json')).sort()) {
      const value = readJson(join(evidenceDir, file), null)
      if (value) evidence.push({ file, value: sanitize(value) })
    }
  }
  const value = {
    schema: 'rhobuilder.ui-summary/v1',
    version: '0.1.0',
    registry: {
      primitives: Array.isArray(registry.primitives) ? registry.primitives.length : 0,
      vertices: Array.isArray(registry.vertices) ? registry.vertices.length : 0,
      edges: Array.isArray(registry.edges) ? registry.edges.length : 0,
      pins: registry.pins ?? {},
    },
    networks,
    evidence,
    safety: {
      ui: 'read-only',
      signing: 'cli-only',
      mainnet: 'refused',
      testnet: 'gated',
    },
  }
  summaryCache = { at: Date.now(), value }
  return value
}

async function network(name, config) {
  const hit = networkCache.get(name)
  if (hit && Date.now() - hit.at < 15_000) return hit.value
  const profile = readJson(join(ROOT, 'registry', 'networks', `${name}.json`), null)
  if (!profile) throw new Error(`unknown network ${name}`)
  let probe = null
  let error = null
  try {
    probe = await runBridge('chain', { action: 'probe', network: name }, bridgeConfig(config))
  } catch (exc) {
    error = { type: exc?.type ?? 'ProbeError', message: String(exc?.message ?? exc) }
  }
  const value = { schema: 'rhobuilder.ui-network/v1', network: name, profile, probe, error }
  networkCache.set(name, { at: Date.now(), value })
  return value
}

function examples() {
  const dir = join(ROOT, 'fixtures', 'designs')
  if (!existsSync(dir)) return []
  return readdirSync(dir)
    .filter((name) => name.endsWith('.json'))
    .sort()
    .map((file) => {
      const value = readJson(join(dir, file), null)
      if (!value?.app) return null
      return {
        file,
        slug: value.app.slug,
        name: value.app.name,
        summary: value.app.summary,
        uniqueValue: value.uniqueValue,
        tags: (value.components ?? []).map((component) => ({
          id: component.id,
          tag: component.tag,
          primitive: component.primitive ?? null,
        })),
      }
    })
    .filter(Boolean)
}

function sendJson(res, status, value) {
  const body = JSON.stringify(value)
  res.writeHead(status, {
    'content-type': 'application/json; charset=utf-8',
    'cache-control': 'no-store',
    'content-length': Buffer.byteLength(body),
  })
  res.end(body)
}

async function apiHandler(req, res, ctx, config) {
  const admission = ctx.connection.admit(req)
  if ('rejection' in admission) {
    res.writeHead(admission.rejection)
    res.end()
    return
  }
  let target
  try {
    target = new URL(req.url ?? '/', 'http://127.0.0.1')
  } catch {
    sendJson(res, 400, { ok: false, error: 'bad request path' })
    return
  }
  const path = target.pathname.slice(API.length) || '/'
  try {
    if (path === '/' || path === '/summary') {
      sendJson(res, 200, summary())
      return
    }
    if (path === '/readiness') {
      sendJson(res, 200, await readiness())
      return
    }
    if (path === '/registry') {
      sendJson(res, 200, readJson(join(ROOT, 'registry', 'primitives.json'), {}))
      return
    }
    if (path === '/network') {
      const name = target.searchParams.get('name') || 'devnet'
      if (!['devnet', 'testnet', 'mainnet'].includes(name)) {
        sendJson(res, 400, { ok: false, error: 'network must be devnet, testnet or mainnet' })
        return
      }
      sendJson(res, 200, await network(name, config))
      return
    }
    if (path === '/examples') {
      sendJson(res, 200, { schema: 'rhobuilder.ui-examples/v1', examples: examples() })
      return
    }
    if (path === '/scaffold-plan') {
      const design = target.searchParams.get('design') || ''
      const template = target.searchParams.get('template') || 'headless'
      if (!/^[a-z0-9-]+\.json$/.test(design) || !['full', 'headless'].includes(template)) {
        sendJson(res, 400, { ok: false, error: 'design must be a fixture name and template full|headless' })
        return
      }
      const plan = await runBridge(
        'scaffold',
        {
          target: join(ROOT, '.rhobuilder-plan-preview'),
          design: join(ROOT, 'fixtures', 'designs', design),
          template,
          confirm: false,
        },
        bridgeConfig(config),
      )
      const files = Array.isArray(plan.plan?.files) ? plan.plan.files.length : 0
      sendJson(res, 200, {
        schema: 'rhobuilder.ui-scaffold-plan/v1',
        ok: true,
        design,
        template,
        count: files,
        files: plan.plan?.files ?? [],
        writes: false,
        note: 'Plan only; no files written. Apply is a separate, explicitly confirmed chat action.',
      })
      return
    }
    if (path === '/evidence') {
      sendJson(res, 200, { schema: 'rhobuilder.ui-evidence/v1', evidence: summary().evidence })
      return
    }
    sendJson(res, 404, { ok: false, error: `unknown rhobuilder route ${path}` })
  } catch (error) {
    sendJson(res, 500, { ok: false, error: String(error?.message ?? error) })
  }
}

function brandIndex(html) {
  let out = html.replace(/<title>[\s\S]*?<\/title>/i, `<title>${TITLE}</title>`)
  out = out.replace(/<link\b[^>]*\brel=["'][^"']*icon[^"']*["'][^>]*>/gi, '')
  const head = [
    '<meta name="description" content="rhobuilder — design, reuse, scaffold and test rhovision dApps on rholo.">',
    '<meta name="theme-color" content="#0f766e">',
    '<link rel="icon" type="image/svg+xml" href="/rhobuilder-ui/rhobuilder-favicon.svg">',
    '<link rel="apple-touch-icon" href="/rhobuilder-ui/rhobuilder-favicon.svg">',
    '<style>:root{--rb-brand:#0f766e;--rb-brand-strong:#115e59;--rb-ink:#0b1220;--rb-paper:#f7f5ef;--rb-amber:#b45309;--rb-muted:#5b6472}</style>',
  ].join('')
  if (out.includes('</head>')) out = out.replace('</head>', `${head}</head>`)
  else out = `${head}${out}`
  return out
}

function staticAsset(name, contentType) {
  return async () => {
    const body = readAsset(name)
    return new Response(body, {
      status: 200,
      headers: {
        'content-type': contentType,
        'cache-control': 'public, max-age=3600',
      },
    })
  }
}

export function apply(ctx, config = {}) {
  const web = ctx.webServer
  web.tapIndex(brandIndex)
  web.register({
    kind: 'exact',
    path: '/rhobuilder-ui/rhobuilder-mark.svg',
    resource: staticAsset('rhobuilder-mark.svg', 'image/svg+xml'),
  })
  web.register({
    kind: 'exact',
    path: '/rhobuilder-ui/rhobuilder-favicon.svg',
    resource: staticAsset('rhobuilder-favicon.svg', 'image/svg+xml'),
  })
  web.register({
    kind: 'prefix',
    path: API,
    handler: (req, res) => {
      void apiHandler(req, res, ctx, config)
    },
  })
}

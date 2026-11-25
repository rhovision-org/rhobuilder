// Rhobuilder host half: brand the DSH index and expose dashboard data.
//
// The browser half is client.js. This file runs in Node as a DSH plugin row. It never
// signs, funds or reads a key; it shells to fixed CLI surfaces, serves local
// registry/evidence files behind DSH's browser-token fence, and may write one validated
// design.json under a contained path.

import { execFile } from 'node:child_process'
import {
  existsSync,
  mkdirSync,
  readFileSync,
  readdirSync,
  realpathSync,
  renameSync,
  statSync,
  unlinkSync,
  writeFileSync,
} from 'node:fs'
import { dirname, isAbsolute, join, relative, resolve, sep } from 'node:path'
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
let projectStatusCache = { at: 0, key: '', value: null }
let preflightCache = { at: 0, value: null }
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
    for (const file of readdirSync(evidenceDir)
      .filter((name) => name.endsWith('.json') && !name.includes('.template.'))
      .sort()) {
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
      ui: 'design-save-only',
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
          leanStatus: component.leanStatus ?? null,
        })),
      }
    })
    .filter(Boolean)
}

function evidenceFor(slug) {
  const documents = summary().evidence ?? []
  return documents
    .map((item) => item.value)
    .find((value) => value?.app?.slug === slug || value?.app?.name === slug)
}

function badgesFor(example, report, { evidence = true, draft = false } = {}) {
  const badges = []
  if (report?.ok) {
    badges.push({
      label: draft ? 'Draft' : 'Scaffold',
      state: draft ? 'neutral' : 'ok',
      detail: draft
        ? 'validated design saved for continuity; no project files are implied'
        : 'design passes the reuse gate; project files may not exist yet',
    })
  } else if (report?.verdict || report?.validation || report?.reuse) {
    badges.push({ label: 'Refused', state: 'fail', detail: report?.error?.hint ?? 'the design or reuse gate refused this draft' })
  } else {
    badges.push({ label: 'Not checked', state: 'warn', detail: report?.error?.hint ?? 'the design gate did not run' })
  }
  const evidenceValue = evidence ? evidenceFor(example.slug) : null
  if (evidenceValue?.app?.verified === true || evidenceValue?.app?.status === 'verified') {
    badges.push({ label: 'Implemented', state: 'ok', detail: 'app root recorded as verified' })
  }
  const suites = evidenceValue?.suites?.results
  if (Array.isArray(suites)) {
    const pass = suites.filter((suite) => suite?.status === 'pass').length
    const fail = suites.filter((suite) => suite?.status === 'fail').length
    const notRun = suites.filter((suite) => suite?.status === 'not_run').length
    badges.push({
      label: 'Tested',
      state: fail === 0 && notRun === 0 && pass > 0 ? 'pass' : fail > 0 ? 'fail' : 'not_run',
      detail: `${pass} pass, ${fail} fail, ${notRun} not run`,
    })
  } else if (evidenceValue?.checks?.make_unit_all?.startsWith('not_run')) {
    badges.push({ label: 'Not run', state: 'warn', detail: evidenceValue.checks.make_unit_all })
  }
  const owed = Array.isArray(report?.owed) ? report.owed : []
  if (owed.length > 0) {
    badges.push({ label: 'Owed', state: 'warn', detail: owed.join(', ') })
  }
  if (
    evidenceValue &&
    (example.tags ?? []).some((tag) => tag.leanStatus === 'proved-model')
  ) {
    badges.push({ label: 'Proved model', state: 'warn', detail: 'shallow Lean model; not a Rholang refinement' })
  }
  return badges
}

async function projectStatus(config) {
  const key = workspaceRoot(config)
  if (Date.now() - projectStatusCache.at < 30_000 && projectStatusCache.key === key) {
    return projectStatusCache.value
  }
  const rows = []
  for (const example of examples()) {
    let report
    try {
      report = await runBridge(
        'design',
        { action: 'validate', path: join(ROOT, 'fixtures', 'designs', example.file) },
        bridgeConfig(config),
        { acceptRefusal: true },
      )
    } catch (error) {
      report = { ok: false, error: { hint: String(error?.message ?? error) }, owed: [] }
    }
    rows.push({
      slug: example.slug,
      name: example.name,
      source: 'fixture',
      file: example.file,
      verdict: report?.verdict ?? (report?.ok ? 'pass' : 'refuse'),
      owed: Array.isArray(report?.owed) ? report.owed : [],
      badges: badgesFor(example, report),
    })
  }
  const designsDir = join(workspaceRoot(config), 'designs')
  if (existsSync(designsDir)) {
    for (const file of readdirSync(designsDir).filter((name) => name.endsWith('.json')).sort()) {
      const raw = readFileSync(join(designsDir, file), 'utf8')
      let report
      try {
        report = await runBridge(
          'design',
          { action: 'validate', stdin: true },
          bridgeConfig(config),
          { input: raw, acceptRefusal: true },
        )
      } catch (error) {
        report = { ok: false, error: { hint: String(error?.message ?? error) }, owed: [] }
      }
      let design = null
      try {
        design = JSON.parse(raw)
      } catch {
        // The report already carries the parse failure.
      }
      const example = {
        slug: design?.app?.slug ?? file.replace(/\.json$/, ''),
        name: design?.app?.name ?? file,
        tags: (design?.components ?? []).map((component) => ({
          id: component.id,
          tag: component.tag,
          leanStatus: component.leanStatus ?? null,
        })),
      }
      rows.push({
        slug: example.slug,
        name: example.name,
        source: 'workspace',
        file: `designs/${file}`,
        verdict: report?.verdict ?? (report?.ok ? 'pass' : 'refuse'),
        owed: Array.isArray(report?.owed) ? report.owed : [],
        badges: badgesFor(example, report, { evidence: false, draft: true }),
      })
    }
  }
  const value = { schema: 'rhobuilder.ui-project-status/v1', projects: rows }
  projectStatusCache = { at: Date.now(), key, value }
  return value
}

async function preflight(config) {
  if (Date.now() - preflightCache.at < 30_000) return preflightCache.value
  const value = await runBridge(
    'baseline',
    { action: 'preflight', network: 'testnet' },
    bridgeConfig(config),
    { acceptRefusal: true },
  )
  preflightCache = { at: Date.now(), value }
  return value
}

function sendJson(res, status, value, extraHeaders = {}) {
  const body = JSON.stringify(value)
  res.writeHead(status, {
    'content-type': 'application/json; charset=utf-8',
    'cache-control': 'no-store',
    'content-length': Buffer.byteLength(body),
    ...extraHeaders,
  })
  res.end(body)
}

function workspaceRoot(config) {
  return resolve(config?.workspace || process.env.RHOBUILDER_WORKSPACE || ROOT)
}

function within(root, target) {
  const rel = relative(root, target)
  return rel === '' || (!rel.startsWith('..') && !isAbsolute(rel))
}

function readBody(req, limit = 65_536) {
  return new Promise((resolveBody, reject) => {
    let declared = Number(req.headers?.['content-length'])
    if (Number.isFinite(declared) && declared > limit) {
      const error = new Error('request body is too large')
      error.status = 413
      reject(error)
      return
    }
    let size = 0
    let settled = false
    const chunks = []
    const fail = (error) => {
      if (settled) return
      settled = true
      reject(error)
    }
    req.on('data', (chunk) => {
      if (settled) return
      const buffer = Buffer.isBuffer(chunk) ? chunk : Buffer.from(chunk)
      size += buffer.length
      if (size > limit) {
        const error = new Error('request body is too large')
        error.status = 413
        fail(error)
        req.destroy?.()
        return
      }
      chunks.push(buffer)
    })
    req.on('end', () => {
      if (settled) return
      settled = true
      resolveBody(Buffer.concat(chunks).toString('utf8'))
    })
    req.on('error', fail)
  })
}

function isJson(req) {
  return String(req.headers?.['content-type'] ?? '')
    .toLowerCase()
    .startsWith('application/json')
}

async function validateDesignBody(body, config) {
  const report = await runBridge(
    'design',
    { action: 'validate', stdin: true },
    bridgeConfig(config),
    { input: body, acceptRefusal: true },
  )
  return { ...report, writes: false, source: 'browser-draft' }
}

function designTarget(body, config) {
  const workspace = workspaceRoot(config)
  const design = body?.design
  if (!design || typeof design !== 'object' || Array.isArray(design)) {
    const error = new Error('design must be a JSON object')
    error.status = 400
    throw error
  }
  const slug = design.app?.slug
  if (typeof slug !== 'string' || !/^[a-z0-9]+(?:-[a-z0-9]+)*$/.test(slug)) {
    const error = new Error('design.app.slug must be a kebab-case slug')
    error.status = 400
    throw error
  }
  let path
  if (body.project !== undefined) {
    if (typeof body.project !== 'string' || body.project.trim() === '') {
      const error = new Error('project must be a relative path')
      error.status = 400
      throw error
    }
    const requested = body.project.trim()
    if (isAbsolute(requested) || requested.split(/[\\/]/).includes('..')) {
      const error = new Error('project must stay inside the workspace')
      error.status = 400
      throw error
    }
    const candidate = resolve(workspace, requested)
    if (!within(workspace, candidate)) {
      const error = new Error('project must stay inside the workspace')
      error.status = 400
      throw error
    }
    let project
    try {
      project = realpathSync(candidate)
    } catch {
      const error = new Error('project directory does not exist')
      error.status = 400
      throw error
    }
    if (!within(workspace, project) || !statSync(project).isDirectory()) {
      const error = new Error('project must stay inside the workspace')
      error.status = 400
      throw error
    }
    if (!existsSync(join(project, '.rhobuilder.json'))) {
      const error = new Error('project has no .rhobuilder.json marker')
      error.status = 400
      throw error
    }
    path = join(project, 'design.json')
  } else {
    const designs = join(workspace, 'designs')
    mkdirSync(designs, { recursive: true, mode: 0o755 })
    const realDesigns = realpathSync(designs)
    if (!within(workspace, realDesigns)) {
      const error = new Error('designs directory escapes the workspace')
      error.status = 400
      throw error
    }
    path = join(realDesigns, `${slug}.json`)
  }
  return { path, workspace }
}

function saveDesign(body, config) {
  const { path, workspace } = designTarget(body, config)
  const raw = `${JSON.stringify(body.design, null, 2)}\n`
  const overwrite = body.overwrite === true
  if (existsSync(path) && !overwrite) {
    const error = new Error('design.json already exists')
    error.status = 409
    error.exists = true
    error.path = relative(workspace, path).split(sep).join('/')
    throw error
  }
  const temporary = join(
    dirname(path),
    `.design-${process.pid}-${Date.now()}-${Math.random().toString(16).slice(2)}.tmp`,
  )
  try {
    writeFileSync(temporary, raw, { mode: 0o644 })
    renameSync(temporary, path)
  } catch (error) {
    try {
      unlinkSync(temporary)
    } catch {
      // The temporary file may not exist.
    }
    throw error
  }
  return {
    schema: 'rhobuilder.design-save/v1',
    ok: true,
    path: relative(workspace, path).split(sep).join('/'),
    bytes: Buffer.byteLength(raw),
    overwrite,
    writes: true,
  }
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
  const method = req.method || 'GET'
  const postRoutes = new Set(['/design-validate', '/design-save'])
  if (postRoutes.has(path) && method !== 'POST') {
    sendJson(res, 405, { ok: false, error: 'use POST' }, { allow: 'POST' })
    return
  }
  if (!postRoutes.has(path) && method !== 'GET') {
    sendJson(res, 405, { ok: false, error: 'use GET' }, { allow: 'GET' })
    return
  }
  try {
    if (path === '/design-validate') {
      if (!isJson(req)) {
        sendJson(res, 400, { ok: false, error: 'content-type must be application/json' })
        return
      }
      let raw
      try {
        raw = await readBody(req)
      } catch (error) {
        sendJson(res, error?.status || 400, { ok: false, error: String(error?.message ?? error) })
        return
      }
      let parsed
      try {
        parsed = JSON.parse(raw)
      } catch {
        sendJson(res, 400, { ok: false, error: 'body is not valid JSON' })
        return
      }
      if (!parsed || typeof parsed !== 'object' || Array.isArray(parsed)) {
        sendJson(res, 400, { ok: false, error: 'design must be a JSON object' })
        return
      }
      sendJson(res, 200, await validateDesignBody(raw, config))
      return
    }
    if (path === '/design-save') {
      if (!isJson(req)) {
        sendJson(res, 400, { ok: false, error: 'content-type must be application/json' })
        return
      }
      let raw
      try {
        raw = await readBody(req)
      } catch (error) {
        sendJson(res, error?.status || 400, { ok: false, error: String(error?.message ?? error) })
        return
      }
      let parsed
      try {
        parsed = JSON.parse(raw)
      } catch {
        sendJson(res, 400, { ok: false, error: 'body is not valid JSON' })
        return
      }
      if (!parsed?.design || typeof parsed.design !== 'object' || Array.isArray(parsed.design)) {
        sendJson(res, 400, { ok: false, error: 'body.design must be a JSON object' })
        return
      }
      const report = await validateDesignBody(JSON.stringify(parsed.design), config)
      if (!report.ok) {
        sendJson(res, 200, {
          schema: 'rhobuilder.design-save/v1',
          ok: false,
          writes: false,
          report,
        })
        return
      }
      try {
        const saved = saveDesign(parsed, config)
        sendJson(res, 200, { ...saved, report })
      } catch (error) {
        if (error?.status === 409) {
          sendJson(res, 409, {
            ok: false,
            exists: true,
            path: error.path,
            error: String(error.message),
          })
          return
        }
        if (error?.status === 400) {
          sendJson(res, 400, { ok: false, error: String(error.message) })
          return
        }
        throw error
      }
      return
    }
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
    if (path === '/project-status') {
      sendJson(res, 200, await projectStatus(config))
      return
    }
    if (path === '/preflight') {
      const name = target.searchParams.get('network') || 'testnet'
      if (name !== 'testnet') {
        sendJson(res, 400, { ok: false, error: 'preflight is testnet-only' })
        return
      }
      sendJson(res, 200, await preflight(config))
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

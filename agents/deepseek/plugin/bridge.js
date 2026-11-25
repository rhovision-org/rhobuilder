// argv, environment and JSON plumbing between a DSH tool and `rhobuilder --json`.
//
// This file imports nothing from DeepSeek Harness, so `node --test` exercises it without a
// runtime and the plugin mounts by absolute path without node_modules. The Rust CLI owns
// every decision; this file only validates and forwards.

import { spawn } from 'node:child_process'

const DEFAULT_TIMEOUT_MS = 120_000
const DEFAULT_MAX_OUTPUT_BYTES = 4 * 1024 * 1024

function requireString(args, name) {
  if (typeof args?.[name] !== 'string' || args[name].length === 0) {
    throw new Error(`rhobuilder: \`${name}\` must be a non-empty string`)
  }
  return args[name]
}

function requireEnum(args, name, allowed) {
  const value = requireString(args, name)
  if (!allowed.includes(value)) {
    throw new Error(`rhobuilder: \`${name}\` must be one of ${allowed.join(', ')}`)
  }
  return value
}

function pushFlag(argv, flag, value) {
  if (value !== undefined && value !== null && value !== '') argv.push(flag, String(value))
}

export function buildInvocation(tool, args = {}, cfg = {}) {
  if (!cfg || typeof cfg.command !== 'string' || cfg.command.length === 0) {
    throw new Error('rhobuilder bridge: config.command is required')
  }
  const base = [cfg.command, ...(cfg.args ?? []), '--json']
  let argv
  switch (tool) {
    case 'version':
      argv = [...base, 'version']
      break
    case 'registry': {
      const action = requireEnum(args, 'action', ['list', 'get', 'graph', 'verify', 'drift'])
      argv = [...base, 'registry', action]
      pushFlag(argv, '--network', args.network)
      pushFlag(argv, '--id', args.id)
      pushFlag(argv, '--owner', args.owner)
      pushFlag(argv, '--kind', args.kind)
      pushFlag(argv, '--root', args.root)
      pushFlag(argv, '--depth', args.depth)
      pushFlag(argv, '--source-root', args.sourceRoot)
      if (action === 'drift' && args.strict === true) argv.push('--strict')
      break
    }
    case 'design': {
      const action = requireEnum(args, 'action', ['validate', 'report'])
      if (args.stdin === true) {
        argv = [...base, 'design', action, '--stdin']
      } else {
        const path = requireString(args, 'path')
        argv = [...base, 'design', action, '--path', path]
      }
      break
    }
    case 'scaffold': {
      const target = requireString(args, 'target')
      const design = requireString(args, 'design')
      const template = args.template ?? 'full'
      if (!['full', 'headless'].includes(template)) {
        throw new Error('rhobuilder: `template` must be full or headless')
      }
      argv = [...base, 'new', target, '--design', design, '--template', template]
      argv.push(args.confirm === true ? '--apply' : '--plan')
      break
    }
    case 'project': {
      const action = requireEnum(args, 'action', ['status'])
      const project = requireString(args, 'project')
      argv = [...base, 'project', action, '--project', project]
      break
    }
    case 'check': {
      const project = requireString(args, 'project')
      const profile = args.profile ?? 'static'
      if (!['static', 'formal', 'full'].includes(profile)) {
        throw new Error('rhobuilder: `profile` must be static, formal or full')
      }
      argv = [...base, 'check', '--project', project, '--profile', profile]
      break
    }
    case 'plan': {
      const project = requireString(args, 'project')
      const network = requireString(args, 'network')
      argv = [...base, 'deploy', 'plan', '--project', project, '--network', network]
      break
    }
    case 'baseline': {
      const action = requireEnum(args, 'action', ['plan', 'verify', 'preflight'])
      const network = requireString(args, 'network')
      argv = [...base, 'baseline', action, '--network', network]
      if (action === 'preflight' && args.strict === true) argv.push('--strict')
      break
    }
    case 'formal': {
      const action = requireEnum(args, 'action', ['check', 'sync'])
      const project = requireString(args, 'project')
      argv = [...base, 'formal', action, '--project', project]
      if (action === 'sync') pushFlag(argv, '--source', args.source)
      break
    }
    case 'chain': {
      const action = requireEnum(args, 'action', ['probe', 'status', 'deploy-status'])
      const network = requireString(args, 'network')
      argv = [...base, 'chain', action, '--network', network]
      pushFlag(argv, '--deploy-id', args.deployId)
      break
    }
    default:
      throw new Error(`rhobuilder bridge: unknown tool ${tool}`)
  }
  return {
    command: argv[0],
    args: argv.slice(1),
    env: { ...process.env, PYTHONDONTWRITEBYTECODE: '1' },
    cwd: cfg.cwd,
  }
}

const PROSE = {
  Usage: 'I could not use those arguments.',
  OutsideWorkspace: 'That path is outside the workspace I am allowed to use.',
  IdentityMismatch: 'The node is not the network this plan expects.',
  NetworkGated: 'The node is not the network this profile expects.',
  ProfileError: 'The network profile could not be read.',
  NoEndpoint: 'The network profile has no endpoint to probe.',
  LiveExecutionGated: 'This is a live action and needs explicit approval, a funded key and the confirmation token.',
  ConfirmationMismatch: 'The confirmation token does not match this network or plan.',
  UnsafeKey: 'I refused to use that key; use a mode-0600 key file and never paste a key into chat.',
  UnsafeKeyPath: 'I refused a key path outside the allowed location.',
  MissingKey: 'The signing key is missing; generate one with the CLI first.',
  Keystore: 'The keystore could not be read or is unsafe.',
  MissingDependency: 'A shared primitive this app reuses is missing on the target network.',
  UnresolvedPlaceholder: 'A generated contract still contains an unresolved placeholder.',
  InstallError: 'The install plan could not be built.',
  InstallFailed: 'The install stopped before it completed.',
  PartialRoot: 'An existing root is incomplete. Stop and reconcile it; do not redeploy a commodity primitive.',
  Verification: 'Root verification did not prove every required key.',
  NodeUnavailable: 'The chain node is unreachable or unhealthy; this is not a pass.',
  BadResponse: 'The node returned an unexpected response.',
  Http: 'The node request failed.',
  CallFailed: 'The chain call failed.',
  FundingError: 'Funding did not complete; the node may be unhealthy or rate-limited.',
  GateFailure: 'A project gate failed.',
  GateError: 'A project gate could not run.',
  FormalError: 'The formal check could not complete.',
  DesignError: 'The design input is invalid.',
  RegistryError: 'The registry input is invalid or missing.',
  InstallPlanError: 'The install plan is invalid.',
  Io: 'A file could not be read or written.',
  Json: 'The input is not valid JSON.',
  ScaffoldError: 'The scaffold input is invalid.',
  ChainError: 'The chain operation failed.',
  PlanError: 'The deploy plan is invalid.',
  AppPlanError: 'The app deploy plan is invalid.',
  UnknownPrimitive: 'The design names a primitive that is not in the registry.',
  UnknownVertex: 'The design graph names a vertex that does not exist.',
}

const PROSE_BY_STATUS = {
  usage: 'I could not use those arguments.',
  identity_mismatch: 'The node is not the network this plan expects.',
  gated: 'This live action needs explicit approval, a funded key and the confirmation token.',
  unsafe_key: 'I refused to use that key; use a mode-0600 key file and never paste a key into chat.',
  missing_dependency: 'A shared primitive this app reuses is missing on the target network.',
  partial_root: 'An existing root is incomplete. Stop and reconcile it; do not redeploy a commodity primitive.',
  node_unavailable: 'The chain node is unreachable or unhealthy; this is not a pass.',
  gate_failure: 'A project gate failed.',
  setup: 'The local design, registry, project or install input needs fixing.',
}

function proseFor(type, status) {
  return PROSE[type] ?? PROSE_BY_STATUS[status] ?? 'The command failed.'
}

export function parseBridgeStdout(stdout, { acceptRefusal = false } = {}) {
  const text = String(stdout ?? '').trim()
  if (!text) throw new Error('rhobuilder bridge wrote no JSON')
  let document
  try {
    document = JSON.parse(text)
  } catch {
    throw new Error(`rhobuilder bridge wrote non-JSON: ${text.slice(-500)}`)
  }
  if (document && (document.kind === 'error' || document.ok === false)) {
    if (acceptRefusal) return document
    const type = document.error?.type ?? 'Error'
    const detail = document.error?.message ?? 'unknown refusal'
    const hint = document.error?.hint
    const status = document.error?.status
    const prose = proseFor(type, status)
    const error = new Error(`${prose} ${type}: ${detail}${hint ? ` — ${hint}` : ''}`)
    error.type = type
    error.status = status
    error.field = document.error?.field
    error.hint = hint
    error.prose = prose
    throw error
  }
  return document
}

export function runProcess(
  invocation,
  {
    timeoutMs = DEFAULT_TIMEOUT_MS,
    maxOutputBytes = DEFAULT_MAX_OUTPUT_BYTES,
    signal,
    acceptRefusal = false,
  } = {},
) {
  return new Promise((resolve, reject) => {
    let child
    const hasInput = invocation.input !== undefined && invocation.input !== null
    try {
      child = spawn(invocation.command, invocation.args, {
        cwd: invocation.cwd,
        env: invocation.env,
        stdio: [hasInput ? 'pipe' : 'ignore', 'pipe', 'pipe'],
      })
    } catch (error) {
      reject(error)
      return
    }
    if (hasInput) {
      // The child may exit before reading the whole draft. EPIPE is not a product
      // failure; the exit status and stdout still decide the result.
      child.stdin?.on('error', () => {})
      child.stdin?.end(String(invocation.input))
    }
    let stdout = ''
    let stderr = ''
    let settled = false
    const finish = (callback) => {
      if (settled) return
      settled = true
      clearTimeout(timer)
      if (signal) signal.removeEventListener('abort', onAbort)
      callback()
    }
    const kill = () => {
      try {
        child.kill('SIGKILL')
      } catch {
        // The close handler settles this.
      }
    }
    const timer = setTimeout(() => {
      kill()
      finish(() => reject(new Error(`rhobuilder bridge timed out after ${timeoutMs} ms`)))
    }, timeoutMs)
    const onAbort = () => {
      kill()
      finish(() => reject(new Error('rhobuilder bridge aborted')))
    }
    if (signal) {
      if (signal.aborted) {
        onAbort()
        return
      }
      signal.addEventListener('abort', onAbort, { once: true })
    }
    child.on('error', (error) => finish(() => reject(error)))
    child.stdout.on('data', (chunk) => {
      stdout += chunk
      if (Buffer.byteLength(stdout) > maxOutputBytes) {
        kill()
        finish(() =>
          reject(new Error(`rhobuilder bridge wrote more than ${maxOutputBytes} bytes`)),
        )
      }
    })
    child.stderr.on('data', (chunk) => {
      stderr += chunk
      if (stderr.length > 65_536) stderr = stderr.slice(-65_536)
    })
    child.on('close', (code) => {
      finish(() => {
        if (code !== 0 && !stdout.trim()) {
          reject(
            new Error(
              `rhobuilder bridge exited ${code}: ${stderr.trim().slice(-500) || 'no stderr'}`,
            ),
          )
          return
        }
        try {
          resolve(parseBridgeStdout(stdout, { acceptRefusal }))
        } catch (error) {
          reject(error)
        }
      })
    })
  })
}

export async function runBridge(tool, args, cfg, exec = {}) {
  const invocation = buildInvocation(tool, args, cfg)
  if (exec.input !== undefined) invocation.input = exec.input
  return runProcess(invocation, {
    timeoutMs: cfg.timeoutMs ?? DEFAULT_TIMEOUT_MS,
    maxOutputBytes: cfg.maxOutputBytes ?? DEFAULT_MAX_OUTPUT_BYTES,
    signal: exec.signal,
    acceptRefusal: exec.acceptRefusal === true,
  })
}

export async function selftest(cfg) {
  const version = await runBridge('version', {}, cfg)
  if (version.schema !== 'rhobuilder.version/v1' || version.ok !== true) {
    throw new Error('rhobuilder plugin: the bridge answered an unexpected envelope')
  }
  console.log(`rhobuilder plugin: ok (${version.version})`)
}

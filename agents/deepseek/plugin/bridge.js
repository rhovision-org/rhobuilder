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
      const path = requireString(args, 'path')
      argv = [...base, 'design', action, '--path', path]
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

export function parseBridgeStdout(stdout) {
  const text = String(stdout ?? '').trim()
  if (!text) throw new Error('rhobuilder bridge wrote no JSON')
  let document
  try {
    document = JSON.parse(text)
  } catch {
    throw new Error(`rhobuilder bridge wrote non-JSON: ${text.slice(-500)}`)
  }
  if (document && (document.kind === 'error' || document.ok === false)) {
    const error = new Error(
      `${document.error?.type ?? 'Error'}: ${document.error?.message ?? 'unknown refusal'}`,
    )
    error.type = document.error?.type
    error.field = document.error?.field
    throw error
  }
  return document
}

export function runProcess(
  invocation,
  { timeoutMs = DEFAULT_TIMEOUT_MS, maxOutputBytes = DEFAULT_MAX_OUTPUT_BYTES, signal } = {},
) {
  return new Promise((resolve, reject) => {
    let child
    try {
      child = spawn(invocation.command, invocation.args, {
        cwd: invocation.cwd,
        env: invocation.env,
        stdio: ['ignore', 'pipe', 'pipe'],
      })
    } catch (error) {
      reject(error)
      return
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
          resolve(parseBridgeStdout(stdout))
        } catch (error) {
          reject(error)
        }
      })
    })
  })
}

export async function runBridge(tool, args, cfg, exec = {}) {
  const invocation = buildInvocation(tool, args, cfg)
  return runProcess(invocation, {
    timeoutMs: cfg.timeoutMs ?? DEFAULT_TIMEOUT_MS,
    maxOutputBytes: cfg.maxOutputBytes ?? DEFAULT_MAX_OUTPUT_BYTES,
    signal: exec.signal,
  })
}

export async function selftest(cfg) {
  const version = await runBridge('version', {}, cfg)
  if (version.schema !== 'rhobuilder.version/v1' || version.ok !== true) {
    throw new Error('rhobuilder plugin: the bridge answered an unexpected envelope')
  }
  console.log(`rhobuilder plugin: ok (${version.version})`)
}

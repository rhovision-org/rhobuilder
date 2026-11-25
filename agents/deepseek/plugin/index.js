// The rhobuilder DSH plugin: seven tools over `rhobuilder --json`.
//
// It imports nothing from @deepseek-ai/*. DSH's ToolsService.register accepts a plain
// ToolDefinition (parameters, output.schema, output.render, execute), so the plugin mounts
// by absolute path with no node_modules. Every decision stays in the Rust CLI; this file
// validates the model's arguments and renders the envelope.

import { runBridge } from './bridge.js'

export const name = 'rhobuilder-tools'
export const inject = ['tools']

const MAX_RENDER_BYTES = 65_536
const OUTPUT_SCHEMA = { type: 'object', additionalProperties: true }

function content(text) {
  const value = String(text)
  if (Buffer.byteLength(value) <= MAX_RENDER_BYTES) {
    return [{ type: 'text', text: value }]
  }
  return [
    {
      type: 'text',
      text: `${value.slice(0, MAX_RENDER_BYTES)}\n... truncated; narrow the query`,
    },
  ]
}

function renderJson(_args, value) {
  try {
    return content(JSON.stringify(value, null, 2))
  } catch {
    return content(String(value))
  }
}

function renderRegistry(_args, value) {
  try {
    const items = Array.isArray(value?.items) ? value.items : []
    const lines = [`registry ${value?.action ?? '?'}: ${items.length} item(s)`]
    for (const item of items.slice(0, 50)) {
      const owner = item?.owner ?? '?'
      const status = item?.status ?? '?'
      const id = item?.id ?? '?'
      lines.push(`- ${id} (${owner}, ${status})`)
    }
    if (items.length > 50) lines.push(`... ${items.length - 50} more`)
    return content(lines.join('\n'))
  } catch {
    return renderJson(_args, value)
  }
}

function renderCheck(_args, value) {
  try {
    const gates = Array.isArray(value?.gates) ? value.gates : []
    const failed = gates.filter((gate) => gate?.status === 'fail')
    const notRun = gates.filter((gate) => gate?.status === 'not_run')
    const owed = Array.isArray(value?.owed) ? value.owed : []
    const lines = [`check ${value?.profile ?? '?'}: ${value?.ok ? 'ok' : 'failed'}`]
    for (const gate of gates) lines.push(`- ${gate?.name ?? '?'}: ${gate?.status ?? '?'}`)
    if (failed.length > 0) lines.push(`${failed.length} gate(s) failed`)
    if (notRun.length > 0) lines.push(`${notRun.length} gate(s) not run; not run is not a pass`)
    if (owed.length > 0) lines.push(`owed: ${owed.join(', ')}`)
    return content(lines.join('\n'))
  } catch {
    return renderJson(_args, value)
  }
}

function renderStatus(_args, value) {
  try {
    const badges = Array.isArray(value?.badges) ? value.badges : []
    const lines = [
      `project ${value?.project?.slug ?? '?'}: ${
        value?.design?.ok ? 'design ok' : 'design refused or unchecked'
      }`,
    ]
    for (const badge of badges) {
      lines.push(`- ${badge?.label ?? '?'}: ${badge?.state ?? '?'} — ${badge?.detail ?? ''}`)
    }
    const owed = Array.isArray(value?.owed) ? value.owed : []
    if (owed.length > 0) lines.push(`owed: ${owed.join(', ')}`)
    return content(lines.join('\n'))
  } catch {
    return renderJson(_args, value)
  }
}

function renderPlan(_args, value) {
  try {
    const steps = value?.preview?.steps ?? value?.deploys ?? []
    const list = Array.isArray(steps) ? steps : []
    const lines = [`deploy plan ${value?.network ?? '?'}: ${list.length} step(s)`]
    for (const step of list) {
      lines.push(`- ${step?.name ?? '?'}: ${step?.status ?? step?.action ?? '?'}`)
    }
    if (value?.planDigest) lines.push(`planDigest: ${value.planDigest}`)
    return content(lines.join('\n'))
  } catch {
    return renderJson(_args, value)
  }
}

function tool({ name: toolName, description, parameters, execute, render = renderJson, timeoutMs }) {
  return {
    name: toolName,
    description,
    parameters,
    output: { schema: OUTPUT_SCHEMA, render },
    timeoutMs,
    async execute(args, exec) {
      return execute(args, exec)
    },
  }
}

function requireString(args, field) {
  if (typeof args?.[field] !== 'string' || args[field].length === 0) {
    throw new Error(`rhobuilder: \`${field}\` must be a non-empty string`)
  }
}

function requireAction(args, allowed) {
  requireString(args, 'action')
  if (!allowed.includes(args.action)) {
    throw new Error(`rhobuilder: \`action\` must be one of ${allowed.join(', ')}`)
  }
}

export function apply(ctx, config = {}) {
  if (typeof config.command !== 'string' || config.command.length === 0) {
    throw new Error('rhobuilder-tools: config.command is required')
  }
  const cfg = {
    command: config.command,
    args: Array.isArray(config.args) ? config.args : [],
    cwd: config.cwd,
    timeoutMs: config.timeoutMs ?? 120_000,
    maxOutputBytes: config.maxOutputBytes ?? 4_194_304,
    registry: config.registry,
  }

  ctx.tools.register(
    tool({
      name: 'rhobuilder_registry',
      description:
        'Query the rhobuilder on-chain primitive registry: list, get one primitive, show the reuse graph, verify catalog claims for a network, or check pinned-commit drift against sibling checkouts. Use it before designing a new component.',
      parameters: {
        type: 'object',
        properties: {
          action: { type: 'string', enum: ['list', 'get', 'graph', 'verify', 'drift'] },
          network: { type: 'string', description: 'devnet, testnet or mainnet' },
          id: { type: 'string', description: 'primitive id for action=get' },
          owner: { type: 'string', description: 'owner filter for action=list' },
          kind: { type: 'string', description: 'kind filter for action=list' },
          root: { type: 'string', description: 'root vertex for action=graph' },
          depth: { type: 'number', description: 'graph depth, 1..8' },
          sourceRoot: { type: 'string', description: 'sibling checkout root for action=drift' },
          strict: { type: 'boolean', description: 'fail on an unavailable pin for action=drift' },
        },
        required: ['action'],
        additionalProperties: false,
      },
      timeoutMs: cfg.timeoutMs,
      execute: (args, exec) => {
        requireAction(args, ['list', 'get', 'graph', 'verify', 'drift'])
        if (args.action === 'get') requireString(args, 'id')
        if (args.action === 'verify') requireString(args, 'network')
        if (args.action === 'graph') requireString(args, 'root')
        if (args.action === 'drift') requireString(args, 'sourceRoot')
        return runBridge('registry', args, cfg, exec)
      },
      render: renderRegistry,
    }),
  )

  ctx.tools.register(
    tool({
      name: 'rhobuilder_design',
      description:
        'Validate or report on a rhobuilder design.json. The report tags each component reuse/extend/new and refuses duplicate commodity primitives.',
      parameters: {
        type: 'object',
        properties: {
          action: { type: 'string', enum: ['validate', 'report'] },
          path: { type: 'string', description: 'path to design.json' },
        },
        required: ['action', 'path'],
        additionalProperties: false,
      },
      timeoutMs: cfg.timeoutMs,
      execute: (args, exec) => {
        requireAction(args, ['validate', 'report'])
        requireString(args, 'path')
        return runBridge('design', args, cfg, { ...exec, acceptRefusal: true })
      },
    }),
  )

  ctx.tools.register(
    tool({
      name: 'rhobuilder_scaffold',
      description:
        'Plan or apply a new rhovision dApp skeleton from design.json. Defaults to a plan; apply requires confirm=true and writes into the target directory.',
      parameters: {
        type: 'object',
        properties: {
          target: { type: 'string', description: 'target directory for the new app' },
          design: { type: 'string', description: 'path to design.json' },
          template: { type: 'string', enum: ['full', 'headless'] },
          confirm: { type: 'boolean', description: 'true applies; false or absent plans' },
        },
        required: ['target', 'design'],
        additionalProperties: false,
      },
      timeoutMs: cfg.timeoutMs,
      execute: (args, exec) => {
        requireString(args, 'target')
        requireString(args, 'design')
        return runBridge('scaffold', args, cfg, exec)
      },
    }),
  )

  ctx.tools.register(
    tool({
      name: 'rhobuilder_check',
      description:
        'Run the generated project gates: static (build/parse/docs/spec), formal (Lean), or full (adds devnet harness).',
      parameters: {
        type: 'object',
        properties: {
          project: { type: 'string', description: 'project directory' },
          profile: { type: 'string', enum: ['static', 'formal', 'full'] },
        },
        required: ['project'],
        additionalProperties: false,
      },
      timeoutMs: cfg.timeoutMs,
      execute: (args, exec) => {
        requireString(args, 'project')
        return runBridge('check', args, cfg, exec)
      },
      render: renderCheck,
    }),
  )

  ctx.tools.register(
    tool({
      name: 'rhobuilder_status',
      description:
        'Report a project read-only: design/scaffold completeness, recorded tests, Lean model obligations, owed arms and deployment records. It never runs make, Docker or a network call.',
      parameters: {
        type: 'object',
        properties: {
          project: { type: 'string', description: 'project directory' },
        },
        required: ['project'],
        additionalProperties: false,
      },
      timeoutMs: cfg.timeoutMs,
      execute: (args, exec) => {
        requireString(args, 'project')
        return runBridge('project', { action: 'status', project: args.project }, cfg, exec)
      },
      render: renderStatus,
    }),
  )

  ctx.tools.register(
    tool({
      name: 'rhobuilder_plan',
      description:
        'Produce a read-only deploy plan for a project and network. It reuses registered primitives and refuses duplicates, missing dependencies, identity mismatch or mainnet. It never signs.',
      parameters: {
        type: 'object',
        properties: {
          project: { type: 'string', description: 'project directory' },
          network: { type: 'string', enum: ['devnet', 'testnet', 'mainnet'] },
        },
        required: ['project', 'network'],
        additionalProperties: false,
      },
      timeoutMs: cfg.timeoutMs,
      execute: (args, exec) => {
        requireString(args, 'project')
        requireString(args, 'network')
        return runBridge('plan', args, cfg, exec)
      },
      render: renderPlan,
    }),
  )

  ctx.tools.register(
    tool({
      name: 'rhobuilder_chain',
      description:
        'Read-only chain access: probe identity and capabilities, read the raw node status, or read a deploy status. It never submits a signed deploy.',
      parameters: {
        type: 'object',
        properties: {
          action: { type: 'string', enum: ['probe', 'status', 'deploy-status'] },
          network: { type: 'string', enum: ['devnet', 'testnet', 'mainnet'] },
          deployId: { type: 'string', description: 'deploy id for action=deploy-status' },
        },
        required: ['action', 'network'],
        additionalProperties: false,
      },
      timeoutMs: cfg.timeoutMs,
      execute: (args, exec) => {
        requireAction(args, ['probe', 'status', 'deploy-status'])
        requireString(args, 'network')
        if (args.action === 'deploy-status') requireString(args, 'deployId')
        return runBridge('chain', args, cfg, exec)
      },
    }),
  )
}

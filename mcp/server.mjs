// Read-only MCP server for rhobuilder. Dependency-free; stdio with newline-delimited JSON.
//
// It reuses the DSH plugin's bridge (`agents/deepseek/plugin/bridge.js`) and exposes only
// read-only/plan tools. Signed deploy execution stays in the CLI.
//
// Lanes:
//   2025-11-25  initialize -> notifications/initialized -> tools/list -> tools/call
//   2026-07-28  server/discover, per-request _meta, resultType on every result
//
// Usage:
//   node mcp/server.mjs --binary /path/to/rhobuilder
//   node mcp/server.mjs --binary cargo run -q -p rhobuilder-cli --

import { createInterface } from 'node:readline'
import { pathToFileURL } from 'node:url'

import { runBridge } from '../agents/deepseek/plugin/bridge.js'

const SERVER_NAME = 'rhobuilder'
const SERVER_VERSION = '0.1.0'
const PROTOCOL_LEGACY = '2025-11-25'
const PROTOCOL_MODERN = '2026-07-28'

function schema(properties, required = []) {
  return {
    type: 'object',
    properties,
    required,
    additionalProperties: false,
  }
}

export const TOOLS = [
  {
    name: 'rhobuilder_registry_list',
    description: 'List registered primitives, optionally filtered by network, owner or kind.',
    inputSchema: schema({
      network: { type: 'string' },
      owner: { type: 'string' },
      kind: { type: 'string' },
    }),
    tool: 'registry',
    args: (args) => ({ action: 'list', ...args }),
  },
  {
    name: 'rhobuilder_registry_get',
    description: 'Show one registered primitive by id.',
    inputSchema: schema({ id: { type: 'string' } }, ['id']),
    tool: 'registry',
    args: (args) => ({ action: 'get', ...args }),
  },
  {
    name: 'rhobuilder_registry_graph',
    description: 'Show the reuse graph around a vertex.',
    inputSchema: schema(
      { root: { type: 'string' }, depth: { type: 'number' } },
      ['root'],
    ),
    tool: 'registry',
    args: (args) => ({ action: 'graph', ...args }),
  },
  {
    name: 'rhobuilder_registry_drift',
    description: 'Check pinned-commit drift against sibling checkouts.',
    inputSchema: schema(
      { sourceRoot: { type: 'string' }, strict: { type: 'boolean' } },
      ['sourceRoot'],
    ),
    tool: 'registry',
    args: (args) => ({ action: 'drift', ...args }),
  },
  {
    name: 'rhobuilder_design_check',
    description: 'Validate a design.json and run the reuse gate.',
    inputSchema: schema({ path: { type: 'string' } }, ['path']),
    tool: 'design',
    args: (args) => ({ action: 'validate', ...args }),
  },
  {
    name: 'rhobuilder_scaffold_plan',
    description: 'Plan a new dApp skeleton; writes nothing.',
    inputSchema: schema(
      {
        target: { type: 'string' },
        design: { type: 'string' },
        template: { type: 'string', enum: ['full', 'headless'] },
      },
      ['target', 'design'],
    ),
    tool: 'scaffold',
    args: (args) => ({ confirm: false, ...args }),
  },
  {
    name: 'rhobuilder_chain_probe',
    description: 'Probe network identity and capabilities; read-only, no exploratory deploy.',
    inputSchema: schema({ network: { type: 'string' } }, ['network']),
    tool: 'chain',
    args: (args) => ({ action: 'probe', ...args }),
  },
  {
    name: 'rhobuilder_deploy_plan',
    description:
      'Read-only deploy plan for a generated project. It never signs and refuses missing dependencies or identity mismatch.',
    inputSchema: schema(
      {
        project: { type: 'string' },
        network: { type: 'string', enum: ['devnet', 'testnet', 'mainnet'] },
      },
      ['project', 'network'],
    ),
    tool: 'plan',
    args: (args) => ({ project: args.project, network: args.network }),
  },
  {
    name: 'rhobuilder_check',
    description:
      'Run the generated project gates through its Makefile: static, formal or full.',
    inputSchema: schema(
      {
        project: { type: 'string' },
        profile: { type: 'string', enum: ['static', 'formal', 'full'] },
      },
      ['project'],
    ),
    tool: 'check',
    args: (args) => ({ project: args.project, profile: args.profile ?? 'static' }),
  },
  {
    name: 'rhobuilder_formal_check',
    description: 'Verify the vendored RhoApp tree hash in a project.',
    inputSchema: schema({ project: { type: 'string' } }, ['project']),
    tool: 'formal',
    args: (args) => ({ action: 'check', ...args }),
  },
]

export function parseArgs(argv) {
  const index = argv.indexOf('--binary')
  if (index < 0 || !argv[index + 1]) {
    throw new Error('mcp: --binary <command> is required')
  }
  return {
    command: argv[index + 1],
    args: argv.slice(index + 2),
    cwd: process.cwd(),
    timeoutMs: 120_000,
    maxOutputBytes: 4_194_304,
  }
}

export function buildToolInvocation(name, args, cfg) {
  const tool = TOOLS.find((candidate) => candidate.name === name)
  if (!tool) throw new Error(`mcp: unknown tool ${name}`)
  return { tool: tool.tool, args: tool.args(args ?? {}) }
}

function modern(message) {
  if (message?.method === 'server/discover') return true
  const version = message?._meta?.['io.modelcontextprotocol/protocolVersion']
  return typeof version === 'string' && version === PROTOCOL_MODERN
}

function result(id, value, isModern) {
  const response = { jsonrpc: '2.0', id, result: value }
  if (isModern) {
    response.result = {
      ...value,
      resultType: value.resultType ?? 'complete',
      _meta: {
        ...(value._meta ?? {}),
        'io.modelcontextprotocol/protocolVersion': PROTOCOL_MODERN,
      },
    }
  }
  return response
}

function error(id, code, message) {
  return { jsonrpc: '2.0', id, error: { code, message } }
}

function toolList() {
  return TOOLS.map(({ name, description, inputSchema }) => ({
    name,
    description,
    inputSchema,
  }))
}

function serverInfo() {
  return { name: SERVER_NAME, version: SERVER_VERSION }
}

function dispatch(message, cfg) {
  const method = message?.method
  const isModern = modern(message)
  if (method === 'initialize') {
    return {
      protocolVersion: PROTOCOL_LEGACY,
      capabilities: { tools: {} },
      serverInfo: serverInfo(),
    }
  }
  if (method === 'server/discover') {
    return {
      protocolVersion: PROTOCOL_MODERN,
      capabilities: { tools: {} },
      serverInfo: serverInfo(),
    }
  }
  if (method === 'ping') return {}
  if (method === 'tools/list') return { tools: toolList() }
  if (method === 'tools/call') {
    const name = message.params?.name
    const args = message.params?.arguments ?? {}
    const invocation = buildToolInvocation(name, args, cfg)
    return { __call: invocation }
  }
  return null
}

export async function handle(message, cfg) {
  if (!message || message.jsonrpc !== '2.0') {
    return error(message?.id ?? null, -32600, 'invalid request')
  }
  const id = message.id
  if (id === undefined) {
    // Notification: no response.
    return null
  }
  const isModern = modern(message)
  if (message.method === 'tools/call') {
    try {
      const invocation = buildToolInvocation(
        message.params?.name,
        message.params?.arguments ?? {},
        cfg,
      )
      const value = await runBridge(invocation.tool, invocation.args, cfg, {})
      return result(
        id,
        {
          content: [{ type: 'text', text: JSON.stringify(value, null, 2) }],
          isError: false,
        },
        isModern,
      )
    } catch (err) {
      return result(
        id,
        {
          content: [{ type: 'text', text: String(err?.message ?? err) }],
          isError: true,
        },
        isModern,
      )
    }
  }
  const value = dispatch(message, cfg)
  if (value === null) {
    return error(id, -32601, `method not found: ${message.method}`)
  }
  if (value.__call) {
    return error(id, -32603, 'internal dispatch error')
  }
  return result(id, value, isModern)
}

export async function serve(input, output, cfg) {
  const readline = createInterface({ input })
  for await (const line of readline) {
    const text = line.trim()
    if (!text) continue
    let message
    try {
      message = JSON.parse(text)
    } catch {
      output.write(JSON.stringify(error(null, -32700, 'parse error')) + '\n')
      continue
    }
    const response = await handle(message, cfg)
    if (response) output.write(JSON.stringify(response) + '\n')
  }
}

const isMain =
  process.argv[1] &&
  import.meta.url === pathToFileURL(process.argv[1]).href

if (isMain) {
  let cfg
  try {
    cfg = parseArgs(process.argv.slice(2))
  } catch (error) {
    console.error(error.message)
    process.exit(2)
  }
  serve(process.stdin, process.stdout, cfg).catch((error) => {
    console.error(`mcp: ${error?.message ?? error}`)
    process.exit(1)
  })
}

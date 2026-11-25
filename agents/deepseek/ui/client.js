// Rhobuilder browser half. Hand-written factory-CJS for the DSH 0.2 client loader.
// No ESM imports, no JSX: React comes from the shell's frozen module table.

let React

function h(type, props, ...children) {
  return React.createElement(type, props ?? null, ...children)
}

const BRAND = 'var(--rb-brand, #0f766e)'
const INK = 'var(--rb-ink, #0b1220)'
const PAPER = 'var(--rb-paper, #f7f5ef)'
const MUTED = 'var(--dsw-alias-label-secondary, #5b6472)'
const BORDER = 'var(--dsw-alias-border-l1, #d8d5cc)'
const SOFT = 'var(--dsw-alias-bg-layer-1, #ffffff)'

const CARD = {
  border: `1px solid ${BORDER}`,
  borderRadius: 12,
  background: 'var(--dsw-alias-bg-base, #ffffff)',
  color: 'var(--dsw-alias-label-primary, #1f2328)',
  padding: '18px 20px',
  font: '14px/1.55 system-ui, sans-serif',
}

const MUTED_TEXT = { color: MUTED, fontSize: 13 }
const CODE = {
  margin: '8px 0 0',
  padding: '10px 12px',
  borderRadius: 8,
  background: 'var(--dsw-alias-bg-layer-2, #f3f1eb)',
  color: 'var(--dsw-alias-label-primary, #1f2328)',
  font: '12px/1.5 ui-monospace, SFMono-Regular, Menlo, monospace',
  overflowX: 'auto',
  whiteSpace: 'pre-wrap',
}

const STARTERS = [
  'Interview me and create a design.json for a bookable service.',
  'Show me the primitive graph and what I can reuse.',
  'Review my project and tell me what is scaffold vs proved.',
  'Check devnet readiness and what I need to install.',
]

function Badge({ tone = 'neutral', children }) {
  const colours = {
    ok: ['#0f766e', 'rgba(15,118,110,0.12)'],
    warn: ['#b45309', 'rgba(180,83,9,0.12)'],
    fail: ['#b42318', 'rgba(180,35,24,0.12)'],
    neutral: [MUTED, 'var(--dsw-alias-bg-layer-2, #f3f1eb)'],
  }
  const [colour, background] = colours[tone] ?? colours.neutral
  return h(
    'span',
    {
      style: {
        display: 'inline-flex',
        alignItems: 'center',
        gap: 5,
        padding: '3px 9px',
        borderRadius: 999,
        border: `1px solid ${BORDER}`,
        background,
        color: colour,
        font: '12px/1.3 system-ui, sans-serif',
        whiteSpace: 'nowrap',
      },
    },
    children,
  )
}

function BrandMark({ size = 26 }) {
  return h('img', {
    src: '/rhobuilder-ui/rhobuilder-mark.svg',
    alt: 'rhobuilder',
    width: size,
    height: size,
    style: { display: 'block', borderRadius: Math.round(size * 0.24) },
  })
}

function Card({ title, subtitle, children, aside }) {
  return h(
    'section',
    { style: { ...CARD, margin: '0 0 14px' } },
    (title || aside) &&
      h(
        'div',
        {
          style: {
            display: 'flex',
            justifyContent: 'space-between',
            alignItems: 'baseline',
            gap: 12,
          },
        },
        h(
          'div',
          null,
          title && h('h2', { style: { margin: 0, fontSize: 15 } }, title),
          subtitle && h('p', { style: { ...MUTED_TEXT, margin: '3px 0 0' } }, subtitle),
        ),
        aside,
      ),
    children,
  )
}

function Wordmark({ size = 30, compact = false }) {
  return h(
    'div',
    { style: { display: 'flex', alignItems: 'center', gap: 10 } },
    h('img', {
      src: '/rhobuilder-ui/rhobuilder-mark.svg',
      alt: '',
      width: size,
      height: size,
      style: { display: 'block', borderRadius: Math.round(size * 0.24) },
    }),
    h(
      'div',
      { style: { lineHeight: 1.1 } },
      h('div', { style: { fontSize: compact ? 14 : 17, fontWeight: 700, color: INK } }, 'rhobuilder'),
      !compact &&
        h(
          'div',
          { style: { fontSize: 11, color: MUTED, marginTop: 2, textTransform: 'uppercase', letterSpacing: '0.06em' } },
          'rholo dApp studio',
        ),
    ),
  )
}

function Hero({ hero, children }) {
  if (!hero) return children
  return h(
    'div',
    { style: { display: 'contents' } },
    h(
      'section',
      {
        'aria-label': 'rhobuilder',
        style: {
          ...CARD,
          maxWidth: 760,
          margin: '0 auto 18px',
          borderLeft: `4px solid ${BRAND}`,
          background: 'var(--dsw-alias-bg-elevated, #fbfaf7)',
          color: INK,
        },
      },
      h(Wordmark, { size: 40 }),
      h(
        'p',
        { style: { margin: '14px 0 0', fontSize: 15, lineHeight: 1.55 } },
        'Design the unique part of a rhovision dApp, reuse the on-chain primitive graph, then scaffold, prove, test and plan a deploy.',
      ),
      h(
        'ol',
        { style: { margin: '10px 0 0', paddingLeft: 20, color: MUTED } },
        h('li', null, 'Design only what is genuinely new.'),
        h('li', null, 'Reuse r-cash, r-sure, r-gen, r-share and r-print by rho:id.'),
        h('li', null, 'Scaffold, prove, test and plan. You sign; this UI never holds a key.'),
      ),
      h(
        'div',
        { style: { display: 'flex', flexWrap: 'wrap', gap: 8, marginTop: 13 } },
        h(Badge, { tone: 'ok' }, 'P5 devnet measured'),
        h(Badge, { tone: 'warn' }, 'P6 testnet gated'),
        h(Badge, { tone: 'neutral' }, 'Mainnet refused'),
        h(Badge, { tone: 'neutral' }, 'Read-only studio'),
      ),
      h(
        'p',
        { style: { ...MUTED_TEXT, margin: '12px 0 0' } },
        'The stock DeepSeek Harness chat is still underneath. Use the Rhobuilder panel in the sidebar for readiness, networks, primitives and examples.',
      ),
    ),
    children,
  )
}

function Starters({ inputActions }) {
  const ready = Boolean(inputActions && inputActions.setDraft && inputActions.submit)
  return h(
    'div',
    { style: { maxWidth: 760, margin: '0 auto' } },
    h('p', { style: { ...MUTED_TEXT, margin: '0 0 6px' } }, 'Start here'),
    h(
      'div',
      { style: { display: 'flex', flexWrap: 'wrap', gap: 8 } },
      STARTERS.map((text) =>
        h(
          'button',
          {
            key: text,
            type: 'button',
            disabled: !ready,
            style: {
              padding: '8px 12px',
              borderRadius: 999,
              border: `1px solid ${BORDER}`,
              background: SOFT,
              color: INK,
              cursor: ready ? 'pointer' : 'default',
              font: '13px/1.35 system-ui, sans-serif',
              textAlign: 'left',
            },
            onClick: () => {
              if (!ready) return
              inputActions.setDraft(text)
              inputActions.submit()
            },
          },
          text,
        ),
      ),
    ),
  )
}

function useApi(path, initial) {
  const [value, setValue] = React.useState(initial)
  const [error, setError] = React.useState(null)
  React.useEffect(() => {
    let live = true
    fetch(path, { credentials: 'same-origin' })
      .then((response) => {
        if (!response.ok) throw new Error(`HTTP ${response.status}; open the harness with its token link`)
        return response.json()
      })
      .then((data) => live && setValue(data))
      .catch((exc) => live && setError(String(exc?.message ?? exc)))
    return () => {
      live = false
    }
  }, [path])
  return { value, error }
}

function statusTone(status) {
  if (status === 'ok' || status === 'pass') return 'ok'
  if (status === 'fail') return 'fail'
  if (status === 'warn' || status === 'setup' || status === 'not_run') return 'warn'
  return 'neutral'
}

function Readiness({ readiness }) {
  const checks = Array.isArray(readiness.value?.checks) ? readiness.value.checks : []
  const setup = Array.isArray(readiness.value?.setupRequired) ? readiness.value.setupRequired : []
  return h(
    Card,
    {
      title: 'Readiness',
      subtitle: 'What this machine can do now. Missing setup is shown, never hidden as a pass.',
      aside: h(Badge, { tone: readiness.value?.ok ? 'ok' : 'warn' }, readiness.value?.ok ? 'Ready' : 'Setup required'),
    },
    readiness.error && h('p', { style: { color: '#b42318' } }, readiness.error),
    checks.length === 0 &&
      h('p', { style: MUTED_TEXT }, readiness.value?.note || 'Readiness has not been checked yet.'),
    h(
      'div',
      { style: { display: 'grid', gap: 7, marginTop: 8 } },
      checks.map((check) =>
        h(
          'div',
          {
            key: check.name,
            style: { display: 'grid', gridTemplateColumns: '110px 90px 1fr', gap: 8, alignItems: 'baseline' },
          },
          h('strong', { style: { fontSize: 13 } }, check.name),
          h(Badge, { tone: statusTone(check.status) }, check.status),
          h('span', { style: MUTED_TEXT }, check.detail),
        ),
      ),
    ),
    setup.length > 0 &&
      h(
        'p',
        { style: { ...MUTED_TEXT, marginTop: 10 } },
        `Blocking setup: ${setup.join(', ')}. Open the setup instructions in the CLI handoff below.`,
      ),
  )
}

function NetworkCard({ name, profile }) {
  const status = profile?.status ?? 'unknown'
  const deploy = profile?.deploy ?? 'unknown'
  const tone = name === 'mainnet' ? 'fail' : deploy === 'gated' ? 'warn' : 'ok'
  return h(
    Card,
    {
      title: name === 'devnet' ? 'Devnet' : name === 'testnet' ? 'Public testnet' : 'Mainnet',
      aside: h(Badge, { tone }, deploy),
    },
    h('p', { style: { margin: 0 } }, status === 'local' ? 'Local, disposable' : status),
    profile?.networkId && h('p', { style: MUTED_TEXT }, `networkId: ${profile.networkId}`),
    profile?.shardId && h('p', { style: MUTED_TEXT }, `shard: ${profile.shardId}`),
    profile?.observed?.height && h('p', { style: MUTED_TEXT }, `observed height: ${profile.observed.height}`),
    Object.keys(profile?.roots ?? {}).length > 0 &&
      h(
        'p',
        { style: { ...MUTED_TEXT, marginTop: 6 } },
        `${Object.keys(profile.roots).length} recorded app root(s)`,
      ),
  )
}

function Domains({ primitives }) {
  const owners = new Map()
  for (const primitive of primitives) {
    const key = primitive.owner || 'app'
    owners.set(key, (owners.get(key) ?? 0) + 1)
  }
  return h(
    'div',
    { style: { display: 'grid', gap: 6, marginTop: 8 } },
    [...owners.entries()].sort().map(([owner, count]) =>
      h(
        'div',
        { key: owner, style: { display: 'flex', gap: 10, justifyContent: 'space-between' } },
        h('strong', { style: { fontSize: 13 } }, owner),
        h('span', { style: MUTED_TEXT }, `${count} primitive(s)`),
      ),
    ),
  )
}

function copyText(text) {
  try {
    if (typeof navigator !== 'undefined' && navigator?.clipboard?.writeText) {
      void navigator.clipboard.writeText(text)
    }
  } catch {
    // The command is still visible in the card.
  }
}

const BUTTON = {
  marginTop: 8,
  padding: '5px 10px',
  borderRadius: 7,
  border: `1px solid ${BORDER}`,
  background: SOFT,
  color: INK,
  cursor: 'pointer',
  font: '12px/1.3 system-ui, sans-serif',
}

function ExampleCard({ example }) {
  const [plan, setPlan] = React.useState({ status: 'idle' })
  const planScaffold = () => {
    setPlan({ status: 'loading' })
    fetch(
      `/rhobuilder/api/scaffold-plan?design=${encodeURIComponent(example.file)}&template=headless`,
      { credentials: 'same-origin' },
    )
      .then((response) => {
        if (!response.ok) throw new Error(`HTTP ${response.status}`)
        return response.json()
      })
      .then((value) => setPlan({ status: 'ok', count: value.count }))
      .catch((error) => setPlan({ status: 'error', message: String(error?.message ?? error) }))
  }
  return h(
    'div',
    { style: { borderTop: `1px solid ${BORDER}`, paddingTop: 9 } },
    h('strong', null, example.name),
    h('p', { style: { ...MUTED_TEXT, margin: '3px 0 6px' } }, example.summary),
    h(
      'div',
      { style: { display: 'flex', gap: 6, flexWrap: 'wrap' } },
      (example.tags ?? []).map((tag) =>
        h(Badge, { key: tag.id, tone: tag.tag === 'new' ? 'warn' : 'neutral' }, `${tag.id}: ${tag.tag}`),
      ),
    ),
    h(
      'div',
      { style: { display: 'flex', gap: 7, flexWrap: 'wrap' } },
      h('button', { type: 'button', style: BUTTON, onClick: planScaffold }, 'Plan scaffold (no writes)'),
      h(
        'button',
        {
          type: 'button',
          style: BUTTON,
          onClick: () =>
            copyText(
              `Review ${example.file} with rhobuilder-dapp-design and rhobuilder-primitive-reuse. Report reuse/extend/new, then propose the scaffold plan. Do not apply without my explicit confirmation.`,
            ),
        },
        'Copy review prompt',
      ),
    ),
    plan.status === 'ok' &&
      h('p', { style: { ...MUTED_TEXT, margin: '7px 0 0' } }, `Plan: ${plan.count} files; no files written.`),
    plan.status === 'loading' && h('p', { style: MUTED_TEXT }, 'Planning…'),
    plan.status === 'error' && h('p', { style: { color: '#b42318' } }, `Plan failed: ${plan.message}`),
  )
}

function Examples({ examples }) {
  const rows = Array.isArray(examples.value?.examples) ? examples.value.examples : []
  return h(
    Card,
    {
      title: 'Examples',
      subtitle: 'Plan a worked design before asking the chat to apply it.',
    },
    rows.length === 0 && h('p', { style: MUTED_TEXT }, 'Examples are unavailable in this installation.'),
    h(
      'div',
      { style: { display: 'grid', gap: 10, marginTop: 8 } },
      rows.map((example) => h(ExampleCard, { key: example.file, example })),
    ),
  )
}

function CliHandoff() {
  return h(
    Card,
    {
      title: 'CLI handoff',
      subtitle: 'Signed, funded and live actions stay in the terminal. The UI never holds a key.',
    },
    h('p', { style: { margin: '8px 0 0' } }, 'Create and fund a devnet ops key:'),
    h(
      'pre',
      { style: CODE },
      'rhobuilder --json baseline keygen --network devnet --out .ops/devnet/keys.json\n' +
        'rhobuilder --json baseline fund --network devnet --keys .ops/devnet/keys.json --execute',
    ),
    h('p', { style: { margin: '10px 0 0' } }, 'Plan, then execute an app deploy (devnet):'),
    h(
      'pre',
      { style: CODE },
      'rhobuilder --json deploy plan --project <dir> --network devnet\n' +
        'rhobuilder --json deploy execute --project <dir> --network devnet --execute --keys .ops/devnet/keys.json',
    ),
    h('p', { style: { margin: '10px 0 0' } }, 'Testnet execution is gated:'),
    h(
      'pre',
      { style: CODE },
      'rhobuilder --json baseline run --network testnet --dry-run\n' +
        '# execute needs --allow-live, a funded key, and the identity+plan --confirm token',
    ),
  )
}

function Dashboard() {
  const summary = useApi('/rhobuilder/api/summary', { networks: {} })
  const readiness = useApi('/rhobuilder/api/readiness', { checks: [] })
  const registry = useApi('/rhobuilder/api/registry', { primitives: [], vertices: [], edges: [] })
  const examples = useApi('/rhobuilder/api/examples', { examples: [] })
  const primitives = Array.isArray(registry.value?.primitives) ? registry.value.primitives : []
  const networks = summary.value?.networks ?? {}
  return h(
    'div',
    { style: { maxWidth: 900, margin: '0 auto', padding: '22px 18px', font: '14px/1.5 system-ui, sans-serif' } },
    h(
      'div',
      { style: { display: 'flex', justifyContent: 'space-between', gap: 12, alignItems: 'center', marginBottom: 16 } },
      h(Wordmark, { size: 34 }),
      h(Badge, { tone: 'ok' }, 'Read-only studio'),
    ),
    h(
      'p',
      { style: { ...MUTED_TEXT, margin: '0 0 16px' } },
      'rhobuilder · design the unique part, reuse the primitive graph, prove and test, then plan a deploy.',
    ),
    h(Readiness, { readiness }),
    h(
      'div',
      { style: { display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(220px, 1fr))', gap: 12 } },
      h(NetworkCard, { name: 'devnet', profile: networks.devnet }),
      h(NetworkCard, { name: 'testnet', profile: networks.testnet }),
      h(NetworkCard, { name: 'mainnet', profile: networks.mainnet }),
    ),
    h(
      Card,
      {
        title: 'Primitive graph',
        subtitle: `${primitives.length} registered primitive(s), ${(registry.value?.edges ?? []).length} cross-app edge(s).`,
      },
      h(Domains, { primitives }),
      h(
        'p',
        { style: { ...MUTED_TEXT, marginTop: 10 } },
        'The registry is curated, not auto-discovered. A rho:id lookup proves resolution, not code identity.',
      ),
    ),
    h(Examples, { examples }),
    h(
      Card,
      { title: 'Truthful status' },
      h(Badge, { tone: 'ok' }, 'P5 devnet measured'),
      h('span', { style: { margin: '0 6px' } }),
      h(Badge, { tone: 'warn' }, 'P6 testnet code-complete, live gated'),
      h('span', { style: { margin: '0 6px' } }),
      h(Badge, { tone: 'neutral' }, 'generic scaffolds: model owed'),
      h(
        'p',
        { style: { ...MUTED_TEXT, margin: '8px 0 0' } },
        'The testnet baseline is blocked on r-cash component:account-factory being verified. Scaffold does not mean finished; proved-model is a shallow Lean model, never a Rholang refinement.',
      ),
    ),
    h(CliHandoff),
    h(
      'p',
      { style: { ...MUTED_TEXT, marginTop: 16 } },
      'Built on DeepSeek Harness (MIT). rhobuilder is AGPL-3.0-only. Signed operations are deliberately CLI-only.',
    ),
  )
}

function About() {
  return h(
    Card,
    { title: 'rhobuilder' },
    h('p', { style: { margin: 0 } }, 'A DSH app builder for rhovision dApps on rholo.'),
    h('p', { style: MUTED_TEXT }, 'Version 0.1.0 · UI read-only · signed actions CLI-only.'),
    h(
      'p',
      { style: MUTED_TEXT },
      'Docs: github.com/rhovision-org/rhobuilder · Originated by Patrick Mockridge.',
    ),
  )
}

window.__ModuleLoader__.load({
  id: 'rhobuilder-dsh-ui',
  factory(require) {
    const react = require('react')
    React = react && react.createElement ? react : react.default
    return {
      inject: ['slots'],
      apply(ctx) {
        const register = (slot, meta, component) =>
          ctx.slots.inject(slot, () => ctx.slots.register(meta, component))
        register('conversation.hero.frame', { name: 'conversation.hero.frame', id: 'rhobuilder-hero', order: -100 }, Hero)
        register(
          'conversation.hero.options',
          { name: 'conversation.hero.options', id: 'rhobuilder-starters', order: 100, label: 'Rhobuilder starters' },
          Starters,
        )
        register('sidebar.brand.mark', { name: 'sidebar.brand.mark' }, BrandMark)
        register('sidebar.brand.name', { name: 'sidebar.brand.name' }, () =>
          h('span', { style: { fontWeight: 700, color: INK } }, 'rhobuilder'),
        )
        register(
          'sidebar.panellist',
          { name: 'sidebar.panellist', id: 'rhobuilder', order: 10, label: 'Rhobuilder' },
          () =>
            h('img', {
              src: '/rhobuilder-ui/rhobuilder-mark.svg',
              alt: 'rhobuilder',
              width: 22,
              height: 22,
            }),
        )
        register('main', { name: 'main', key: 'rhobuilder' }, Dashboard)
        register(
          'settings.section',
          { name: 'settings.section', id: 'rhobuilder-about', order: 80, label: 'Rhobuilder' },
          About,
        )
      },
    }
  },
})

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
        h(Badge, { tone: 'neutral' }, 'Design save only'),
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
  const value = String(status ?? '')
  if (['ok', 'pass', 'verified', 'tested'].includes(value)) return 'ok'
  if (['fail', 'refused'].includes(value)) return 'fail'
  if (
    ['warn', 'setup', 'not_run', 'not-run', 'owed', 'proved-model', 'candidate'].includes(value) ||
    value.startsWith('not_run')
  ) {
    return 'warn'
  }
  return 'neutral'
}

const FIELD = {
  width: '100%',
  boxSizing: 'border-box',
  padding: '6px 8px',
  borderRadius: 6,
  border: `1px solid ${BORDER}`,
  background: 'var(--dsw-alias-bg-base, #ffffff)',
  color: INK,
  font: '13px/1.4 system-ui, sans-serif',
}

const LABEL = { display: 'block', margin: '9px 0 0', color: MUTED, fontSize: 12 }

function splitList(value) {
  return String(value ?? '')
    .split(/[\n,]/)
    .map((item) => item.trim())
    .filter(Boolean)
}

function buildDesignDraft(form = {}) {
  const name = String(form.name ?? '').trim()
  const slug =
    String(form.slug ?? '').trim() ||
    name
      .toLowerCase()
      .replace(/[^a-z0-9]+/g, '-')
      .replace(/^-+|-+$/g, '') ||
    'new-app'
  const reusePrimitives = splitList(form.reuse ?? 'rcash.account-factory')
  const customId = String(form.customId ?? 'unique').trim() || 'unique'
  const arms = splitList(form.arms ?? 'act')
  const actors = splitList(form.actors ?? 'user:act').map((line) => {
    const [id, capabilities] = line.split(':')
    return {
      id: String(id ?? '').trim() || 'user',
      capabilities: splitList(capabilities ?? 'act'),
    }
  })
  const resources = splitList(form.resources ?? `${customId}-state:${customId}State`).map((line) => {
    const [id, stateCell] = line.split(':')
    return {
      id: String(id ?? '').trim() || `${customId}-state`,
      owner: 'app',
      stateCell: String(stateCell ?? '').trim() || `${customId}State`,
      lifecycle: ['open', 'closed'],
    }
  })
  const components = reusePrimitives.map((primitive, index) => ({
    id: String(primitive).replace(/[^a-z0-9]+/gi, '-').replace(/^-+|-+$/g, '') || `reuse-${index + 1}`,
    tag: 'reuse',
    primitive,
    adapter: `adapter-${index + 1}.rho`,
  }))
  const leanStatus = String(form.leanStatus ?? 'owed').trim() || 'owed'
  const custom = {
    id: customId,
    tag: 'new',
    kind: 'custom',
    whyUnique:
      String(form.customWhy ?? '').trim() ||
      'Describe the coordination this app owns instead of rebuilding a commodity primitive.',
    noRegisteredAlternative: [reusePrimitives[0] ?? 'rcash.account-factory'],
    arms,
    invariants: splitList(form.invariants ?? ''),
  }
  if (leanStatus === 'proved-model') {
    custom.leanStatus = 'proved-model'
    custom.lean = String(form.lean ?? 'model').trim() || 'model'
  } else {
    custom.leanStatus = 'owed'
  }
  components.push(custom)
  return {
    schema: 'rhobuilder.design/v1',
    app: {
      name,
      slug,
      summary: String(form.summary ?? '').trim(),
      networks: ['devnet', 'testnet'],
    },
    uniqueValue: String(form.uniqueValue ?? '').trim(),
    actors,
    resources,
    components,
    graph: { vertices: [], edges: [] },
    conservation: splitList(form.conservation ?? '').map((line) => {
      const [path, assertion] = line.split('|')
      return { path: String(path ?? '').trim(), assertion: String(assertion ?? '').trim() }
    }),
    concurrency: splitList(form.concurrency ?? '').map((line) => {
      const [risk, mechanism] = line.split('|')
      return { risk: String(risk ?? '').trim(), mechanism: String(mechanism ?? '').trim() }
    }),
    remedies: [],
    acceptance: [
      {
        id: 'a1',
        given: String(form.given ?? 'a user has access').trim(),
        when: String(form.when ?? 'they invoke the unique arm').trim(),
        then: String(form.then ?? 'the app state changes exactly once').trim(),
      },
    ],
  }
}

function designReportView(value) {
  const report = value?.report ?? value ?? {}
  if (report?.kind === 'error') {
    return {
      ok: false,
      verdict: 'error',
      error: String(report?.error?.hint ?? report?.error?.message ?? 'the design could not be checked'),
      checks: [],
      conflicts: [],
      owed: [],
      missing: [],
    }
  }
  const validation = Array.isArray(report?.validation?.checks) ? report.validation.checks : []
  const reuse = Array.isArray(report?.reuse?.checks) ? report.reuse.checks : []
  return {
    ok: report?.ok === true,
    verdict: report?.verdict ?? (report?.ok ? 'pass' : 'refuse'),
    error: null,
    checks: [...validation, ...reuse].filter((check) => check?.status !== 'ok'),
    conflicts: Array.isArray(report?.reuse?.conflicts) ? report.reuse.conflicts : [],
    owed: Array.isArray(report?.owed) ? report.owed : [],
    missing: Array.isArray(report?.reuse?.missing) ? report.reuse.missing : [],
  }
}

function DesignComposer() {
  const initial = {
    name: 'My dApp',
    slug: 'my-dapp',
    summary: 'What this app does in one sentence.',
    uniqueValue: 'The unique coordination this app owns; commodity behavior is reused.',
    reuse: 'rcash.account-factory',
    customId: 'unique',
    customWhy: 'No registered primitive performs this specific coordination.',
    arms: 'act',
    invariants: '',
    leanStatus: 'owed',
    actors: 'user:act',
    resources: 'unique-state:uniqueState',
    conservation: '',
    concurrency: '',
    given: 'a user has access',
    when: 'they invoke the unique arm',
    then: 'the app state changes exactly once',
    project: '',
  }
  const [form, setForm] = React.useState(initial)
  const [report, setReport] = React.useState(null)
  const [save, setSave] = React.useState({ state: 'idle' })
  const update = (key, value) => {
    setForm({ ...form, [key]: value })
    setReport(null)
    setSave({ state: 'idle' })
  }
  const field = (key) => ({
    value: form[key] ?? '',
    onChange: (event) => update(key, event.target.value),
  })
  const draft = buildDesignDraft(form)
  const validate = () => {
    setReport(null)
    setSave({ state: 'idle' })
    fetch('/rhobuilder/api/design-validate', {
      method: 'POST',
      credentials: 'same-origin',
      headers: { 'content-type': 'application/json' },
      body: JSON.stringify(draft),
    })
      .then((response) => response.json().then((value) => ({ response, value })))
      .then(({ response, value }) => {
        if (!response.ok) throw new Error(value?.error ?? `HTTP ${response.status}`)
        setReport(value)
      })
      .catch((error) => setSave({ state: 'error', message: String(error?.message ?? error) }))
  }
  const saveDesign = (overwrite = false) => {
    if (!report?.ok) return
    setSave({ state: 'saving' })
    fetch('/rhobuilder/api/design-save', {
      method: 'POST',
      credentials: 'same-origin',
      headers: { 'content-type': 'application/json' },
      body: JSON.stringify({
        design: draft,
        project: String(form.project ?? '').trim() || undefined,
        overwrite,
      }),
    })
      .then((response) => response.json().then((value) => ({ response, value })))
      .then(({ response, value }) => {
        if (response.status === 409) {
          setSave({ state: 'exists', path: value?.path ?? '' })
          return
        }
        if (!response.ok || !value?.ok) {
          throw new Error(value?.error ?? value?.report?.error ?? `HTTP ${response.status}`)
        }
        setSave({ state: 'saved', path: value.path })
      })
      .catch((error) => setSave({ state: 'error', message: String(error?.message ?? error) }))
  }
  const view = report ? designReportView(report) : null
  return h(
    Card,
    {
      title: 'Design composer',
      subtitle: 'Draft the unique part, validate it against the reuse gate, and save design.json for continuity.',
    },
    h('label', { style: LABEL }, 'App name', h('input', { style: FIELD, ...field('name') })),
    h('label', { style: LABEL }, 'Slug', h('input', { style: FIELD, ...field('slug') })),
    h('label', { style: LABEL }, 'Summary', h('input', { style: FIELD, ...field('summary') })),
    h('label', { style: LABEL }, 'Unique value', h('textarea', { style: FIELD, rows: 2, ...field('uniqueValue') })),
    h(
      'label',
      { style: LABEL },
      'Reuse primitives (comma separated)',
      h('input', { style: FIELD, ...field('reuse') }),
    ),
    h(
      'div',
      { style: { display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(190px, 1fr))', gap: 10 } },
      h('label', { style: LABEL }, 'Unique component id', h('input', { style: FIELD, ...field('customId') })),
      h('label', { style: LABEL }, 'Arms', h('input', { style: FIELD, ...field('arms') })),
      h(
        'label',
        { style: LABEL },
        'Lean status',
        h(
          'select',
          { style: FIELD, ...field('leanStatus') },
          h('option', { value: 'owed' }, 'owed'),
          h('option', { value: 'proved-model' }, 'proved-model'),
        ),
      ),
    ),
    h(
      'label',
      { style: LABEL },
      'Why the unique component exists',
      h('textarea', { style: FIELD, rows: 2, ...field('customWhy') }),
    ),
    h('label', { style: LABEL }, 'Actors (id:capability, …)', h('input', { style: FIELD, ...field('actors') })),
    h(
      'label',
      { style: LABEL },
      'Acceptance: given / when / then',
      h('input', { style: FIELD, ...field('given') }),
      h('input', { style: { ...FIELD, marginTop: 6 }, ...field('when') }),
      h('input', { style: { ...FIELD, marginTop: 6 }, ...field('then') }),
    ),
    h(
      'label',
      { style: LABEL },
      'Project folder (optional; blank saves to designs/<slug>.json)',
      h('input', { style: FIELD, ...field('project') }),
    ),
    h(
      'div',
      { style: { display: 'flex', gap: 7, flexWrap: 'wrap' } },
      h('button', { type: 'button', style: BUTTON, onClick: validate }, 'Validate draft'),
      h(
        'button',
        {
          type: 'button',
          style: { ...BUTTON, opacity: report?.ok ? 1 : 0.5 },
          disabled: !report?.ok,
          onClick: () => saveDesign(false),
        },
        'Save design.json',
      ),
      h(
        'button',
        { type: 'button', style: BUTTON, onClick: () => copyText(JSON.stringify(draft, null, 2)) },
        'Copy design JSON',
      ),
      h(
        'button',
        {
          type: 'button',
          style: BUTTON,
          onClick: () =>
            copyText(
              `Review the design at ${save.path || 'designs/<slug>.json'} with rhobuilder-dapp-design and rhobuilder-primitive-reuse. Report reuse/extend/new and owed arms before proposing a scaffold.`,
            ),
        },
        'Copy review prompt',
      ),
    ),
    view &&
      h(
        'div',
        { style: { marginTop: 10 } },
        h(Badge, { tone: view.ok ? 'ok' : 'fail' }, view.ok ? 'Design valid' : `Design ${view.verdict}`),
        view.error && h('p', { style: { color: '#b42318' } }, view.error),
        view.conflicts.map((conflict) =>
          h(
            'p',
            { key: conflict.component, style: { ...MUTED_TEXT, margin: '5px 0 0' } },
            `Conflict: ${conflict.component} duplicates ${conflict.primitive}. ${conflict.detail}`,
          ),
        ),
        view.missing.length > 0 &&
          h('p', { style: MUTED_TEXT }, `Unregistered alternatives: ${view.missing.join(', ')}`),
        view.owed.length > 0 &&
          h('p', { style: MUTED_TEXT }, `Owed Lean obligations: ${view.owed.join(', ')}`),
        view.checks.slice(0, 12).map((check) =>
          h(
            'p',
            { key: `${check.id}-${check.status}`, style: { ...MUTED_TEXT, margin: '4px 0 0' } },
            `${check.status}: ${check.id} — ${check.detail}`,
          ),
        ),
      ),
    save.state === 'saving' && h('p', { style: MUTED_TEXT }, 'Saving…'),
    save.state === 'saved' &&
      h(
        'p',
        { style: { ...MUTED_TEXT, marginTop: 8 } },
        `Saved to ${save.path}. `,
        h('button', { type: 'button', style: BUTTON, onClick: () => copyText(save.path) }, 'Copy path'),
      ),
    save.state === 'exists' &&
      h(
        'p',
        { style: { color: '#b45309', marginTop: 8 } },
        `${save.path} already exists. `,
        h(
          'button',
          { type: 'button', style: BUTTON, onClick: () => saveDesign(true) },
          'Overwrite design.json',
        ),
      ),
    save.state === 'error' && h('p', { style: { color: '#b42318' } }, `Design action failed: ${save.message}`),
  )
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

function ProjectStatus() {
  const status = useApi('/rhobuilder/api/project-status', { projects: [] })
  const rows = Array.isArray(status.value?.projects) ? status.value.projects : []
  return h(
    Card,
    {
      title: 'Project status',
      subtitle:
        'Design, scaffold, tests, model and deployment are separate claims. Not run is not a pass.',
    },
    status.error && h('p', { style: { color: '#b42318' } }, status.error),
    rows.length === 0 &&
      h('p', { style: MUTED_TEXT }, 'No fixture or saved workspace designs are available.'),
    h(
      'div',
      { style: { display: 'grid', gap: 10, marginTop: 8 } },
      rows.map((row) =>
        h(
          'div',
          { key: `${row.source}-${row.slug}`, style: { borderTop: `1px solid ${BORDER}`, paddingTop: 9 } },
          h('strong', null, row.name ?? row.slug),
          h(
            'span',
            { style: { ...MUTED_TEXT, marginLeft: 8 } },
            `${row.source === 'workspace' ? 'saved draft' : 'fixture'} · ${row.file ?? ''}`,
          ),
          h(
            'div',
            { style: { display: 'flex', gap: 6, flexWrap: 'wrap', marginTop: 5 } },
            (row.badges ?? []).map((badge) =>
              h(
                Badge,
                { key: `${badge.label}-${badge.state}`, tone: statusTone(badge.state) },
                badge.label,
              ),
            ),
          ),
          h(
            'p',
            { style: { ...MUTED_TEXT, margin: '5px 0 0' } },
            (row.badges ?? []).map((badge) => `${badge.label}: ${badge.detail}`).join(' · '),
          ),
        ),
      ),
    ),
  )
}

function evidenceView(item) {
  const value = item?.value ?? {}
  const apps = []
  if (value.baseline && typeof value.baseline === 'object') {
    for (const [name, app] of Object.entries(value.baseline)) {
      apps.push({
        name,
        commit: String(app?.commit ?? '').slice(0, 12),
        root: app?.root ?? '',
        status: app?.status ?? 'not_run',
        steps: app?.steps ?? 0,
      })
    }
  }
  if (value.app && typeof value.app === 'object') {
    apps.push({
      name: value.app.slug ?? value.app.name ?? 'app',
      commit: String(value.app.commit ?? 'local').slice(0, 12),
      root: value.app.root ?? '',
      status: value.app.status ?? 'not_run',
      steps: value.app.steps ?? 0,
    })
  }
  const suites = Array.isArray(value.suites?.results)
    ? value.suites.results.map((suite) => ({
        name: suite?.suite ?? '?',
        status: suite?.status ?? 'not_run',
        detail: suite?.detail ?? '',
      }))
    : []
  const checks =
    value.checks && typeof value.checks === 'object'
      ? Object.entries(value.checks).map(([name, status]) => ({ name, status: String(status) }))
      : []
  const notes = Array.isArray(value.notes) ? value.notes : value.note ? [value.note] : []
  return {
    file: item?.file ?? 'evidence',
    recordedAt: value.recordedAt ?? '',
    network: value.network ?? value.networkId ?? '',
    apps,
    suites,
    checks,
    notes,
  }
}

function Evidence() {
  const evidence = useApi('/rhobuilder/api/evidence', { evidence: [] })
  const rows = Array.isArray(evidence.value?.evidence) ? evidence.value.evidence : []
  return h(
    Card,
    {
      title: 'Measured evidence',
      subtitle: 'What actually ran, with machine states translated. Evidence is not authority.',
    },
    evidence.error && h('p', { style: { color: '#b42318' } }, evidence.error),
    rows.length === 0 && h('p', { style: MUTED_TEXT }, 'No evidence documents are installed.'),
    h(
      'div',
      { style: { display: 'grid', gap: 14, marginTop: 8 } },
      rows.map((item) => {
        const view = evidenceView(item)
        return h(
          'div',
          { key: view.file, style: { borderTop: `1px solid ${BORDER}`, paddingTop: 9 } },
          h('strong', null, view.file),
          h(
            'p',
            { style: { ...MUTED_TEXT, margin: '3px 0 6px' } },
            `${view.network || 'unknown network'} · recorded ${view.recordedAt || 'unknown'}`,
          ),
          h(
            'div',
            { style: { display: 'grid', gap: 5 } },
            view.apps.map((app) =>
              h(
                'div',
                { key: app.name, style: { display: 'flex', gap: 7, flexWrap: 'wrap', alignItems: 'center' } },
                h('strong', null, app.name),
                h(Badge, { tone: statusTone(app.status) }, app.status),
                h('span', { style: MUTED_TEXT }, `${app.steps} step(s)`),
                app.commit && h('span', { style: MUTED_TEXT }, `commit ${app.commit}`),
                app.root &&
                  h(
                    'button',
                    { type: 'button', style: BUTTON, onClick: () => copyText(app.root) },
                    `Copy ${app.root.slice(0, 18)}…`,
                  ),
              ),
            ),
          ),
          view.suites.length > 0 &&
            h(
              'p',
              { style: { ...MUTED_TEXT, margin: '7px 0 0' } },
              `Suites: ${view.suites
                .map((suite) => `${suite.name} ${suite.status}`)
                .join(' · ')}`,
            ),
          view.checks.length > 0 &&
            h(
              'p',
              { style: { ...MUTED_TEXT, margin: '7px 0 0' } },
              `Checks: ${view.checks.map((check) => `${check.name} ${check.status}`).join(' · ')}`,
            ),
          view.notes.map((note, index) =>
            h('p', { key: index, style: { ...MUTED_TEXT, margin: '5px 0 0' } }, note),
          ),
          h(
            'details',
            { style: { marginTop: 8 } },
            h('summary', { style: MUTED_TEXT }, 'Raw JSON'),
            h(
              'pre',
              { style: CODE },
              JSON.stringify(item.value, null, 2).slice(0, 100_000),
            ),
          ),
        )
      }),
    ),
    h(
      'p',
      { style: { ...MUTED_TEXT, marginTop: 10 } },
      'Evidence is a record, not authority. The live check is baseline verify --deep, which re-resolves every root.',
    ),
  )
}

function Preflight() {
  const preflight = useApi('/rhobuilder/api/preflight?network=testnet', { blockers: [] })
  const blockers = Array.isArray(preflight.value?.blockers) ? preflight.value.blockers : []
  return h(
    Card,
    {
      title: 'Testnet campaign',
      subtitle: 'Read-only preflight. Live execution needs a separate explicit approval.',
    },
    preflight.error && h('p', { style: { color: '#b42318' } }, preflight.error),
    h(
      'div',
      { style: { display: 'flex', gap: 6, flexWrap: 'wrap', marginTop: 8 } },
      h(Badge, { tone: blockers.length ? 'warn' : 'ok' }, `${blockers.length} blocker(s)`),
      h(Badge, { tone: 'neutral' }, 'Live: not run'),
    ),
    blockers.length === 0 &&
      h('p', { style: MUTED_TEXT }, 'No blockers are recorded; live execution still needs approval.'),
    h(
      'div',
      { style: { display: 'grid', gap: 7, marginTop: 8 } },
      blockers.map((blocker) =>
        h(
          'div',
          { key: `${blocker.kind}-${blocker.detail}`, style: { borderTop: `1px solid ${BORDER}`, paddingTop: 7 } },
          h('strong', null, blocker.kind ?? 'blocker'),
          h('p', { style: { ...MUTED_TEXT, margin: '2px 0 0' } }, blocker.detail ?? ''),
          blocker.hint && h('p', { style: { ...MUTED_TEXT, margin: '2px 0 0' } }, blocker.hint),
        ),
      ),
    ),
    h(
      'p',
      { style: { ...MUTED_TEXT, marginTop: 9 } },
      'Approval checkpoints: campaign approval, pin resolution, faucet spend approval, --allow-live with the identity/plan --confirm token, and manual partial-root reconciliation. This UI never executes any of them.',
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
    h('p', { style: { margin: '8px 0 0' } }, 'Start the existing local devnet and check it:'),
    h(
      'pre',
      { style: CODE },
      'python3 tools/devnet.py status --json\n' +
        'python3 tools/devnet.py start --json\n' +
        'python3 tools/devnet.py onboard --json   # dry run; --execute is the approved step',
    ),
    h('p', { style: { margin: '10px 0 0' } }, 'Create and fund a devnet ops key:'),
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
      h(Badge, { tone: 'ok' }, 'Design save only · no signing'),
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
    h(DesignComposer),
    h(Examples, { examples }),
    h(ProjectStatus),
    h(Evidence),
    h(Preflight),
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
      internals: { buildDesignDraft, designReportView, evidenceView },
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

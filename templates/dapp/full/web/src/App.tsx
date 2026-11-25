const uniqueValue = '__unique_value__'

export default function App() {
  return (
    <main style={{ fontFamily: 'system-ui, sans-serif', margin: '3rem auto', maxWidth: 720 }}>
      <h1>__app_name__</h1>
      <p>__summary__</p>
      <h2>Unique value</h2>
      <p>{uniqueValue}</p>
      <p>
        Status: <strong>Scaffold</strong>. The chain client and booking flow are not wired
        yet; the Rholang cells, harness and Lean obligations are scaffolds.
      </p>
    </main>
  )
}

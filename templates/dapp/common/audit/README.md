# Audit registers

`registers/arms.yaml` and `registers/findings.yaml` are generated from the design at
scaffold time. Add a finding only with a reproduction and a regression test; do not mark a
finding fixed until both are in the tree.

`status` values follow the org vocabulary: `scaffold`, `implemented`, `tested`,
`proved-model`, `proved-conformance`, `modeled`, `assumption`, `owed`, `open`.

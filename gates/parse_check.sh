#!/usr/bin/env bash
# Parse Rholang files with the target node's own parser.
#
#   RHOLO_DIR=/path/to/rholo-fixed ./tools/parse_check.sh contracts/src/*.rho
#
# Without RHOLO_DIR this exits 2 (not_run); it never reports a parse pass it did not run.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
RHOLO_DIR="${RHOLO_DIR:-}"
if [ -z "$RHOLO_DIR" ]; then
  echo "parse-check: not_run (set RHOLO_DIR to the rholo checkout)" >&2
  exit 2
fi
if [ ! -d "$RHOLO_DIR/rholang" ]; then
  echo "parse-check: not_run ($RHOLO_DIR/rholang does not exist)" >&2
  exit 2
fi

CRATE="$ROOT/.devnet/parse-check"
mkdir -p "$CRATE/src"

ARGS=()
for arg in "$@"; do
  ARGS+=("$(cd "$(dirname "$arg")" && pwd)/$(basename "$arg")")
done

cat > "$CRATE/Cargo.toml" <<EOF
[package]
name = "app-parse"
version = "0.1.0"
edition = "2021"

[dependencies]
rchain-rholang = { path = "$(cd "$RHOLO_DIR" && pwd)/rholang" }
EOF

cat > "$CRATE/src/main.rs" <<'EOF'
use std::{env, fs};
fn main() {
    let mut bad = 0;
    for path in env::args().skip(1) {
        let text = match fs::read_to_string(&path) {
            Ok(text) => text,
            Err(error) => {
                bad += 1;
                println!("FAIL {path}: {error}");
                continue;
            }
        };
        match rchain_rholang::parser::parse(&text) {
            Ok(_) => println!("OK   {path}"),
            Err(error) => {
                bad += 1;
                println!("FAIL {path}: {error:?}");
            }
        }
    }
    std::process::exit(if bad == 0 { 0 } else { 1 });
}
EOF

cd "$CRATE"
RUSTC_BOOTSTRAP=1 CC="${CC:-gcc}" cargo build --release --quiet
exec "$CRATE/target/release/app-parse" "${ARGS[@]}"

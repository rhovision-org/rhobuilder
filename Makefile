PY ?= python3
CARGO ?= cargo
NODE ?= node
MDBOOK ?= mdbook

.PHONY: help check rust-check fmt clippy test plugin-test launcher-test \
	docs docs-check dsh-check registry-check scaffold-e2e formal-check clean

help:
	@echo "rhobuilder targets:"
	@echo "  make check          fmt + clippy + rust tests + plugin tests + launcher tests + docs"
	@echo "  make rust-check     cargo fmt --check, clippy -D warnings, cargo test"
	@echo "  make plugin-test    node --test agents/deepseek/plugin/test"
	@echo "  make launcher-test  python unittest for tools/"
	@echo "  make docs-check     mdBook build plus SUMMARY/link check"
	@echo "  make dsh-check      read-only DeepSeek Harness preflight"
	@echo "  make chain-probe    read-only testnet identity probe"
	@echo "  make registry-check validate the committed registry (drift is P1-next)"
	@echo "  make scaffold-e2e   P2: generate and verify a fixture dApp"
	@echo "  make formal-check   P4: Lean RhoApp and per-dApp obligations"

check: rust-check plugin-test launcher-test registry-check docs-check

rust-check: fmt clippy test

fmt:
	$(CARGO) fmt --all -- --check

clippy:
	$(CARGO) clippy --workspace --all-targets -- -D warnings

test:
	$(CARGO) test --workspace

plugin-test:
	$(NODE) --test agents/deepseek/plugin/test/*.test.mjs

launcher-test:
	$(PY) -m unittest discover -s tools/tests -v

docs:
	$(MDBOOK) build

docs-check: docs
	$(PY) tools/check_docs.py

dsh-check:
	$(PY) tools/launch_dsh.py --check

chain-probe:
	./target/debug/rhobuilder --json chain probe --network testnet

registry-check:
	$(PY) tools/check_registry.py

scaffold-e2e:
	@echo "scaffold-e2e: not implemented until P2" >&2
	@exit 2

formal-check:
	@echo "formal-check: not implemented until P4" >&2
	@exit 2

clean:
	$(CARGO) clean
	rm -rf docs/book .dsh .dsh-home

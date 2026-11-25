PY ?= python3
CARGO ?= cargo
NODE ?= node
MDBOOK ?= mdbook
SOURCE_ROOT ?= $(HOME)
DESIGN ?= fixtures/designs/booking.json

.PHONY: help check rust-check fmt clippy test plugin-test ui-test launcher-test \
	docs docs-check dsh-check registry-check registry-drift vendor-install-check gates-check \
	package-check chain-probe design-check design-refuse scaffold-e2e mcp-test baseline-plan \
	baseline-preflight devnet-status formal-check clean

help:
	@echo "rhobuilder targets:"
	@echo "  make check          fmt + clippy + rust tests + plugin tests + launcher tests + docs"
	@echo "  make rust-check     cargo fmt --check, clippy -D warnings, cargo test"
	@echo "  make plugin-test    node --test agents/deepseek/plugin/test"
	@echo "  make ui-test        check and test the rhobuilder DSH UI package"
	@echo "  make launcher-test  python unittest for tools/"
	@echo "  make docs-check     mdBook build plus docs/README link/status check"
	@echo "  make dsh-check      read-only DeepSeek Harness preflight"
	@echo "  make chain-probe    read-only testnet identity probe"
	@echo "  make registry-check validate the committed registry"
	@echo "  make registry-drift pinned-commit drift against $(SOURCE_ROOT)"
	@echo "  make vendor-install-check verify pinned vendored terms and hashes"
	@echo "  make gates-check    verify canonical gates and their vendored copies"
	@echo "  make package-check  verify the complete release package layout"
	@echo "  make design-check   validate $(DESIGN) and run the reuse gate"
	@echo "  make design-refuse  assert the duplicate-escrow fixture is refused"
	@echo "  make scaffold-e2e   generate the booking fixture and run its gates"
	@echo "  make baseline-plan  read-only testnet baseline plan"
	@echo "  make baseline-preflight read-only pins/roots/funding blockers"
	@echo "  make devnet-status  read-only local devnet container/node status"
	@echo "  make formal-check   P4: Lean RhoApp and per-dApp obligations"

check: rust-check plugin-test ui-test mcp-test launcher-test registry-check vendor-install-check gates-check package-check design-check docs-check

rust-check: fmt clippy test

fmt:
	$(CARGO) fmt --all -- --check

clippy:
	$(CARGO) clippy --workspace --all-targets -- -D warnings

test:
	$(CARGO) test --workspace

plugin-test:
	$(NODE) --test agents/deepseek/plugin/test/*.test.mjs

ui-test:
	$(NODE) --check agents/deepseek/ui/index.js
	$(NODE) --check agents/deepseek/ui/client.js
	$(NODE) --test agents/deepseek/ui/test/*.test.mjs

mcp-test:
	$(NODE) --test mcp/test/*.test.mjs

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

baseline-plan:
	./target/debug/rhobuilder --json baseline plan --network testnet

baseline-preflight:
	./target/debug/rhobuilder --json baseline preflight --network testnet

devnet-status:
	$(PY) tools/devnet.py status --json

design-check:
	$(CARGO) run -q -p rhobuilder-cli -- --json design check --path $(DESIGN)

design-refuse:
	@cargo run -q -p rhobuilder-cli -- design check \
		--path fixtures/designs/duplicate-escrow.json >/dev/null 2>&1 \
		&& { echo "expected refusal"; exit 1; } || echo "refused as expected"

registry-check:
	$(PY) tools/check_registry.py

registry-drift:
	$(CARGO) run -q -p rhobuilder-cli -- --json registry drift \
		--source-root $(SOURCE_ROOT) --strict

vendor-install-check:
	$(PY) tools/vendor_install.py --check

gates-check:
	$(PY) tools/gate_sync.py --check

package-check:
	$(PY) tools/package_release.py --check

scaffold-e2e:
	rm -rf /tmp/rhobuilder-e2e /tmp/rhobuilder-e2e-refused
	$(CARGO) run -q -p rhobuilder-cli -- new /tmp/rhobuilder-e2e \
		--design fixtures/designs/booking.json --template headless --apply --force
	$(MAKE) -C /tmp/rhobuilder-e2e check
	cd /tmp/rhobuilder-e2e && $(CARGO) check --workspace
	$(CARGO) run -q -p rhobuilder-cli -- --json formal check --project /tmp/rhobuilder-e2e
	@if $(CARGO) run -q -p rhobuilder-cli -- new /tmp/rhobuilder-e2e-refused \
		--design fixtures/designs/duplicate-escrow.json --template headless \
		--plan --force >/dev/null 2>&1; then \
		echo "scaffold-e2e: expected the duplicate design to be refused" >&2; exit 1; \
	else \
		echo "scaffold-e2e: duplicate design refused as expected"; \
	fi

formal-check:
	@if command -v lake >/dev/null 2>&1; then \
		cd formal/lean/RhoApp && lake build; \
	else \
		echo "formal-check: not_run (lake is not installed)" >&2; exit 2; \
	fi

clean:
	$(CARGO) clean
	rm -rf docs/book .dsh .dsh-home

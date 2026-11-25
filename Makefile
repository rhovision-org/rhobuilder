PY ?= python3
CARGO ?= cargo
NODE ?= node
MDBOOK ?= mdbook
SOURCE_ROOT ?= /home/patrick
DESIGN ?= fixtures/designs/booking.json

.PHONY: help check rust-check fmt clippy test plugin-test launcher-test \
	docs docs-check dsh-check registry-check registry-drift chain-probe \
	design-check design-refuse scaffold-e2e mcp-test formal-check clean

help:
	@echo "rhobuilder targets:"
	@echo "  make check          fmt + clippy + rust tests + plugin tests + launcher tests + docs"
	@echo "  make rust-check     cargo fmt --check, clippy -D warnings, cargo test"
	@echo "  make plugin-test    node --test agents/deepseek/plugin/test"
	@echo "  make launcher-test  python unittest for tools/"
	@echo "  make docs-check     mdBook build plus SUMMARY/link check"
	@echo "  make dsh-check      read-only DeepSeek Harness preflight"
	@echo "  make chain-probe    read-only testnet identity probe"
	@echo "  make registry-check validate the committed registry"
	@echo "  make registry-drift pinned-commit drift against $(SOURCE_ROOT)"
	@echo "  make design-check   validate $(DESIGN) and run the reuse gate"
	@echo "  make design-refuse  assert the duplicate-escrow fixture is refused"
	@echo "  make scaffold-e2e   generate the booking fixture and run its gates"
	@echo "  make formal-check   P4: Lean RhoApp and per-dApp obligations"

check: rust-check plugin-test mcp-test launcher-test registry-check design-check docs-check

rust-check: fmt clippy test

fmt:
	$(CARGO) fmt --all -- --check

clippy:
	$(CARGO) clippy --workspace --all-targets -- -D warnings

test:
	$(CARGO) test --workspace

plugin-test:
	$(NODE) --test agents/deepseek/plugin/test/*.test.mjs

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

PY ?= python3
CARGO ?= cargo
MDBOOK ?= mdbook
RHOLO_DIR ?=
RHOBUILDER_BIN ?= rhobuilder
RHOBUILDER_NETWORK ?= devnet
RHOBUILDER_REGISTRY ?=
KEYS ?= .ops/$(RHOBUILDER_NETWORK)/keys.json

.PHONY: help check spec-check gates-check docs docs-check parse-check formal-check unit-all \
	client-check devnet-up keys install ci clean

help:
	@echo "check        spec structure, term budgets, JSON and docs"
	@echo "parse-check  node parser; needs RHOLO_DIR (exit 2 otherwise)"
	@echo "formal-check Lean model; needs lake (exit 2 otherwise)"
	@echo "unit-all     behavioural suites; needs RHOBUILDER_DEVNET=1 (exit 2 otherwise)"
	@echo "client-check web typecheck/test/build when web/ exists"

check: gates-check spec-check docs-check

gates-check:
	$(PY) tools/check_gates.py

spec-check:
	$(PY) tools/spec_check.py

docs:
	$(MDBOOK) build

docs-check: docs
	$(PY) tools/spec_check.py --docs

parse-check:
	bash tools/parse_check.sh contracts/src/*.rho

formal-check:
	@if command -v lake >/dev/null 2>&1; then \
		cd spec/lean && lake build; \
	else \
		echo "formal-check: not_run (lake is not installed)" >&2; exit 2; \
	fi
	$(PY) tools/check_vendored.py
	$(PY) tools/check_obligations.py

unit-all:
	$(PY) harness/run_suites.py

devnet-up:
	@if [ -n "$(RHOBUILDER_DEVNET_CMD)" ]; then \
		$(RHOBUILDER_DEVNET_CMD); \
	else \
		echo "devnet-up: set RHOBUILDER_DEVNET_CMD or start a devnet at $$RHOBUILDER_NODE"; \
	fi

keys:
	$(RHOBUILDER_BIN) --json $(if $(RHOBUILDER_REGISTRY),--registry $(RHOBUILDER_REGISTRY)) \
		baseline keygen --network $(RHOBUILDER_NETWORK) --out $(KEYS)

install:
	RHOBUILDER_DEVNET=1 RHOBUILDER_NETWORK=$(RHOBUILDER_NETWORK) \
	RHOBUILDER_BIN=$(RHOBUILDER_BIN) RHOBUILDER_KEYS=$(KEYS) \
	RHOBUILDER_REGISTRY=$(RHOBUILDER_REGISTRY) \
	$(PY) harness/install.py

ci: check unit-all

client-check:
	@if [ -d web ]; then \
		npm --prefix web ci && npm --prefix web test -- --run && npm --prefix web run build; \
	else \
		echo "client-check: headless template (no web/)"; \
	fi

clean:
	$(CARGO) clean
	rm -rf docs/book .devnet

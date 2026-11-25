#!/usr/bin/env python3
"""Vendor the pinned sibling deploy terms without touching sibling worktrees.

Each plan names a sibling repo, a commit and a set of built terms. The tool reads the
terms with ``git show <commit>:<path>`` only, records their SHA-256, and writes them under
``registry/install/<app>/<commit>/``. ``--check`` re-reads the committed copy and, when the
sibling checkout is available, re-checks the pinned blob.

The terms are AGPL-3.0-only and keep their placeholders. Rendering happens at install time
in ``rhobuilder-chain``; no key, root URI or admin key is baked into this tree.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import re
import subprocess
import sys
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parent.parent
SIBLING_ROOT = Path(os.environ.get("RHOBUILDER_SIBLING_ROOT", str(Path.home())))
INSTALL_ROOT = ROOT / "registry" / "install"
SCHEMA = "rhobuilder.install-plan/v1"
ORIGIN_SCHEMA = "rhobuilder.install-origin/v1"

# Root keys were extracted from the pinned built terms' own ``Root!("register", ...)``
# calls. Keep this list explicit: a missing key is a real install failure.
PLANS: dict[str, dict[str, Any]] = {
    "r-cash": {
        "dir": "Rholphin",
        "repo": "https://github.com/rhovision-org/r-cash",
        "commit": "49b441dea4f5561992387ed670ab263fcd5dd164",
        "sourceRoot": "contracts/build/deploy/v3",
        "rootKey": "rcash:root",
        "phlo": {"limit": 2_000_000, "price": 1},
        "dependencies": [],
        "steps": [
            {"name": "00_root", "term": "00_root.rho", "root": True},
            {"name": "03_admin_guard", "term": "03_admin_guard.rho"},
            {"name": "01_kernel", "term": "01_kernel.rho"},
            {"name": "02_bounds", "term": "02_bounds.rho"},
            {"name": "04_governor", "term": "04_governor.rho"},
            {"name": "05_amm_math", "term": "05_amm_math.rho"},
            {"name": "06_collateral_math", "term": "06_collateral_math.rho"},
            {"name": "07_oracle_math", "term": "07_oracle_math.rho"},
            {"name": "08_claim_code", "term": "08_claim_code.rho"},
            {"name": "12_oracle_registry", "term": "12_oracle_registry.rho"},
            {"name": "13_oracle_hub", "term": "13_oracle_hub.rho"},
            {"name": "32_currency_registry", "term": "32_currency_registry.rho"},
            {
                "name": "14_asset_resolver",
                "term": "14_asset_resolver.rho",
                "captures": [
                    {"placeholder": "__ASSET_RESOLVER_URI__", "from": "uris", "index": 0}
                ],
            },
            {"name": "40_transfer", "term": "40_transfer.rho"},
            {"name": "09_identity", "term": "09_identity.rho"},
        ],
        "requiredKeys": [
            "rcash:root",
            "component:admin-guard",
            "component:clock",
            "component:verifier",
            "component:codec",
            "component:bounds",
            "component:governor",
            "component:amm-math",
            "component:collateral-math",
            "component:oracle-math",
            "component:claim-code",
            "component:oracle-registry",
            "component:oracle-hub",
            "component:stablecoin-directory",
            "component:currency-registry",
            "component:asset-resolver",
            "component:transfer-book",
            "component:account-factory",
        ],
    },
    "r-gen": {
        "dir": "RGen",
        "repo": "https://github.com/rhovision-org/r-gen",
        "commit": "bcc840c25e334048bef390823a21aaf7e209d15b",
        "sourceRoot": "contracts/build/deploy",
        "rootKey": "component:root",
        "resolveArity": 2,
        "phlo": {"limit": 8_000_000, "price": 1},
        "dependencies": ["r-cash"],
        "steps": [
            {"name": "00_root", "term": "00_root.rho", "root": True},
            {
                "name": "10_identity",
                "term": "10_identity.rho",
                "captures": [
                    {"placeholder": "__IDENTITY_DIRECTORY_URI__", "from": "map", "key": "identity-directory"}
                ],
            },
            {
                "name": "11_credentials",
                "term": "11_credentials.rho",
                "captures": [
                    {"placeholder": "__CREDENTIAL_FACTORY_URI__", "from": "map", "key": "credential-factory"}
                ],
            },
            {
                "name": "30_oracle",
                "term": "30_oracle.rho",
                "captures": [
                    {"placeholder": "__ORACLE_FACTORY_URI__", "from": "map", "key": "oracle-factory"}
                ],
            },
            {"name": "40_provenance", "term": "40_provenance.rho"},
            {
                "name": "22_value",
                "term": "22_value.rho",
                "captures": [
                    {
                        "placeholder": "__VALUE_PROPOSITION_FACTORY_URI__",
                        "from": "map",
                        "key": "value-proposition-factory",
                    }
                ],
            },
            {"name": "20_claim", "term": "20_claim.rho"},
        ],
        "requiredKeys": [
            "component:root",
            "component:identity-directory",
            "component:identity-factory",
            "component:issuer",
            "component:credential-factory",
            "component:oracle-factory",
            "component:provenance-factory",
            "component:value-proposition-factory",
            "component:claim-factory",
        ],
    },
    "r-share": {
        "dir": "RShare",
        "repo": "https://github.com/rhovision-org/r-share",
        "commit": "7114acbabba31d3575c7c858c74f393755c91404",
        "sourceRoot": "contracts/build",
        "rootKey": "component:root",
        "phlo": {"limit": 4_000_000, "price": 1},
        "dependencies": ["r-cash"],
        "steps": [
            {"name": "00_root", "term": "00_root.rho", "root": True},
            {"name": "01_kernel", "term": "01_kernel.rho"},
            {"name": "02_admin", "term": "02_admin.rho"},
            {"name": "40_payments", "term": "40_payments.rho"},
            {"name": "41_hold_deposit", "term": "41_hold_deposit.rho"},
            {"name": "03_terms", "term": "03_terms.rho"},
            {"name": "04_remedy", "term": "04_remedy.rho"},
            {"name": "10_provider", "term": "10_provider.rho"},
            {"name": "13_unit", "term": "13_unit.rho"},
            {"name": "14_listing_contract", "term": "14_listing_contract.rho"},
            {"name": "15_bond", "term": "15_bond.rho"},
            {"name": "34_booking_instance", "term": "34_booking_instance.rho"},
            {"name": "22_booking_contract", "term": "22_booking_contract.rho"},
            {"name": "23_guest_ledger", "term": "23_guest_ledger.rho"},
            {"name": "20_booking", "term": "20_booking.rho"},
        ],
        "requiredKeys": [
            "component:root",
            "component:clock",
            "component:codec",
            "component:verifier",
            "component:admin",
            "component:admin-guard",
            "component:terms",
            "component:remedy",
            "component:provider-book",
            "component:unit-factory",
            "component:listing-factory",
            "component:bond-factory",
            "component:guest-book",
            "component:booking-factory",
            "component:guest-ledger-factory",
            "component:booking-instance-factory",
            "component:payments",
            "component:hold-deposit",
        ],
    },
    "r-sure": {
        "dir": "RSure",
        "repo": "https://github.com/rhovision-org/r-sure",
        "commit": "0da5d0436e5f33eec3de20adb887ece4b1edcad3",
        "sourceRoot": "contracts/build",
        "rootKey": "component:root",
        "phlo": {"limit": 4_000_000, "price": 1},
        "dependencies": ["r-cash", "r-gen"],
        "steps": [
            {"name": "00_root", "term": "00_root.rho", "root": True},
            {"name": "01_kernel", "term": "01_kernel.rho"},
            {"name": "02_admin", "term": "02_admin.rho"},
            {"name": "03_terms", "term": "03_terms.rho"},
            {"name": "04_math", "term": "04_math.rho"},
            {"name": "05_rgen", "term": "05_rgen.rho"},
            {"name": "40_payments", "term": "40_payments.rho"},
            {"name": "15_distribution", "term": "15_distribution.rho"},
            {"name": "16_fund", "term": "16_fund.rho"},
            {"name": "10_syndicate", "term": "10_syndicate.rho"},
            {"name": "11_broker", "term": "11_broker.rho"},
            {"name": "12_risk", "term": "12_risk.rho"},
            {"name": "13_slip", "term": "13_slip.rho"},
            {"name": "14_line", "term": "14_line.rho"},
            {"name": "17_policy", "term": "17_policy.rho"},
            {"name": "18_claim", "term": "18_claim.rho"},
            {"name": "19_books", "term": "19_books.rho"},
        ],
        "requiredKeys": [
            "component:root",
            "component:clock",
            "component:codec",
            "component:verifier",
            "component:admin",
            "component:admin-guard",
            "component:terms",
            "component:suremath",
            "component:rgen",
            "component:syndicate-book",
            "component:broker-book",
            "component:risk-book",
            "component:slip-factory",
            "component:line-factory",
            "component:distribution-factory",
            "component:fund",
            "component:policy-factory",
            "component:claim-factory",
            "component:claim-book",
            "component:policy-book",
            "component:variance-signal-book",
            "component:payments",
        ],
    },
    "r-print": {
        "dir": "RPrint",
        "repo": "https://github.com/rhovision-org/r-print",
        "commit": "a2d91b344ec5d5ece38fff579467c2cf57c82dae",
        "sourceRoot": "contracts/build",
        "rootKey": "component:root",
        "phlo": {"limit": 4_000_000, "price": 1},
        "dependencies": ["r-cash", "r-gen", "r-sure"],
        "steps": [
            {"name": "00_root", "term": "00_root.rho", "root": True},
            {"name": "01_kernel", "term": "01_kernel.rho"},
            {"name": "02_admin", "term": "02_admin.rho"},
            {"name": "40_payments", "term": "40_payments.rho"},
            {"name": "03_terms", "term": "03_terms.rho"},
            {"name": "05_rgen", "term": "05_rgen.rho"},
            {"name": "06_rsure", "term": "06_rsure.rho"},
            {"name": "10_publication", "term": "10_publication.rho"},
            {"name": "11_publication_cell", "term": "11_publication_cell.rho"},
            {"name": "20_commission", "term": "20_commission.rho"},
            {"name": "21_commission_escrow", "term": "21_commission_escrow.rho"},
            {"name": "30_article", "term": "30_article.rho"},
            {"name": "31_edition", "term": "31_edition.rho"},
            {"name": "32_coordination", "term": "32_coordination.rho"},
            {"name": "33_coordination_cell", "term": "33_coordination_cell.rho"},
        ],
        "requiredKeys": [
            "component:root",
            "component:clock",
            "component:codec",
            "component:verifier",
            "component:admin",
            "component:admin-guard",
            "component:terms",
            "component:rgen-adapter",
            "component:rsure-adapter",
            "component:publication-factory",
            "component:publication-cell-factory",
            "component:commission-factory",
            "component:commission-escrow-factory",
            "component:article-factory",
            "component:edition-factory",
            "component:coordination-factory",
            "component:coordination-cell-factory",
            "component:payments",
        ],
    },
}

PLACEHOLDER_RE = re.compile(r"__[A-Z][A-Z0-9_]*__")


def run_git(repo: Path, *args: str) -> bytes:
    proc = subprocess.run(
        ["git", "-C", str(repo), *args],
        capture_output=True,
        check=False,
    )
    if proc.returncode != 0:
        raise SystemExit(
            f"vendor-install: git {' '.join(args)} failed in {repo}: "
            f"{proc.stderr.decode(errors='replace').strip()}"
        )
    return proc.stdout


def plan_path(app: str, commit: str) -> Path:
    return INSTALL_ROOT / app / commit


def load_plan(app: str) -> tuple[dict[str, Any], Path]:
    spec = PLANS[app]
    base = plan_path(app, spec["commit"])
    path = base / "plan.json"
    if not path.is_file():
        raise SystemExit(f"vendor-install: missing {path}; run --write")
    return json.loads(path.read_text(encoding="utf-8")), base


def write_app(app: str, spec: dict[str, Any], quiet: bool = False) -> list[str]:
    repo = SIBLING_ROOT / spec["dir"]
    base = plan_path(app, spec["commit"])
    terms_dir = base / "terms"
    terms_dir.mkdir(parents=True, exist_ok=True)
    origin_terms: dict[str, dict[str, str]] = {}
    steps: list[dict[str, Any]] = []
    all_placeholders: set[str] = set()

    for step in spec["steps"]:
        source = f"{spec['sourceRoot']}/{step['term']}"
        data = run_git(repo, "show", f"{spec['commit']}:{source}")
        text = data.decode("utf-8")
        placeholders = sorted(set(PLACEHOLDER_RE.findall(text)))
        all_placeholders.update(placeholders)
        digest = hashlib.sha256(data).hexdigest()
        target = terms_dir / step["term"]
        target.write_bytes(data)
        origin_terms[step["term"]] = {
            "path": source,
            "sha256": digest,
        }
        rendered = dict(step)
        rendered["placeholders"] = placeholders
        steps.append(rendered)
        if not quiet:
            print(f"{app}: {step['term']} ({len(data)} bytes, {digest[:12]}…)")

    plan = {
        "schema": SCHEMA,
        "app": app,
        "repo": spec["repo"],
        "commit": spec["commit"],
        "sourceRoot": spec["sourceRoot"],
        "rootKey": spec["rootKey"],
        "resolveArity": spec.get("resolveArity", 3),
        "phlo": spec["phlo"],
        "dependencies": spec["dependencies"],
        "requiredKeys": spec["requiredKeys"],
        "placeholders": sorted(all_placeholders),
        "steps": steps,
    }
    (base / "plan.json").write_text(json.dumps(plan, indent=2) + "\n", encoding="utf-8")
    origin = {
        "schema": ORIGIN_SCHEMA,
        "app": app,
        "repo": spec["repo"],
        "commit": spec["commit"],
        "sourceRoot": spec["sourceRoot"],
        "license": "AGPL-3.0-only",
        "terms": origin_terms,
    }
    (base / "ORIGIN.json").write_text(json.dumps(origin, indent=2) + "\n", encoding="utf-8")
    return sorted(origin_terms)


def check_app(app: str) -> list[str]:
    plan, base = load_plan(app)
    errors: list[str] = []
    origin_path = base / "ORIGIN.json"
    if not origin_path.is_file():
        return [f"{app}: missing ORIGIN.json"]
    origin = json.loads(origin_path.read_text(encoding="utf-8"))
    for step in plan["steps"]:
        term = step["term"]
        path = base / "terms" / term
        if not path.is_file():
            errors.append(f"{app}: missing term {term}")
            continue
        data = path.read_bytes()
        digest = hashlib.sha256(data).hexdigest()
        recorded = origin.get("terms", {}).get(term, {}).get("sha256")
        if digest != recorded:
            errors.append(f"{app}: {term} hash {digest[:12]} != {recorded[:12] if recorded else 'missing'}")
        text = data.decode("utf-8")
        leftover = sorted(set(PLACEHOLDER_RE.findall(text)) - set(plan.get("placeholders", [])))
        if leftover:
            errors.append(f"{app}: {term} has unplanned placeholders {leftover}")
    if errors:
        return errors
    repo = SIBLING_ROOT / PLANS[app]["dir"]
    if repo.is_dir():
        for step in plan["steps"]:
            source = f"{plan['sourceRoot']}/{step['term']}"
            try:
                pinned = run_git(repo, "show", f"{plan['commit']}:{source}")
            except SystemExit as error:
                errors.append(str(error))
                break
            local = (base / "terms" / step["term"]).read_bytes()
            if hashlib.sha256(pinned).digest() != hashlib.sha256(local).digest():
                errors.append(f"{app}: {step['term']} differs from pinned blob")
    return errors


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--write", action="store_true", help="write plans and terms")
    parser.add_argument("--check", action="store_true", help="verify committed terms")
    parser.add_argument("--app", choices=sorted(PLANS))
    parser.add_argument("--json", action="store_true")
    args = parser.parse_args(argv)
    if not args.write and not args.check:
        parser.error("pass --write or --check")
    apps = [args.app] if args.app else sorted(PLANS)
    report: dict[str, Any] = {"schema": "rhobuilder.vendor-install/v1", "ok": True, "apps": {}}
    if args.write:
        for app in apps:
            terms = write_app(app, PLANS[app], quiet=args.json)
            report["apps"][app] = {"written": len(terms), "commit": PLANS[app]["commit"]}
        INSTALL_ROOT.mkdir(parents=True, exist_ok=True)
        index = {
            "schema": "rhobuilder.install-index/v1",
            "apps": {app: PLANS[app]["commit"] for app in sorted(PLANS)},
        }
        (INSTALL_ROOT / "index.json").write_text(
            json.dumps(index, indent=2) + "\n", encoding="utf-8"
        )
    if args.check:
        errors = []
        for app in apps:
            errs = check_app(app)
            report["apps"].setdefault(app, {})["ok"] = not errs
            report["apps"][app]["errors"] = errs
            errors.extend(errs)
        report["ok"] = not errors
    if args.json:
        print(json.dumps(report, indent=2))
    else:
        print(f"vendor-install: {'ok' if report['ok'] else 'failed'} ({len(apps)} app(s))")
        for app, entry in report["apps"].items():
            print(f"  {app}: {entry}")
    return 0 if report["ok"] else 1


if __name__ == "__main__":
    raise SystemExit(main())

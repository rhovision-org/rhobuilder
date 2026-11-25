#!/usr/bin/env python3
"""Behavioural suites for the bookable-service example.

Exit codes: 0 all required suites pass, 1 any failure, 2 any not-run and no failure.
The result file is deployments/suite-results.json.
"""

from __future__ import annotations

import concurrent.futures
import json
import sys
import time
from pathlib import Path

import app

ROOT = Path(__file__).resolve().parent.parent
RESULTS = ROOT / "deployments" / "suite-results.json"


def suite_root(root: str) -> None:
    keys = [
        "component:root",
        "component:dispute-factory",
        "component:payments",
        "component:booking-adapter",
        "component:cover-adapter",
    ]
    for key in keys:
        reply = app.resolve(root, key)
        app.expect(reply and reply[0] is True, f"{key}: {reply!r}")
        app.expect(
            isinstance(reply[1], str) and reply[1].startswith("rho:id:"),
            f"{key} did not return a URI: {reply!r}",
        )


def suite_payments(root: str) -> None:
    reply = app.resolve(root, "component:payments")
    uri = app.uri_of(reply)
    health = app.invoke(uri, "health")
    app.expect(health and health[0] is True, f"payments health: {health!r}")
    app.expect(
        isinstance(health[1], str) and health[1].startswith("rho:id:"),
        f"payments did not resolve the account factory: {health!r}",
    )
    account = app.invoke(uri, "ensureAccount", app.rq(app.deployer_public()))
    app.expect(account and account[0] is True, f"ensureAccount: {account!r}")


def suite_booking(root: str) -> None:
    uri = app.uri_of(app.resolve(root, "component:booking-adapter"))
    health = app.invoke(uri, "health")
    app.expect(health and health[0] is True, f"booking health: {health!r}")
    for item in health[1:3]:
        app.expect(
            isinstance(item, list)
            and item
            and item[0] is True
            and isinstance(item[1], str)
            and item[1].startswith("rho:id:"),
            f"booking adapter item: {item!r}",
        )


def suite_cover(root: str) -> None:
    uri = app.uri_of(app.resolve(root, "component:cover-adapter"))
    health = app.invoke(uri, "health")
    app.expect(health and health[0] is True, f"cover health: {health!r}")
    for item in health[1:3]:
        app.expect(
            isinstance(item, list)
            and item
            and item[0] is True
            and isinstance(item[1], str)
            and item[1].startswith("rho:id:"),
            f"cover adapter item: {item!r}",
        )


def open_dispute(root: str, dispute_id: str, amount: int = 100) -> str:
    factory = app.uri_of(app.resolve(root, "component:dispute-factory"))
    reply = app.invoke(
        factory, "open", f'{app.rq(dispute_id)}, "guest", "provider", {amount}'
    )
    return app.uri_of(reply)


def suite_dispute(root: str) -> None:
    uri = open_dispute(root, f"d-{time.time_ns()}")
    panel = app.invoke(uri, "selectPanel", '{"p1": 10, "p2": 8}')
    app.expect(panel and panel[0] is True, f"selectPanel: {panel!r}")
    app.expect(set(panel[1]) == {"p1", "p2"}, f"unexpected panel: {panel!r}")
    first = app.invoke(uri, "recordVerdict", '"p1", "allow"')
    app.expect(first and first[0] is True, f"first vote: {first!r}")
    again = app.invoke(uri, "recordVerdict", '"p1", "deny"')
    app.expect(again and again[0] is False, f"double vote: {again!r}")
    second = app.invoke(uri, "recordVerdict", '"p2", "allow"')
    app.expect(second and second[0] is True, f"second vote: {second!r}")
    state = app.invoke(uri, "get")
    app.expect(isinstance(state, dict) and state.get("state") == "resolved", f"state: {state!r}")
    settled = app.invoke(uri, "settle", "60, 40")
    app.expect(settled and settled[0] is True, f"settle: {settled!r}")


def suite_refusal(root: str) -> None:
    dispute_id = f"r-{time.time_ns()}"
    uri = open_dispute(root, dispute_id)
    provider = app.invoke(uri, "selectPanel", '{"provider": 10}')
    app.expect(provider and provider[0] is False, f"provider on panel: {provider!r}")
    empty = app.invoke(uri, "selectPanel", "{}")
    app.expect(empty and empty[0] is False, f"empty panel: {empty!r}")
    good = app.invoke(uri, "selectPanel", '{"p1": 10}')
    app.expect(good and good[0] is True, f"valid panel: {good!r}")
    outsider = app.invoke(uri, "recordVerdict", '"nobody", "allow"')
    app.expect(outsider and outsider[0] is False, f"non-panelist: {outsider!r}")
    duplicate = app.invoke(
        app.uri_of(app.resolve(root, "component:dispute-factory")),
        "open",
        f'{app.rq(dispute_id)}, "guest", "provider", 100',
    )
    app.expect(duplicate and duplicate[0] is False, f"duplicate open: {duplicate!r}")


def suite_idempotency(root: str) -> None:
    dispute_id = f"i-{time.time_ns()}"
    uri = open_dispute(root, dispute_id)
    factory = app.uri_of(app.resolve(root, "component:dispute-factory"))
    duplicate = app.invoke(
        factory, "open", f'{app.rq(dispute_id)}, "guest", "provider", 100'
    )
    app.expect(duplicate and duplicate[0] is False, f"duplicate open: {duplicate!r}")
    app.invoke(uri, "selectPanel", '{"p1": 10}')
    first = app.invoke(uri, "recordVerdict", '"p1", "allow"')
    app.expect(first and first[0] is True, f"vote: {first!r}")
    replay = app.invoke(uri, "recordVerdict", '"p1", "allow"')
    app.expect(replay and replay[0] is False, f"replay: {replay!r}")
    payments = app.uri_of(app.resolve(root, "component:payments"))
    one = app.invoke(payments, "ensureAccount", app.rq(app.deployer_public()))
    two = app.invoke(payments, "ensureAccount", app.rq(app.deployer_public()))
    app.expect(one and one[0] is True, f"ensureAccount one: {one!r}")
    app.expect(two and two[0] is True, f"ensureAccount two: {two!r}")


def suite_conservation(root: str) -> None:
    uri = open_dispute(root, f"c-{time.time_ns()}", amount=100)
    app.invoke(uri, "selectPanel", '{"p1": 10}')
    app.invoke(uri, "recordVerdict", '"p1", "allow"')
    bad = app.invoke(uri, "settle", "60, 50")
    app.expect(bad and bad[0] is False, f"non-conserved settle: {bad!r}")
    good = app.invoke(uri, "settle", "60, 40")
    app.expect(good and good[0] is True, f"conserved settle: {good!r}")
    state = app.invoke(uri, "get")
    app.expect(isinstance(state, dict) and state.get("settled") == 60, f"settled: {state!r}")
    app.expect(isinstance(state, dict) and state.get("refund") == 40, f"refund: {state!r}")


def suite_concurrency(root: str) -> None:
    factory = app.uri_of(app.resolve(root, "component:dispute-factory"))
    dispute_id = f"x-{time.time_ns()}"
    term = f'{app.rq(dispute_id)}, "guest", "provider", 100'
    with concurrent.futures.ThreadPoolExecutor(max_workers=2) as pool:
        results = list(
            pool.map(
                lambda _: app.invoke(factory, "open", term),
                range(2),
            )
        )
    winners = [item for item in results if item and item[0] is True]
    app.expect(len(winners) == 1, f"duplicate ids both won: {results!r}")
    uri = app.uri_of(winners[0])
    app.invoke(uri, "selectPanel", '{"p1": 10, "p2": 8}')
    with concurrent.futures.ThreadPoolExecutor(max_workers=2) as pool:
        votes = list(
            pool.map(
                lambda pair: app.invoke(
                    uri, "recordVerdict", f'"{pair[0]}", "{pair[1]}"'
                ),
                [("p1", "allow"), ("p2", "allow")],
            )
        )
    app.expect(all(item and item[0] is True for item in votes), f"votes: {votes!r}")
    outsider = app.invoke(uri, "recordVerdict", '"p3", "allow"')
    app.expect(outsider and outsider[0] is False, f"outsider: {outsider!r}")


SUITES = {
    "root": suite_root,
    "payments": suite_payments,
    "booking": suite_booking,
    "cover": suite_cover,
    "dispute": suite_dispute,
    "refusal": suite_refusal,
    "idempotency": suite_idempotency,
    "conservation": suite_conservation,
    "concurrency": suite_concurrency,
}


def main() -> int:
    results = []
    try:
        app.require_devnet()
        root = app.app_root()
    except app.NotRun as exc:
        print(f"unit-all: not_run ({exc})")
        payload = {
            "schema": "rhobuilder.suite-results/v1",
            "suite": "bookable-service",
            "status": "not_run",
            "reason": str(exc),
            "results": [],
        }
        RESULTS.parent.mkdir(parents=True, exist_ok=True)
        RESULTS.write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")
        return 2
    for name, suite in SUITES.items():
        started = time.time()
        try:
            suite(root)
            status = "pass"
            detail = ""
        except app.NotRun as exc:
            status = "not_run"
            detail = str(exc)
        except Exception as exc:  # noqa: BLE001 - suite failures are data
            status = "fail"
            detail = f"{type(exc).__name__}: {exc}"
        results.append(
            {
                "suite": name,
                "status": status,
                "detail": detail,
                "seconds": round(time.time() - started, 3),
            }
        )
        print(f"--- {name}: {status} ---")
        if detail:
            print(f"  {detail}")
    payload = {
        "schema": "rhobuilder.suite-results/v1",
        "suite": "bookable-service",
        "generatedAt": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
        "results": results,
    }
    RESULTS.parent.mkdir(parents=True, exist_ok=True)
    RESULTS.write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")
    if any(item["status"] == "fail" for item in results):
        return 1
    if any(item["status"] == "not_run" for item in results):
        return 2
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

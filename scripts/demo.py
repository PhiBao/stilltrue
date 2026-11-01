#!/usr/bin/env python3
"""Run the seeded demo against a live StillTrue API and print the receipt.

Usage:
    python3 scripts/demo.py
    STILLTRUE_API=http://localhost:8788 python3 scripts/demo.py
"""
from __future__ import annotations

import json
import os
import sys
import time
import urllib.request

API = os.environ.get("STILLTRUE_API", "http://localhost:8788").rstrip("/")


def request(path: str, payload: dict | None = None) -> dict:
    data = None if payload is None else json.dumps(payload).encode()
    req = urllib.request.Request(
        f"{API}{path}",
        data=data,
        headers={"Content-Type": "application/json"},
        method="POST" if payload is not None else "GET",
    )
    with urllib.request.urlopen(req, timeout=30) as response:
        return json.load(response)


def main() -> int:
    demo = request("/api/demo")
    print(f"→ checking: {demo['title']}")
    created = request("/api/checks", {"text": demo["text"], "source": "demo"})
    check_id = created["id"]
    print(f"  check {check_id}")

    last = ""
    for _ in range(60):
        time.sleep(2)
        check = request(f"/api/checks/{check_id}")
        if check["stage_message"] != last:
            last = check["stage_message"]
            print(f"  {check['status']:<12} {last}")
        if check["status"] in ("done", "failed"):
            break
    else:
        print("timed out waiting for the check", file=sys.stderr)
        return 1

    if check["status"] == "failed":
        print(f"failed: {check.get('error')}", file=sys.stderr)
        return 1

    print(f"\nfreshness: {check['freshness_score']}/100")
    print(
        f"claims: {check['claim_count']} · changed {check['changed_count']} · "
        f"supported {check['supported_count']} · "
        f"unverifiable {check['unverifiable_count']} · stable {check['stable_count']}"
    )
    print("\nverdicts:")
    for claim in check["claims"]:
        print(
            f"  {claim['id']}  {claim['verdict']:<13} "
            f"{claim['explanation'][:90]}"
        )
        if claim["corrected_text"]:
            print(f"      now: {claim['corrected_text']}")

    proof = check["proof"]
    print("\nreceipt:")
    print(f"  route      {proof['provider']} / {proof['model']}")
    print(f"  failover   {proof['fallback_used']} ({proof['fallback_reason'] or 'primary held'})")
    print(f"  tokens     {proof['prompt_tokens']} in / {proof['completion_tokens']} out")
    print(f"  tavily     {proof['tavily_searches']} searches · {proof['tavily_credits']} credits")
    print(f"  wall time  {proof['duration_ms'] / 1000:.1f}s")
    print(f"  share      {os.environ.get('STILLTRUE_WEB', 'http://localhost:3100')}/check/{check_id}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

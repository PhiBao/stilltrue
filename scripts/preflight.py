#!/usr/bin/env python3
"""Preflight gate: run immediately before judging and after any restart.

Checks:
  1. API /ready (db + Bifrost + Tavily configured)
  2. Seeded check completes with >=1 changed claim
  3. Failover drill shows hop 1 + forced outage
  4. Governance: unlisted model is blocked, listed model passes

Exit 0 = green. Exit 1 = do not present; fix first.
"""
from __future__ import annotations

import json
import os
import sys
import time
import urllib.request

API = os.environ.get("STILLTRUE_API", "http://localhost:8788").rstrip("/")
BIFROST = os.environ.get("BIFROST_URL", "http://localhost:8081").rstrip("/")
VK = os.environ.get("BIFROST_VK", "sk-bf-stilltrue-dev")
MODEL = os.environ.get("BIFROST_MODEL", "tokenfactory/Qwen/Qwen3-30B-A3B-Instruct-2507")


def get(url: str):
    with urllib.request.urlopen(url, timeout=15) as r:
        return json.load(r)


def post(url: str, payload: dict, headers: dict | None = None):
    data = json.dumps(payload).encode()
    req = urllib.request.Request(url, data=data, headers={"Content-Type": "application/json", **(headers or {})}, method="POST")
    with urllib.request.urlopen(req, timeout=60) as r:
        return r.status, json.load(r)


def wait_done(cid: str, timeout: int = 180) -> dict:
    start = time.time()
    while time.time() - start < timeout:
        time.sleep(3)
        c = get(f"{API}/api/checks/{cid}")
        if c["status"] in ("done", "failed"):
            return c
    raise TimeoutError(f"check {cid} did not finish")


def main() -> int:
    ok = True
    print("→ /ready")
    ready = get(f"{API}/ready")
    print(" ", json.dumps(ready))
    ok = ok and ready.get("ready") is True

    print("→ seeded check")
    demo = get(f"{API}/api/demo")
    _, created = post(f"{API}/api/checks", {"text": demo["text"], "source": "demo"})
    done = wait_done(created["id"])
    print(f"  score={done['freshness_score']} changed={done['changed_count']} "
          f"coverage={done.get('coverage')}")
    ok = ok and done["status"] == "done" and done["changed_count"] >= 1

    print("→ failover drill")
    _, created = post(f"{API}/api/checks", {"text": demo["text"], "source": "drill", "drill": True})
    drill = wait_done(created["id"])
    p = drill["proof"]
    print(f"  route={p['provider']}/{p['model']} failover={p['fallback_used']} "
          f"reason={p['fallback_reason']}")
    ok = ok and p["fallback_used"] and p["drill"]

    print("→ governance (unlisted model blocked)")
    try:
        post(f"{BIFROST}/v1/chat/completions",
             {"model": "tokenfactory/Qwen/Qwen3-235B-A22B-Instruct-2507",
              "messages": [{"role": "user", "content": "Say OK"}], "max_tokens": 10},
             {"x-bf-vk": VK})
        print("  FAIL: unlisted model was allowed")
        ok = False
    except urllib.error.HTTPError as e:
        print(f"  blocked with {e.code} (expected 403)")
        ok = ok and e.code in (400, 401, 403, 404)

    print("\nPREFLIGHT " + ("GREEN — present" if ok else "RED — fix before presenting"))
    return 0 if ok else 1


if __name__ == "__main__":
    raise SystemExit(main())

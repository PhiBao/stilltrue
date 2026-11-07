#!/usr/bin/env python3
"""Labeled eval strip: run live checks, print honest metrics.

Usage:
    python3 scripts/eval.py
"""
from __future__ import annotations
import json, os, sys, time, urllib.request

API = os.environ.get("STILLTRUE_API", "http://localhost:8788").rstrip("/")

def request(path: str, payload: dict | None = None) -> dict:
    data = None if payload is None else json.dumps(payload).encode()
    req = urllib.request.Request(f"{API}{path}", data=data,
        headers={"Content-Type": "application/json"},
        method="POST" if payload is not None else "GET")
    with urllib.request.urlopen(req, timeout=30) as r:
        return json.load(r)

def run_one(name: str, text: str) -> dict:
    created = request("/api/checks", {"text": text, "source": "eval"})
    cid = created["id"]
    for _ in range(60):
        time.sleep(2)
        c = request(f"/api/checks/{cid}")
        if c["status"] in ("done", "failed"):
            break
    return c

def main() -> int:
    cases = json.load(open("eval/eval_cases.json"))
    print(f"running {len(cases)} labeled checks against {API}\n")
    for case in cases:
        c = run_one(case["name"], case["text"])
        if c["status"] == "failed":
            print(f"{case['name']:<26} FAILED  {c.get('error')}")
            continue
        by = {}
        for cl in c["claims"]:
            by[cl["verdict"]] = by.get(cl["verdict"], 0) + 1
        print(f"{case['name']:<26} score={c['freshness_score']} "
              f"coverage={c.get('coverage')} claims={c['claim_count']} "
              f"{by}")
        for cl in c["claims"]:
            print(f"   {cl['id']} {cl['verdict']:<13} {cl['quote'][:80]}")
    return 0

if __name__ == "__main__":
    raise SystemExit(main())

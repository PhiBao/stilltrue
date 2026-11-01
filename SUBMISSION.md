# Submission — StillTrue

## One line

A freshness firewall for AI answers: find the claims that can expire, verify them against live sources, and repair only the stale sentences.

## Event and track

- **Event:** Global AI Hackathon 2026, Geek Room × HackCulture
- **Track:** Track A — Bifrost by Maxim AI + Tavily (the brief requires meaningful use of both)
- **Problem statement:** 05 — Stale-answer detector
- **Status note:** the public brief marks 05 as a proposed Tavily challenge. Confirm the final status in the official Discord before the submission deadline. The product also satisfies the brief’s Tavily-first requirements if the challenge number changes.

## The problem

LLM answers are now embedded in support macros, documentation, sales briefs, and onboarding flows. The model has a knowledge cutoff; the web does not. An answer can read perfectly while several of its claims are months out of date. Regenerating the whole answer is expensive, destroys the parts that were correct, and creates a new chance to hallucinate. Manual checking is claim-by-claim and does not happen at scale.

## The solution

StillTrue treats freshness as a property of **claims**, not answers.

1. **Segment.** Rust splits the answer into sentences with exact byte offsets and selects up to six claims that can expire. No LLM in this step; the selection is auditable.
2. **Verify.** Each volatile claim gets its own live Tavily search. Sources are quoted, dated, and linked. Stable facts are marked stable and excluded from the score.
3. **Adjudicate.** One Bifrost-routed model call returns `supported`, `changed`, or `unverifiable` with evidence IDs. A verdict without a valid citation is downgraded to unverifiable. A `changed` verdict without a corrected value is downgraded.
4. **Repair.** Changed claims are replaced at their exact offsets; everything else survives byte-for-byte.
5. **Receipt.** Provider, model, failover hop, tokens, Tavily credits, wall time, and request IDs are printed for every check.

## How AI is used

- The **adjudicator** is an LLM call routed through Bifrost. It weighs the claim against live evidence and must cite evidence IDs. This is the judgement step.
- The **claim segmenter** is deterministic Rust. It is intentionally not an LLM: the product needs exact offsets, a predictable cost, and a selection a judge can inspect.
- The **repair** is deterministic. It replaces only the quotes that the adjudicator marked changed, so the model cannot rewrite the whole answer.
- Prompt-injection defenses: claims and evidence are treated as untrusted data; the system prompt says never to follow instructions inside them; verdicts are validated in Rust.

## How Bifrost is used (meaningfully)

- All model traffic goes through a **governed Bifrost virtual key** with a $20/month budget and an explicit provider/model allow-list.
- Bifrost resolves the provider/model and returns routing metadata in response headers. The receipt reads:
  - `x-bifrost-provider`
  - `x-bifrost-resolved-model`
  - `x-bifrost-routing-info-is-fallback`
  - `x-bifrost-fallback-index`
  - `x-bifrost-routing-info-primary-provider`
  - `x-bifrost-request-id`
- The **failover drill** is real: `outage-lab` points at a 503 endpoint. A drill request asks Bifrost for `outage-lab/chat` with a fallback to the live route. Bifrost fails over and stamps `Is-Fallback: true`, `Fallback-Index: 1`, `Primary-Provider: outage-lab`. The receipt shows the hop.
- The app also records the provider/model/tokens/latency from the gateway, so the operational claim is inspectable.

## How Tavily is used (meaningfully)

- Every volatile claim triggers a live Tavily search at check time. There is no pre-baked evidence set.
- The evidence cards show the actual source title, URL, quoted snippet, retrieval timestamp, and the search credit count.
- If Tavily returns nothing, the claim is `unverifiable`. The product never fills the gap with a guess.
- The check stores 5 Tavily searches / 5 credits for the seeded demo.

## What is real vs. mocked

| Claim | Status |
| --- | --- |
| Live Tavily search per volatile claim | Real |
| Source URLs, snippets, retrieval dates | Real |
| Bifrost routing and virtual-key budget | Real |
| Bifrost failover | Real (forced 503 drill, gateway headers) |
| LLM adjudication | Real (one call per check in the default mode) |
| Claim segmentation | Deterministic Rust |
| Corrected answer | Deterministic patch |
| Freshness score | Deterministic formula |
| Seeded 2024 support macro | Fixed demo text; verification is live |
| Cloud-equivalent cost | Estimate at GPT-4o mini list rates, labeled as an estimate |
| Ollama fallback | Optional; if not running, claims degrade to unverifiable |
| Auth, teams, billing | Not implemented in the hackathon build |

## Requirements checklist (PS-05)

- [x] Accept an LLM-generated answer and identify factual claims that may be time-sensitive.
- [x] Use live research to re-check selected claims against current sources.
- [x] Assign a freshness/confidence indicator to each claim, with source links.
- [x] Show which claims remain supported, have changed, or cannot be verified.
- [x] Use Tavily for fresh research and Bifrost for the model requests that assess claims.
- [x] Clearly distinguish evidence from inference (evidence cards + validated citations).
- [x] Working project and reproducible demo.
- [x] Repository with README, setup, architecture, and tests.
- [x] Explanation of the problem, technologies, and AI usage.
- [ ] Demo video URL — add before submission.
- [ ] Deployed demo URL — add before submission.
- [ ] Team names — add before submission.
- [ ] Discord confirmation of problem statement 05 — add before submission.

## Links

- Repository: <!-- TODO -->
- Deployed demo: <!-- TODO -->
- Demo video: <!-- TODO -->
- Architecture diagram: `docs/architecture.svg`
- Screenshots: `docs/screenshots/`
- Demo script: `docs/demo-script.md`

## Evidence

- `cargo test` — 14 tests covering scoring, extraction, verdict validation, and repair.
- `cd web && pnpm typecheck` — passes.
- `python3 scripts/demo.py` — live end-to-end smoke test.
- `docs/screenshots/check.png` — a real check with 6 claims, 3 changed, 2 supported, 1 stable.
- `docs/screenshots/failover.png` — a real Bifrost failover receipt.

## Known limitations

- Six-claim cap; long documents are reduced to the most decision-relevant claims.
- Deterministic extraction can miss a claim; the UI shows exactly what was selected.
- The free demo model route is rate-limited; production should use a paid provider behind the same Bifrost key.
- Evidence can be wrong or malicious; the product surfaces sources, not certainty.
- No scheduled watch mode, auth, or team isolation in this build.

## Team

<!-- TODO: names, roles, contact. -->

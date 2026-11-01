# Demo recording script

Target length: **2:40–3:00**. Record at 1440×900, browser zoom 100%, bookmarks bar hidden. Use a clean browser profile with no extensions.

## Before recording

1. Start the full stack: `docker compose up -d` (or `make bifrost`, `make api`, `make web`).
2. Open `http://localhost:3100` once and run the seeded demo. This warms the model route and confirms Tavily is reachable.
3. Keep a completed check open in a second tab as a fallback if the live run is slow.
4. Have the failover check ready in a third tab (run it once before recording).
5. Close other tabs, notifications, and terminals with secrets.

If the primary model route is unavailable during recording, use the pre-run check and say: “The check is live; this is the completed evidence room from a run a moment ago.” Do not fake a result.

## Scene 1 — The hook (0:00–0:20)

**Screen:** Home page, hero.

**Narration:**
> “This answer was true in 2024. It is still sitting in a help center. Three of its claims are no longer true, and nobody has noticed.”

**Action:** Scroll slowly past the headline to the specimen. Let the “44 / freshness” and the red highlights land.

## Scene 2 — The problem (0:20–0:40)

**Screen:** Specimen, then the composer.

**Narration:**
> “AI answers are now customer-facing. The model has a knowledge cutoff; the web does not. Regenerating the answer destroys what was right and gives the model another chance to invent something. Checking it by hand is claim-by-claim, and nobody does it.”

**Action:** Click **Run this check** on the specimen.

## Scene 3 — The live check (0:40–1:15)

**Screen:** Check page. The scan line moves; claims appear; evidence cards fill in.

**Narration:**
> “StillTrue segments the answer in Rust with exact offsets. It only searches the claims that can expire. Tavily returns live sources. One Bifrost-routed model call weighs the evidence. A verdict without a citation is rejected.”

**Action:** Let the page finish. Point at the score, the strip, and the first changed claim.

**On-screen beat:** The first evidence card expands: “GPT-4o costs $2.50 per million input tokens and $10 per million output tokens.” The source is a live page retrieved today.

## Scene 4 — The catch (1:15–1:45)

**Screen:** Evidence margin, claim 1 and claim 5.

**Narration:**
> “Here is the old price: five dollars in, fifteen out. The current price is two-fifty in, ten out. The Assistants API is deprecated; the Responses API is recommended. Both corrections carry a source and a retrieval date.”

**Action:** Click claim 1, then claim 5. Scroll to the “What changed” list.

## Scene 5 — The repair (1:45–2:10)

**Screen:** Toggle **Repaired**.

**Narration:**
> “The repaired view changes only the contradicted sentences. The GPT-4o mini price and the Batch API discount were correct, so they survive byte-for-byte. This is a patch, not a rewrite.”

**Action:** Toggle **Original → Repaired** twice. Hover the green replaced sentences. Click **Copy repaired**.

## Scene 6 — The receipt and failover (2:10–2:40)

**Screen:** Gateway receipt.

**Narration:**
> “Every check prints a receipt: route, model, tokens, Tavily credits, wall time, request ID. And because the model route is governed by Bifrost, we can prove failover instead of claiming it.”

**Action:** Click **Test provider failover**. The new check completes. Point at:

```text
requested  chat
failover   yes · hop 1
reason     Gateway failed over from outage-lab/chat to tokenfactory/Qwen/Qwen3-30B-A3B-Instruct-2507.
drill      forced primary outage
```

**Narration:**
> “The primary route was forced to return a 503. Bifrost failed over to the live route, and the receipt shows the hop. No mock, no animation.”

## Scene 7 — Close (2:40–2:55)

**Screen:** Home page or the repaired answer.

**Narration:**
> “StillTrue. Answers expire. Proof doesn’t.”

**Action:** End on the wordmark and the repaired answer. Cut.

## Optional 20-second appendix

Show `http://localhost:3100/method` and the architecture diagram:

> “The extraction is deterministic, the adjudication is evidence-bound, and the gateway is observable. That is the whole system.”

## Fallback lines

- If Tavily is slow: “The check is waiting on live sources; this is the honest failure mode. If a source is missing, the claim becomes unverifiable, not green.”
- If the primary model route is unavailable: “The gateway failed over, or the claim became unverifiable. The free Pollinations fallback allows one new reply every 45 seconds; a production deployment keeps a paid provider behind the same key.”
- If a verdict looks wrong: “The evidence is on screen. The product’s job is to make the claim auditable, not to be infallible.”

## Do not

- Do not show the `.env` file, the Tavily key, or the Bifrost admin dashboard.
- Do not claim the score is a probability.
- Do not call the seeded answer “live model output”; call it a fixed 2024 support macro with live verification.
- Do not speed up or cut the failover receipt; it is the proof.

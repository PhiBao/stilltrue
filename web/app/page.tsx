import { Footer } from "@/components/Footer";
import { Header } from "@/components/Header";
import { Workbench } from "@/components/Workbench";

export default function Home() {
  return (
    <>
      <Header />
      <main>
        <section className="hero">
          <div className="shell hero-grid">
            <div>
              <div className="eyebrow">Frequency check for AI answers</div>
              <h1>
                Answers expire.
                <br />
                <em>Proof doesn&apos;t.</em>
              </h1>
            </div>
            <div className="hero-side">
              <p>
                StillTrue audits every time-sensitive claim in an AI answer
                against the live web, shows you exactly what changed, and
                repairs only the stale sentences.
              </p>
              <div className="hero-meta">
                <span>Claim-level</span>
                <span>Cited evidence</span>
                <span>Repair, not rewrite</span>
              </div>
            </div>
          </div>
        </section>

        <Workbench />

        <section className="steps-section" id="method">
          <div className="shell">
            <div className="section-head">
              <h2>Three moves. No blind trust.</h2>
              <p>
                Most tools regenerate the whole answer and hope. StillTrue
                isolates what can expire, verifies it, and changes only what
                the evidence contradicts.
              </p>
            </div>
            <div className="steps">
              <article className="step">
                <div className="step-num">01 / SEGMENT</div>
                <h3>Find the sentences that can rot.</h3>
                <p>
                  Prices, versions, limits, policies, leadership. Each claim
                  keeps an exact offset into the original answer, so every
                  highlight is auditable — not a paraphrase.
                </p>
                <span className="step-tag">deterministic offsets</span>
              </article>
              <article className="step">
                <div className="step-num">02 / VERIFY</div>
                <h3>Search only what can change.</h3>
                <p>
                  Tavily returns live sources for each volatile claim. The
                  adjudicator must cite evidence or say unverifiable. No
                  evidence, no verdict.
                </p>
                <span className="step-tag">Tavily · live web</span>
              </article>
              <article className="step">
                <div className="step-num">03 / REPAIR</div>
                <h3>Patch the stale sentence.</h3>
                <p>
                  Changed claims are replaced with the evidence-backed current
                  value. Everything that still holds is left exactly as written.
                  Bifrost routes and governs every model call.
                </p>
                <span className="step-tag">Bifrost · routed</span>
              </article>
            </div>
          </div>
        </section>

        <section className="steps-section" style={{ borderBottom: 0 }}>
          <div className="shell">
            <div className="section-head">
              <h2>The answer is the artifact.</h2>
              <p>
                A freshness score is not a vibe. It is a weighted average of
                cited claim verdicts. Open any source, inspect the receipt, and
                decide for yourself.
              </p>
            </div>
            <div className="steps" style={{ borderTopColor: "var(--rule)" }}>
              <article className="step">
                <div className="step-num">PROOF / 01</div>
                <h3>Then vs. now.</h3>
                <p>
                  Every changed claim carries the old value, the current value,
                  and the source that proves it. The correction is a drop-in
                  replacement, not a new answer.
                </p>
              </article>
              <article className="step">
                <div className="step-num">PROOF / 02</div>
                <h3>A gateway receipt.</h3>
                <p>
                  Provider, model, fallback event, token counts, Tavily credits,
                  and request IDs — the operational truth behind each check,
                  printed like a till receipt.
                </p>
              </article>
              <article className="step">
                <div className="step-num">PROOF / 03</div>
                <h3>Fail visible, not silent.</h3>
                <p>
                  When a source or a model is unavailable, the verdict becomes
                  unverifiable. The product never manufactures confidence to
                  keep the demo green.
                </p>
              </article>
            </div>
          </div>
        </section>
      </main>
      <Footer />
    </>
  );
}

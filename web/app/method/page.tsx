import Link from "next/link";
import { Footer } from "@/components/Footer";
import { Header } from "@/components/Header";

export const metadata = {
  title: "How a check happens — StillTrue",
};

export default function MethodPage() {
  return (
    <>
      <Header />
      <main className="shell method-page">
        <section className="method-hero">
          <div>
            <div className="eyebrow">Method · the whole system</div>
            <h1>A check is a small, inspectable system.</h1>
          </div>
          <p>
            No agent loop, no hidden retrieval, no second model rewriting the
            whole answer. StillTrue separates the mechanical work from the one
            judgement call that needs a model — and makes both visible.
          </p>
        </section>

        <figure>
          <div className="diagram-frame">
            {/* eslint-disable-next-line @next/next/no-img-element */}
            <img
              src="/diagrams/architecture.svg"
              alt="StillTrue architecture: answer, deterministic segmentation, Tavily verification, evidence bundle, Bifrost-routed adjudication, and surgical repair."
            />
          </div>
          <figcaption className="diagram-caption">
            <span>Architecture · one check</span>
            <span>
              Rust orchestrator · Tavily live sources · Bifrost gateway ·
              SQLite receipts
            </span>
          </figcaption>
        </figure>

        <div className="method-grid">
          <article className="method-item">
            <div className="step-num">01 / SEGMENT</div>
            <h3>The model never decides what is a claim.</h3>
            <p>
              The Rust service splits the answer into sentences and keeps exact
              byte offsets. It scores each sentence for factual density and
              time-sensitivity, then selects at most six. A judge can inspect
              the selection; there is no prompt-shaped black box in this step.
            </p>
          </article>
          <article className="method-item">
            <div className="step-num">02 / VERIFY</div>
            <h3>Only volatile claims cost a search.</h3>
            <p>
              Prices, versions, limits, policies and recommendations are sent to
              Tavily. Stable facts such as a founding date are marked stable and
              left out of the score. Each volatile claim gets its own query and
              up to four live sources with quoted snippets.
            </p>
          </article>
          <article className="method-item">
            <div className="step-num">03 / ADJUDICATE</div>
            <h3>Cite, change, or abstain.</h3>
            <p>
              One Bifrost-routed call receives the claims and their evidence.
              A verdict is only accepted when it cites a real evidence ID. A
              &ldquo;changed&rdquo; verdict must include a drop-in corrected
              value; otherwise it is downgraded to unverifiable. No evidence,
              no verdict.
            </p>
          </article>
          <article className="method-item">
            <div className="step-num">04 / REPAIR</div>
            <h3>The rest of the answer is untouched.</h3>
            <p>
              Corrections replace the exact quote of each changed claim, working
              backwards through the document so offsets stay valid. The output
              is a surgical patch, not a regeneration. Supported and stable
              sentences survive byte-for-byte.
            </p>
          </article>
        </div>

        <h2 className="method-section-title">What is actually running.</h2>
        <div className="real-grid">
          <article className="real-item">
            <h4>Real · Tavily</h4>
            <p>
              Every volatile claim triggers a live Tavily search at check time.
              Source URLs, snippets, retrieval timestamps and credit counts are
              stored with the check. There is no pre-baked evidence set.
            </p>
          </article>
          <article className="real-item">
            <h4>Real · Bifrost</h4>
            <p>
              All model traffic goes through a governed Bifrost virtual key.
              Provider, resolved model, failover hop, token counts and request
              IDs come from Bifrost response headers and are printed in the
              receipt.
            </p>
          </article>
          <article className="real-item">
            <h4>Real · failure</h4>
            <p>
              If Tavily returns nothing, the claim is unverifiable. If the
              primary model route fails, the gateway failover is recorded, and
              an application-level takeover is labeled as such. The product
              never fills gaps with invented confidence.
            </p>
          </article>
        </div>

        <h2 className="method-section-title">Why Bifrost and Tavily, together.</h2>
        <p className="method-prose">
          Tavily is the product&apos;s source of truth: without live retrieval,
          a freshness check is just another model guessing. Bifrost is the
          product&apos;s source of operational truth: it makes the model call
          governable, observable and resilient, and it produces the receipt a
          reviewer needs to trust the verdict. The two are not interchangeable
          add-ons; removing either breaks the loop. A search without
          adjudication is a pile of links. An adjudication without live search
          is a model asserting its training data.
        </p>

        <h2 className="method-section-title">Deliberate limits.</h2>
        <p className="method-prose">
          A check accepts up to 12,000 characters and verifies at most six
          claims, four sources each. These caps keep a check under a minute and
          keep Tavily credit use predictable. The demo deployment uses
          TokenFactory (Nebius) with a metered key; Pollinations remains a free
          fallback. The architecture is provider-agnostic and a production
          deployment can point the same Bifrost virtual key at a frontier model. The scoring weights are explicit: supported and
          stable count 1.0, unverifiable 0.5, changed 0.0, weighted by claim
          importance. The score is a summary of cited verdicts, not a
          probability.
        </p>

        <p style={{ marginTop: 34 }}>
          <Link className="btn btn--primary" href="/">
            Run a check <span aria-hidden="true">→</span>
          </Link>
        </p>
      </main>
      <Footer />
    </>
  );
}

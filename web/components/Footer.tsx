import Link from "next/link";

export function Footer() {
  return (
    <footer className="site-footer">
      <div className="shell footer-inner">
        <div>
          <div className="wordmark" style={{ fontSize: 18, gap: 0 }}>
            <span>
              StillTrue<span className="q">?</span>
            </span>
          </div>
          <p className="footer-note" style={{ marginTop: 10 }}>
            A freshness firewall for AI answers. Built for the Global AI
            Hackathon 2026 with Bifrost by Maxim AI and Tavily.
          </p>
        </div>
        <div className="footer-links">
          <Link href="/method">How it works</Link>
          <Link href="/history">History</Link>
          <a
            href="https://github.com/maximhq/bifrost"
            target="_blank"
            rel="noreferrer"
          >
            Bifrost ↗
          </a>
          <a href="https://tavily.com" target="_blank" rel="noreferrer">
            Tavily ↗
          </a>
        </div>
      </div>
    </footer>
  );
}

import Link from "next/link";
import { Mark } from "./Mark";

export function Header({ compact = false }: { compact?: boolean }) {
  return (
    <header className="site-header">
      <div className="shell header-inner">
        <Link href="/" className="wordmark" aria-label="StillTrue home">
          <span className="mark">
            <Mark size={17} />
          </span>
          <span>
            StillTrue<span className="q">?</span>
          </span>
        </Link>
        <nav className="nav" aria-label="Main">
          {!compact && (
            <>
              <Link href="/#method">Method</Link>
              <Link href="/history">History</Link>
              <Link href="/method">Architecture</Link>
            </>
          )}
          {compact && (
            <>
              <Link href="/">New check</Link>
              <Link href="/history">History</Link>
            </>
          )}
          <span className="live">live sources</span>
        </nav>
      </div>
    </header>
  );
}

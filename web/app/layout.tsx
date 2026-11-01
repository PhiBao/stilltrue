import type { Metadata, Viewport } from "next";
import "./globals.css";

export const metadata: Metadata = {
  title: "StillTrue — freshness firewall for AI answers",
  description:
    "StillTrue audits every time-sensitive claim in an AI answer against live sources, shows what changed, and repairs only the stale sentences.",
  applicationName: "StillTrue",
  icons: {
    icon: [
      { url: "/favicon.svg", type: "image/svg+xml" },
      { url: "/favicon-32.png", sizes: "32x32", type: "image/png" },
      { url: "/favicon.ico", sizes: "any" },
    ],
    apple: [{ url: "/apple-icon.png", sizes: "180x180", type: "image/png" }],
  },
  openGraph: {
    title: "StillTrue — answers expire. Proof doesn't.",
    description:
      "A claim-level freshness check for AI answers. Live sources, cited verdicts, surgical repair.",
    type: "website",
    images: [
      {
        url: "/og.png",
        width: 1200,
        height: 630,
        alt: "StillTrue — answers expire. Proof doesn't.",
      },
    ],
  },
  twitter: {
    card: "summary_large_image",
    title: "StillTrue — answers expire. Proof doesn't.",
    description:
      "A claim-level freshness check for AI answers. Live sources, cited verdicts, surgical repair.",
    images: ["/og.png"],
  },
};

export const viewport: Viewport = {
  themeColor: "#f4efe4",
  colorScheme: "light",
};

export default function RootLayout({
  children,
}: Readonly<{ children: React.ReactNode }>) {
  return (
    <html lang="en">
      <head>
        <link
          rel="preload"
          href="/fonts/fraunces-latin.woff2"
          as="font"
          type="font/woff2"
          crossOrigin=""
        />
        <link
          rel="preload"
          href="/fonts/instrument-sans-latin.woff2"
          as="font"
          type="font/woff2"
          crossOrigin=""
        />
      </head>
      <body>{children}</body>
    </html>
  );
}

# Deploying StillTrue

The local demo is a complete stack. A public deployment needs one extra piece: a reverse proxy that puts the web app and the API on one origin and keeps the gateway private.

## Recommended: one VM, Docker Compose, Caddy

### 1. DNS

Point an `A` record at the VM:

```text
stilltrue.example.com → 203.0.113.10
```

Open ports **80** and **443** only.

### 2. Environment

```bash
git clone <your-repo> stilltrue
cd stilltrue
cp .env.example .env
```

Set at least:

```dotenv
TAVILY_API_KEY=tvly-...
TOKENFACTORY_API_KEY=tf-...
DOMAIN=stilltrue.example.com
WEB_ORIGIN=https://stilltrue.example.com
```

Do **not** expose `BIFROST_URL`, `BIFROST_VK`, or the Tavily key to the browser. They are server-side only.

### 3. Start the production stack

```bash
docker compose \
  -f docker-compose.yml \
  -f docker-compose.prod.yml \
  up -d --build
```

The production override (requires Docker Compose 2.24+ for `!reset`):

- adds Caddy on ports 80/443 with automatic TLS
- removes the public port mappings for the API, web, and Bifrost
- builds the web app with `NEXT_PUBLIC_API_BASE=""` so the browser calls the same origin at `/api/*`
- routes `/api/*` to the Rust API and everything else to Next.js

Caddy streams SSE (`flush_interval -1`), so the check page receives live progress. The frontend also has a polling fallback.

### 4. Verify

```bash
curl -s https://stilltrue.example.com/api/health
# {"ok":true,"service":"stilltrue-api"}

curl -sI https://stilltrue.example.com/ | grep -iE 'x-content-type|x-frame|referrer'
```

Then run the seeded demo in a browser and the failover drill from the receipt. The drill should show:

```text
requested  chat
failover   yes · hop 1
reason     Gateway failed over from outage-lab/chat to tokenfactory/Qwen/Qwen3-30B-A3B-Instruct-2507.
drill      forced primary outage
```

### 5. Lock down the gateway

The default Bifrost dashboard has no password. It must not be reachable from the public internet. The production override removes its host port; keep it that way.

If you need the dashboard for operations, put it behind an SSH tunnel or a private network, and set a Bifrost setup token and admin password first:

```bash
export BIFROST_SETUP_TOKEN=$(openssl rand -hex 32)
```

Rotate the development virtual key in `infra/bifrost/config.json` before a public launch.

## Managed platforms

If you prefer a platform such as Render, Railway, or Fly.io:

1. Deploy **Bifrost** as a private service (Docker image `maximhq/bifrost`). Do not give it a public URL.
2. Deploy the **API** (`crates/api/Dockerfile`) with `BIFROST_URL` pointing at the private Bifrost service and `TAVILY_API_KEY` set.
3. Deploy the **web** (`web/Dockerfile`) with:
   - build arg `NEXT_PUBLIC_API_BASE=""`
   - runtime `API_ORIGIN=http://<api-private-host>:8788`
4. Put both behind one public domain. Route `/api/*` to the API and `/` to the web. Most platforms’ HTTP routers support this; if not, add Caddy or nginx in front.

SSE needs a proxy that does not buffer. On nginx:

```nginx
location /api/ {
    proxy_pass http://stilltrue-api:8788;
    proxy_http_version 1.1;
    proxy_set_header Connection "";
    proxy_buffering off;
    proxy_read_timeout 300s;
}
```

## Production checklist

- [ ] `TAVILY_API_KEY` and `TOKENFACTORY_API_KEY` set, never committed
- [ ] Bifrost private, admin password set, dev virtual key rotated
- [ ] `WEB_ORIGIN` matches the public web origin
- [ ] `NEXT_PUBLIC_API_BASE` empty for same-origin deployment
- [ ] `/api/health` returns 200
- [ ] a seeded check returns 44/100 with 3 changed claims
- [ ] the failover drill shows `hop 1` and `forced primary outage`
- [ ] mobile check page is usable
- [ ] security headers present
- [ ] SQLite volume persisted (`api-data`)
- [ ] a backup or snapshot policy exists for the SQLite volume

## Scaling notes

The API uses SQLite and is designed for a single instance. For multiple API replicas, move the check store to Postgres and add a shared rate limiter. Bifrost supports clustering for multi-node gateway deployments. The product code does not depend on SQLite specifics beyond the SQLx queries in `db.rs`.

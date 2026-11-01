.PHONY: help api web bifrost compose test build demo clean

help:
	@echo "StillTrue — common commands"
	@echo ""
	@echo "  make api       Run the Rust API on :8788"
	@echo "  make web       Run the Next.js app on :3100"
	@echo "  make bifrost   Start the Bifrost gateway on :8081"
	@echo "  make compose   Build and run the full stack with Docker"
	@echo "  make test      Run Rust tests and the frontend typecheck"
	@echo "  make build     Release-build the API and production-build the web app"
	@echo "  make demo      Run the seeded demo against the live API"
	@echo ""
	@echo "Environment: copy .env.example to .env and set TAVILY_API_KEY."

api:
	cargo run -p stilltrue-api

web:
	cd web && PORT=3100 pnpm dev

bifrost:
	docker compose up -d bifrost

compose:
	docker compose up --build

test:
	cargo test
	cd web && pnpm typecheck

build:
	cargo build --release -p stilltrue-api
	cd web && pnpm build

demo:
	python3 scripts/demo.py

clean:
	cargo clean
	rm -rf web/.next web/node_modules

#!/usr/bin/env bash
# Start all three services. Ctrl-C stops them.
#   core    http://localhost:8080   (Rust agent)
#   sidecar http://localhost:8090   (Python: SIE extract, Statsig, Airbyte)
#   web     http://localhost:3000   (Next.js dashboard; phone view at /phone/maya)
set -euo pipefail
cd "$(dirname "$0")/.."
trap 'kill 0' EXIT
(cd sidecar && uv run uvicorn app:app --port 8090) &
(cd core && cargo run --release) &
(cd web && pnpm dev) &
wait

#!/usr/bin/env bash
set -euo pipefail

PORT=${PORT:-8080}
mkdir -p data
docker compose up --build -d
echo "Service is up at http://127.0.0.1:${PORT}"



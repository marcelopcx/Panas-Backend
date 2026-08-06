#!/usr/bin/env bash
# Reaplica el esquema panas.sql sobre la BD existente.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

docker compose exec -T db psql -U panas -d panas < db/panas.sql
echo "→ Migración / esquema reaplicado."

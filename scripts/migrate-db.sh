#!/usr/bin/env bash
# Reaplica el esquema panas.sql y migraciones ligeras sobre la BD existente.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

docker compose exec -T db psql -U panas -d panas < db/panas.sql

docker compose exec -T db psql -U panas -d panas <<'SQL'
SET search_path TO panas;
ALTER TABLE usuarios ADD COLUMN IF NOT EXISTS bio VARCHAR(280);
SQL

echo "→ Migración / esquema reaplicado."

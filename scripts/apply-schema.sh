#!/usr/bin/env bash
# Aplica db/panas.sql contra DATABASE_URL (Supabase / Postgres remoto).
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
: "${DATABASE_URL:?Define DATABASE_URL (URI de Supabase con sslmode=require)}"
psql "$DATABASE_URL" -v ON_ERROR_STOP=1 -f "$ROOT/db/panas.sql"
echo "→ Esquema Panas aplicado."

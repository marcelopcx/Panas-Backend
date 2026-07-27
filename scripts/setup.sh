#!/usr/bin/env bash
# Inicializa el proyecto: .env, PostgreSQL y esquema panas.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

if [[ ! -f .env ]]; then
  cp .env.example .env
  echo "→ Creado .env desde .env.example"
fi

# shellcheck disable=SC1091
set -a
source .env
set +a

echo "→ Levantando PostgreSQL (Docker)..."
docker compose up -d --wait

schema_exists() {
  docker compose exec -T db psql -U panas -d panas -tAc \
    "SELECT 1 FROM information_schema.schemata WHERE schema_name = 'panas'" \
    | grep -q 1
}

if schema_exists; then
  echo "→ Esquema panas ya está aplicado."
else
  echo "→ Aplicando esquema desde db/panas.sql..."
  docker compose exec -T db psql -U panas -d panas < db/panas.sql
  echo "→ Esquema aplicado."
fi

echo "→ Cargando datos semilla (db/seed_data.sql)..."
docker compose exec -T db psql -U panas -d panas < db/seed_data.sql || true

echo ""
echo "Listo. Puedes ejecutar: cargo run"
echo "DATABASE_URL=${DATABASE_URL}"

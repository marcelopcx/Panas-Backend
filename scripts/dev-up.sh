#!/usr/bin/env bash
# Arranca Postgres + aplica esquema + deja el API listo (cargo run).
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

echo "→ Levantando PostgreSQL..."
docker compose up -d --wait

echo "→ Aplicando esquema..."
./scripts/migrate-db.sh

echo ""
echo "Listo. En otra terminal:"
echo "  cd \"$ROOT\""
echo "  cargo run"
echo ""
echo "Health:  curl http://127.0.0.1:${PORT:-8080}/health"
echo "API:     http://TU_IP_LOCAL:${PORT:-8080}"
echo "WS:      ws://TU_IP_LOCAL:${PORT:-8080}/ws/chats/{id}?token=JWT"
echo ""
IP="$(ipconfig getifaddr en0 2>/dev/null || ipconfig getifaddr en1 2>/dev/null || true)"
if [[ -n "${IP:-}" ]]; then
  echo "Tu IP local parece ser: $IP"
  echo "En el frontend (.env):"
  echo "  EXPO_PUBLIC_API_URL=http://$IP:${PORT:-8080}"
fi

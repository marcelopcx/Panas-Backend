#!/usr/bin/env bash
# Borra el volumen de Postgres y vuelve a inicializar el entorno.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

echo "→ Deteniendo contenedores y eliminando volumen..."
docker compose down -v

echo "→ Reinicializando..."
"$ROOT/scripts/setup.sh"

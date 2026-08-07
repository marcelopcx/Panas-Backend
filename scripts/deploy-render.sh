#!/usr/bin/env bash
# Crea/actualiza el servicio panas-api en Render con DATABASE_URL de Supabase.
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

if [[ -f "$ROOT/.env.deploy" ]]; then
  # shellcheck disable=SC1091
  set -a
  source "$ROOT/.env.deploy"
  set +a
fi

: "${RENDER_API_KEY:?Falta RENDER_API_KEY}"
: "${DATABASE_URL:?Falta DATABASE_URL}"

API="https://api.render.com/v1"
AUTH="Authorization: Bearer ${RENDER_API_KEY}"

echo "→ Obteniendo owner..."
OWNER_JSON=$(curl -sS -H "$AUTH" "$API/owners?limit=20")
OWNER_ID=$(echo "$OWNER_JSON" | python3 -c 'import sys,json; d=json.load(sys.stdin); print(d[0]["owner"]["id"] if d else "")')
if [[ -z "$OWNER_ID" ]]; then
  echo "No se pudo obtener owner de Render:"; echo "$OWNER_JSON"; exit 1
fi
echo "   owner=$OWNER_ID"

echo "→ Aplicando esquema en Supabase..."
psql "$DATABASE_URL" -v ON_ERROR_STOP=1 -f "$ROOT/db/panas.sql"

echo "→ Buscando servicio panas-api..."
SERVICES=$(curl -sS -H "$AUTH" "$API/services?limit=50")
SERVICE_ID=$(echo "$SERVICES" | python3 -c '
import sys, json
items = json.load(sys.stdin)
for it in items:
    s = it.get("service") or it
    if s.get("name") == "panas-api":
        print(s["id"]); break
')

JWT_SECRET=$(python3 -c 'import secrets; print(secrets.token_hex(32))')

ENV_VARS=$(python3 - <<PY
import json, os
print(json.dumps([
  {"key": "DATABASE_URL", "value": os.environ["DATABASE_URL"]},
  {"key": "JWT_SECRET", "value": os.environ.get("JWT_SECRET", "$JWT_SECRET")},
  {"key": "JWT_EXPIRATION_HOURS", "value": "24"},
  {"key": "HOST", "value": "0.0.0.0"},
  {"key": "CLOUDINARY_CLOUD_NAME", "value": "mpc-uru"},
  {"key": "CLOUDINARY_UPLOAD_PRESET", "value": "n3n6sbhv"},
  {"key": "CLOUDINARY_FOLDER", "value": "panas"},
  {"key": "DEFAULT_AVATAR_URL", "value": "https://res.cloudinary.com/mpc-uru/image/upload/panas/avatars/default.jpg"},
]))
PY
)

if [[ -z "$SERVICE_ID" ]]; then
  echo "→ Creando servicio Docker panas-api..."
  PAYLOAD=$(python3 - <<PY
import json, os
body = {
  "type": "web_service",
  "name": "panas-api",
  "ownerId": os.environ["OWNER_ID"],
  "repo": "https://github.com/marcelopcx/Panas-Backend",
  "autoDeploy": "yes",
  "branch": "main",
  "serviceDetails": {
    "env": "docker",
    "plan": "free",
    "envSpecificDetails": {
      "dockerContext": ".",
      "dockerfilePath": "./Dockerfile"
    },
    "healthCheckPath": "/health",
  },
  "envVars": json.loads('''$ENV_VARS'''),
}
print(json.dumps(body))
PY
)
  # OWNER_ID for python
  export OWNER_ID
  PAYLOAD=$(OWNER_ID="$OWNER_ID" DATABASE_URL="$DATABASE_URL" JWT_SECRET="$JWT_SECRET" python3 - <<'PY'
import json, os
body = {
  "type": "web_service",
  "name": "panas-api",
  "ownerId": os.environ["OWNER_ID"],
  "repo": "https://github.com/marcelopcx/Panas-Backend",
  "autoDeploy": "yes",
  "branch": "main",
  "serviceDetails": {
    "env": "docker",
    "plan": "free",
    "envSpecificDetails": {
      "dockerContext": ".",
      "dockerfilePath": "./Dockerfile"
    },
    "healthCheckPath": "/health",
  },
  "envVars": [
    {"key": "DATABASE_URL", "value": os.environ["DATABASE_URL"]},
    {"key": "JWT_SECRET", "value": os.environ["JWT_SECRET"]},
    {"key": "JWT_EXPIRATION_HOURS", "value": "24"},
    {"key": "HOST", "value": "0.0.0.0"},
    {"key": "CLOUDINARY_CLOUD_NAME", "value": "mpc-uru"},
    {"key": "CLOUDINARY_UPLOAD_PRESET", "value": "n3n6sbhv"},
    {"key": "CLOUDINARY_FOLDER", "value": "panas"},
    {"key": "DEFAULT_AVATAR_URL", "value": "https://res.cloudinary.com/mpc-uru/image/upload/panas/avatars/default.jpg"},
  ],
}
print(json.dumps(body))
PY
)
  RESP=$(curl -sS -X POST -H "$AUTH" -H "Content-Type: application/json" \
    -H "Accept: application/json" \
    "$API/services" -d "$PAYLOAD")
  echo "$RESP" | python3 -m json.tool | head -40
  SERVICE_ID=$(echo "$RESP" | python3 -c 'import sys,json; d=json.load(sys.stdin); print((d.get("service") or d).get("id",""))')
else
  echo "→ Servicio existente: $SERVICE_ID — actualizando env vars..."
  curl -sS -X PUT -H "$AUTH" -H "Content-Type: application/json" \
    "$API/services/$SERVICE_ID/env-vars" \
    -d "$(DATABASE_URL="$DATABASE_URL" JWT_SECRET="$JWT_SECRET" python3 - <<'PY'
import json, os
print(json.dumps([
    {"key": "DATABASE_URL", "value": os.environ["DATABASE_URL"]},
    {"key": "JWT_SECRET", "value": os.environ["JWT_SECRET"]},
    {"key": "JWT_EXPIRATION_HOURS", "value": "24"},
    {"key": "HOST", "value": "0.0.0.0"},
    {"key": "CLOUDINARY_CLOUD_NAME", "value": "mpc-uru"},
    {"key": "CLOUDINARY_UPLOAD_PRESET", "value": "n3n6sbhv"},
    {"key": "CLOUDINARY_FOLDER", "value": "panas"},
    {"key": "DEFAULT_AVATAR_URL", "value": "https://res.cloudinary.com/mpc-uru/image/upload/panas/avatars/default.jpg"},
]))
PY
)" | python3 -m json.tool | head -20
  echo "→ Disparando deploy..."
  curl -sS -X POST -H "$AUTH" "$API/services/$SERVICE_ID/deploys" -H "Content-Type: application/json" -d '{}' | python3 -m json.tool | head -20
fi

echo "→ SERVICE_ID=$SERVICE_ID"
echo "Espera el build en https://dashboard.render.com"
echo "Health: https://panas-api.onrender.com/health"

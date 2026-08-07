# Panas (Backend)

¡Bienvenido a **Panas**! API REST + WebSocket de la app de descubrimiento de amistades y chat. El desarrollo forma parte de una asignación práctica para el curso de **Desarrollo de Aplicaciones Móviles** en la **Universidad Rafael Urdaneta (URU)**.

Arquitectura desacoplada:

* **Rust** con **[Actix Web](https://actix.rs/)** como servidor HTTP.
* Persistencia en **PostgreSQL** mediante **[SQLx](https://github.com/launchbadge/sqlx)**.
* Autenticación **JWT** (Bearer) y contraseñas con **bcrypt**.
* Imágenes con **Cloudinary** (avatares y fotos de chat).
* Chat en vivo con **actix-ws**.
* Push con **Expo Push Notifications**.

Cliente móvil: **[Panas — Frontend](https://github.com/marcelopcx/Panas-Frontend)** (React Native + Expo).

Referencia completa de endpoints: **[API.md](./API.md)**.

---

## Guía de inicialización del proyecto

### Prerrequisitos

1. **[Rust](https://www.rust-lang.org/tools/install)** (toolchain *stable*).
2. **[Docker](https://www.docker.com/)** y **Docker Compose** (PostgreSQL local).
3. *(Opcional)* `psql` para aplicar el esquema a una BD remota.

### Pasos (local)

1. **Navegá al directorio del backend:**

   ```bash
   cd backend
   ```

   *(Si clonaste solo este repo: `cd Panas-Backend`.)*

2. **Dale permisos a los scripts** *(solo la primera vez)*:

   ```bash
   chmod +x scripts/*.sh
   ```

3. **Inicializá el entorno** (`.env`, Docker en puerto **5433**, esquema):

   ```bash
   make dev-up
   ```

   Este comando:
   * Crea `.env` desde `.env.example` si no existe.
   * Levanta PostgreSQL (`panas_db`, puerto host **5433** para no chocar con otros proyectos del curso).
   * Aplica `db/panas.sql` (esquema `panas`).
   * Muestra tu IP local sugerida para el frontend.

4. **Revisá `.env`** si hace falta:

   ```env
   DATABASE_URL=postgres://panas:secret123@127.0.0.1:5433/panas
   JWT_SECRET=un_secreto_largo_minimo_32_caracteres_cambiar_en_produccion
   JWT_EXPIRATION_HOURS=24
   HOST=0.0.0.0
   PORT=8080
   CLOUDINARY_CLOUD_NAME=mpc-uru
   CLOUDINARY_UPLOAD_PRESET=n3n6sbhv
   CLOUDINARY_FOLDER=panas
   DEFAULT_AVATAR_URL=https://res.cloudinary.com/mpc-uru/image/upload/panas/avatars/default.jpg
   ```

   *No subas `.env` a git.*

5. **Iniciá la API:**

   ```bash
   cargo run
   ```

   Escucha en **`0.0.0.0:8080`** (accesible por IP local desde el teléfono).

6. **Health check:**

   ```bash
   curl http://127.0.0.1:8080/health
   ```

### Alternativa con Make setup

```bash
make setup
cargo run
```

---

## Despliegue (producción)

Servicio en Render (Docker):

* **URL:** https://panas-api.onrender.com  
* **Health:** `GET /health`  
* Repo: este mismo (`Dockerfile` + `render.yaml`)

Variables importantes en el servicio: `DATABASE_URL` (Postgres con SSL), `JWT_SECRET`, Cloudinary, `HOST=0.0.0.0`. Render inyecta `PORT`.

Aplicar esquema una vez sobre la BD remota:

```bash
psql "$DATABASE_URL" -f db/panas.sql
# o
./scripts/apply-schema.sh
```

*Nota:* en el plan free de Render el servicio puede dormirse; la primera request tarda ~30–60 s.

---

## Arquitectura de carpetas

* `src/main.rs` — Entrada: pool, config, CORS, **HttpServer**.
* `src/lib.rs` — Módulos del crate.
* `src/config/` — Variables de entorno (`AppConfig`, Cloudinary, avatar por defecto).
* `src/db/` — Pool PostgreSQL (`search_path` → `panas`).
* `src/auth/` — Extractores JWT (`AuthenticatedUser`).
* `src/handlers/` — Controladores HTTP (auth, amistad, chat, notificaciones, imágenes, WS).
* `src/services/` — Lógica de negocio + cliente Expo Push + Cloudinary.
* `src/models/` — Request/response y filas SQLx.
* `src/routes/` — Registro de rutas Actix.
* `src/error/` — `ApiError` unificado.
* `db/panas.sql` — Esquema completo.
* `scripts/` — `dev-up.sh`, `migrate-db.sh`, `apply-schema.sh`, deploy helpers.
* `api/` — Colección [Bruno](https://www.usebruno.com/) (si está presente).
* `Dockerfile` / `render.yaml` — Deploy en Render.
* `docker-compose.yml` — Postgres 16 local (puerto **5433**).

---

## Mapa UI → API

| Pantalla front | Endpoints |
|----------------|-----------|
| Login | `POST /auth/login` |
| Registro | `POST /auth/register` (+ opcional `POST /auth/me/avatar`) |
| Forgot password | `POST /auth/forgot-password` |
| Perfil | `GET/PATCH/DELETE /auth/me`, `POST /auth/me/avatar` |
| Push | `POST/DELETE /auth/me/push-token` |
| Meet | `GET /descubrir` · `POST /descubrir/pasar` · `POST /amistades` |
| Bandeja | `GET /amistades/pendientes` · aceptar / rechazar |
| Chats | `GET /chats` |
| Mensajes | `GET/POST /chats/{id}/mensajes` · imagen · leer |
| Campana | `GET /notificaciones` · leer · eliminar |
| Chat en vivo | `WS /ws/chats/{id}?token=JWT` |

Auth: header `Authorization: Bearer <token>`.

Detalle de bodies y respuestas: **[API.md](./API.md)**.

---

## Conexión con el Frontend (Expo)

Repositorio: **[Panas-Frontend](https://github.com/marcelopcx/Panas-Frontend)**

```env
# Local
EXPO_PUBLIC_API_URL=http://TU_IP_LOCAL:8080

# Producción
EXPO_PUBLIC_API_URL=https://panas-api.onrender.com
```

*El backend escucha en el puerto **8080** por defecto (`PORT` en `.env`). Debe coincidir con el frontend.*

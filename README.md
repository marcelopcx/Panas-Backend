# Panas (Backend)

API REST del proyecto **Panas**, desarrollado para el curso de **Desarrollo de Aplicaciones Móviles** en la **Universidad Rafael Urdaneta (URU)**.

Este repositorio contiene la estructura base del backend, alineada con **Nook's Cookbook** y **Game World**, lista para agregar el modelo de datos y la lógica de negocio de Panas.

Stack:

- **Rust** con **[Actix Web](https://actix.rs/)**
- **PostgreSQL** con **[SQLx](https://github.com/launchbadge/sqlx)**
- Autenticación **JWT** (Bearer) y contraseñas con **bcrypt** (infraestructura preparada)

El cliente móvil está en el repositorio del frontend: **[Panas — Frontend](https://github.com/marcelopcx/Panas-Frontend)** (React Native + Expo).

---

## Guía de inicialización

### Prerrequisitos

1. **[Rust](https://www.rust-lang.org/tools/install)** (toolchain *stable*).
2. **[Docker](https://www.docker.com/)** y **Docker Compose**.

### Pasos

1. **Entrá al directorio del backend:**
   ```bash
   cd backend
   ```

2. **Dale permisos de ejecución a los scripts** *(solo la primera vez)*:
   ```bash
   chmod +x scripts/*.sh
   ```

3. **Inicializá el entorno** (`.env`, Docker, esquema):
   ```bash
   make setup
   ```
   Este comando realiza automáticamente:
   * Crea `.env` desde `.env.example` si no existe.
   * Levanta PostgreSQL con Docker Compose.
   * Aplica el esquema `panas`.

4. **Revisá las variables de entorno** en `.env` si hace falta:
   ```env
   DATABASE_URL=postgres://panas:secret123@127.0.0.1:5432/panas
   JWT_SECRET=un_secreto_largo_minimo_32_caracteres_cambiar_en_produccion
   JWT_EXPIRATION_HOURS=24
   HOST=0.0.0.0
   PORT=8080
   ```

5. **Iniciá el servidor:**
   ```bash
   cargo run
   ```

6. **Comprobá el health check:**
   ```bash
   curl http://127.0.0.1:8080/health
   ```

### Alternativa sin Make

```bash
./scripts/setup.sh
cargo run
```

---

## Arquitectura de carpetas

- `src/main.rs` — Punto de entrada: pool de conexiones, configuración y arranque de **HttpServer**.
- `src/lib.rs` — Módulos públicos del crate.
- `src/config/` — Carga de variables de entorno (`AppConfig`).
- `src/db/` — Creación del pool de PostgreSQL (`search_path` → `panas`).
- `src/auth/` — Extractores `AuthenticatedUser` y `OptionalAuthenticatedUser` (JWT).
- `src/handlers/` — Controladores HTTP (por ahora solo `health`).
- `src/services/` — Lógica de negocio (utilidades JWT en `auth`).
- `src/models/` — Structs de request/response y filas de BD *(listo para completar)*.
- `src/routes/` — Registro de servicios Actix (`configure`).
- `src/error/` — Errores de API unificados (`ApiError`).
- `db/` — Esquema SQL (`panas.sql`) y datos semilla (`seed_data.sql`).
- `scripts/` — `setup.sh`, `migrate-db.sh` y `reset-db.sh`.
- `api/` — Colección [Bruno](https://www.usebruno.com/) para probar endpoints.
- `docker-compose.yml` — PostgreSQL 16 en contenedor.
- `build.rs` — Carga `.env` al compilar; activa **SQLx offline** si existe `.sqlx/`.

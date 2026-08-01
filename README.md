# Panas (Backend)

API REST + WebSocket del proyecto **Panas** (URU — Desarrollo de Aplicaciones Móviles).

Stack: **Rust** · **Actix Web** · **PostgreSQL / SQLx** · **JWT** · **Cloudinary** · **actix-ws**

Frontend: [Panas — Frontend](https://github.com/marcelopcx/Panas-Frontend)

---

## Arranque

```bash
chmod +x scripts/*.sh
make setup          # .env + Docker Postgres + esquema
cargo run
curl http://127.0.0.1:8080/health
```

Variables en `.env` (ver `.env.example`): `DATABASE_URL`, `JWT_*`, `CLOUDINARY_*`, `DEFAULT_AVATAR_URL`.

---

## API

### Auth / perfil (CRUD)

| Método | Ruta | Auth | Descripción |
|--------|------|------|-------------|
| POST | `/auth/register` | — | Registro (`username`, `email`, `password`, opc. `nombre`, `apellido`, `bio`, `url_avatar`) |
| POST | `/auth/login` | — | Login → `{ token, user }` |
| GET | `/auth/me` | Bearer | Perfil completo |
| PATCH | `/auth/me` | Bearer | Actualizar perfil |
| DELETE | `/auth/me` | Bearer | Eliminar cuenta |
| POST | `/auth/me/avatar` | Bearer | Multipart `file` → sube a Cloudinary y guarda `url_avatar` |
| GET | `/usuarios` | Bearer | Buscar usuarios (`?q=&page=&limit=`) |
| GET | `/usuarios/{id}` | — | Perfil público |

### Amistades (swipe)

| Método | Ruta | Descripción |
|--------|------|-------------|
| POST | `/amistades` | Enviar solicitud `{ "id_usuario": N }` |
| GET | `/amistades/pendientes` | Solicitudes recibidas (para swipe) |
| POST | `/amistades/{id}/aceptar` | **Swipe derecha** → acepta y crea chat |
| POST | `/amistades/{id}/rechazar` | **Swipe izquierda** → rechaza |
| POST | `/amistades/{id}/decidir` | `{ "accion": "aceptar" \| "rechazar" }` |
| GET | `/amistades` | Amigos aceptados (incluye `id_chat`) |

### Chats / mensajes (persistencia)

| Método | Ruta | Descripción |
|--------|------|-------------|
| GET | `/chats` | Lista de chats |
| POST | `/chats` | Abrir chat con amigo `{ "id_usuario": N }` |
| GET | `/chats/{id}/mensajes` | Historial (`?page=&limit=`) |
| POST | `/chats/{id}/mensajes` | Enviar texto/imagen `{ "contenido"?, "url_imagen"? }` |
| POST | `/chats/{id}/imagen` | Multipart `file` → sube imagen, persiste mensaje y emite por WS |

### WebSocket (mensajes en vivo)

```
ws://HOST:8080/ws/chats/{id_chat}?token=<JWT>
```

También acepta header `Authorization: Bearer <JWT>`.

**Cliente → servidor**
```json
{ "type": "enviar", "contenido": "hola", "url_imagen": null }
{ "type": "ping" }
```

**Servidor → cliente**
```json
{ "type": "mensaje", "mensaje": { "id_mensaje": 1, "id_chat": 1, "id_remitente": 2, "contenido": "hola", "url_imagen": null, "tipo": "texto", "fecha_envio": "..." } }
{ "type": "pong" }
{ "type": "error", "error": "..." }
```

Los mensajes enviados por REST o WS se **persisten** en PostgreSQL y se **retransmiten** a los participantes conectados al mismo chat.

---

## Arquitectura

- `src/handlers/` — HTTP / WS
- `src/services/` — negocio (`auth`, `amistad`, `chat`, `cloudinary`) + `ChatHub` (broadcast WS)
- `src/models/` — DTOs
- `src/auth/` — extractor JWT
- `db/panas.sql` — esquema

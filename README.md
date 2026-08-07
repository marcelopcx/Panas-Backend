# Panas (Backend)

API REST + WebSocket alineada al frontend Expo (auth, Meet, Bandeja, Chats, Perfil, notificaciones).

Stack: **Rust** · **Actix Web** · **PostgreSQL / SQLx** · **JWT** · **Cloudinary** · **actix-ws**

Frontend: [Panas — Frontend](https://github.com/marcelopcx/Panas-Frontend)

---

## Arranque

### Local (recomendado para probar)

1. Abrí **Docker Desktop** y esperá a que esté listo.
2. En el backend:

```bash
cd backend
make dev-up      # Postgres + esquema + te muestra la IP
cargo run        # API en :8080
```

3. En el frontend, creá `.env`:

```env
EXPO_PUBLIC_API_URL=http://TU_IP_LOCAL:8080
```

(Usá la IP que imprimió `make dev-up`, no `localhost`, para que el teléfono alcance la API.)

4. `npx expo start -c` y escaneá el QR.

Comprobá:

```bash
curl http://127.0.0.1:8080/health
```

### Render (nube, gratis)

1. Subí `Dockerfile` + `render.yaml` al repo (ya están en el proyecto).
2. En [render.com](https://render.com): **New → Blueprint** → conectá `Panas-Backend`.
3. Creá también un **PostgreSQL** en Render y pegá su `DATABASE_URL` en el servicio web.
4. Aplicá el esquema una vez (Shell de Render o `psql`):

```bash
psql "$DATABASE_URL" -f db/panas.sql
```

5. En el frontend:

```env
EXPO_PUBLIC_API_URL=https://panas-api.onrender.com
```

(ajustá al hostname real que te dé Render).

Variables ya definidas en el Blueprint: JWT, Cloudinary (`mpc-uru` / folder `panas`), `HOST=0.0.0.0`.

---

## Mapa UI → API

| Pantalla front | Endpoints |
|----------------|-----------|
| Login | `POST /auth/login` `{ email, password }` |
| Registro | `POST /auth/register` `{ email, password, full_name, url_avatar? }` |
| Forgot password | `POST /auth/forgot-password` `{ email }` |
| Perfil | `GET/PATCH/DELETE /auth/me`, `POST /auth/me/avatar` |
| Push | `POST/DELETE /auth/me/push-token` (Expo Push) |
| Privacidad | `PATCH /auth/me` `{ privacidad: "publico"\|"privado"\|"solo_amigos" }` |
| Meet (deck) | `GET /descubrir` · swipe izq `POST /descubrir/pasar` · swipe der `POST /amistades` |
| Bandeja | `GET /amistades/pendientes` · `POST /amistades/{id}/aceptar\|rechazar` |
| Chats | `GET /chats` (incluye `name`, `last_message`, `unread`, `updated_at`) |
| Mensajes | `GET/POST /chats/{id}/mensajes` · `POST /chats/{id}/imagen` · `POST /chats/{id}/leer` |
| Campana | `GET /notificaciones` · `PATCH .../leer` · `POST .../leer-todas` · `DELETE` |
| Chat en vivo | `ws://HOST:8080/ws/chats/{id}?token=JWT` |

Auth: header `Authorization: Bearer <token>` (WS también acepta `?token=`).

---

## Auth

```http
POST /auth/register
{ "email": "a@b.com", "password": "secreto12", "full_name": "Jhon Doe", "url_avatar": null }

POST /auth/login
{ "email": "a@b.com", "password": "secreto12" }
→ { "token": "...", "user": { "id_usuario", "username", "email", "url_avatar" } }

POST /auth/forgot-password
{ "email": "a@b.com" }

GET    /auth/me
PATCH  /auth/me   { "full_name"?, "privacidad"?, "bio"?, "url_avatar"?, ... }
DELETE /auth/me
POST   /auth/me/avatar   multipart field `file`
POST   /auth/me/push-token   { "token": "ExponentPushToken[...]" }
DELETE /auth/me/push-token
```

Password mínimo **8** caracteres (como valida el front).

---

## Descubrir / Meet

```http
GET  /descubrir?limit=20
→ [{ "id_usuario", "name", "url_avatar", "bio", "username" }]

POST /descubrir/pasar
{ "id_usuario": 5 }          # swipe izquierda

POST /amistades
{ "id_usuario": 5 }          # swipe derecha → solicitud
```

Solo aparecen perfiles `privacidad = publico`, sin amistad/solicitud pendiente y no pasados antes.

---

## Amistades (Bandeja)

```http
GET  /amistades/pendientes
→ [{ "id_solicitud", "name", "message", "url_avatar", ... }]

POST /amistades/{id}/aceptar     # swipe derecha en bandeja → crea chat
POST /amistades/{id}/rechazar    # swipe izquierda
POST /amistades/{id}/decidir     { "accion": "aceptar" | "rechazar" }
GET  /amistades                  # amigos + id_chat
```

---

## Chats / mensajes

```http
GET  /chats
→ [{ "id_chat", "name", "url_avatar", "last_message", "updated_at", "unread", "otro_usuario" }]

POST /chats                  { "id_usuario": N }
GET  /chats/{id}/mensajes    ?page=&limit=     # también marca leído
POST /chats/{id}/mensajes    { "text": "hola" } o { "contenido": "hola" } o { "url_imagen": "..." }
POST /chats/{id}/imagen      multipart `file`
POST /chats/{id}/leer
```

WebSocket:

```
ws://HOST:8080/ws/chats/{id}?token=<JWT>
→ { "type": "enviar", "text": "hola" }
← { "type": "mensaje", "mensaje": { ... } }
```

---

## Notificaciones

```http
GET    /notificaciones?solo_no_leidas=true
→ { "items": [...], "unread": 3 }

PATCH  /notificaciones/{id}/leer
POST   /notificaciones/leer-todas
DELETE /notificaciones/{id}
```

Se crean automáticamente al recibir solicitud, al aceptar amistad y al recibir mensaje.

---

## Privacidad

| Valor API | UI front |
|-----------|----------|
| `publico` | Público (aparece en Meet) |
| `privado` | Privado (solo el dueño ve el perfil) |
| `solo_amigos` | Solo amigos |

# Panas — Referencia de API

Base URL local: `http://TU_IP_LOCAL:8080`  
Base URL producción: `https://panas-api.onrender.com`

Autenticación: header `Authorization: Bearer <JWT>` en rutas protegidas.  
WebSocket también acepta `?token=<JWT>`.

Errores típicos: JSON `{ "error": "mensaje" }` con códigos `400` / `401` / `403` / `404` / `409` / `500`.

---

## Infraestructura

### `GET /health`

Sin auth. Respuesta de texto confirmando que el servicio está en línea.

---

## Autenticación y perfil

| Método | Ruta | Auth | Descripción |
|--------|------|------|-------------|
| `POST` | `/auth/register` | No | Alta de usuario |
| `POST` | `/auth/login` | No | Login → JWT |
| `POST` | `/auth/forgot-password` | No | Stub de recuperación |
| `GET` | `/auth/me` | Sí | Perfil propio |
| `PATCH` | `/auth/me` | Sí | Actualizar perfil |
| `DELETE` | `/auth/me` | Sí | Eliminar cuenta |
| `POST` | `/auth/me/avatar` | Sí | Subir avatar (multipart `file`) |
| `POST` | `/auth/me/push-token` | Sí | Registrar Expo Push token |
| `DELETE` | `/auth/me/push-token` | Sí | Quitar push token |

### Registro

```json
POST /auth/register
{
  "email": "a@b.com",
  "password": "secreto12",
  "full_name": "Jhon Doe",
  "url_avatar": null
}
```

Password mínimo **8** caracteres. Si no hay avatar, se usa `DEFAULT_AVATAR_URL`.

### Login

```json
POST /auth/login
{ "email": "a@b.com", "password": "secreto12" }
```

Respuesta:

```json
{
  "token": "...",
  "user": {
    "id_usuario": 1,
    "username": "...",
    "email": "a@b.com",
    "url_avatar": "..."
  }
}
```

### Perfil (`GET /auth/me`)

Incluye `name`, `privacidad` (`publico` | `privado` | `solo_amigos`), `bio`, etc.

### Privacidad

| Valor API | UI |
|-----------|-----|
| `publico` | Público (aparece en Meet) |
| `privado` | Privado |
| `solo_amigos` | Solo amigos |

---

## Descubrir / Meet

| Método | Ruta | Auth | Descripción |
|--------|------|------|-------------|
| `GET` | `/descubrir?limit=20` | Sí | Candidatos públicos |
| `POST` | `/descubrir/pasar` | Sí | Swipe izquierda |
| `POST` | `/amistades` | Sí | Swipe derecha → solicitud |

```json
POST /descubrir/pasar
{ "id_usuario": 5 }

POST /amistades
{ "id_usuario": 5 }
```

Solo perfiles `privacidad = publico`, sin amistad ni solicitud pendiente, y no pasados antes.

---

## Amistades (bandeja)

| Método | Ruta | Auth | Descripción |
|--------|------|------|-------------|
| `GET` | `/amistades/pendientes` | Sí | Solicitudes recibidas |
| `POST` | `/amistades/{id}/aceptar` | Sí | Aceptar (crea chat) |
| `POST` | `/amistades/{id}/rechazar` | Sí | Rechazar |
| `POST` | `/amistades/{id}/decidir` | Sí | `{ "accion": "aceptar" \| "rechazar" }` |
| `GET` | `/amistades` | Sí | Amigos + `id_chat` |

---

## Chats y mensajes

| Método | Ruta | Auth | Descripción |
|--------|------|------|-------------|
| `GET` | `/chats` | Sí | Lista (`name`, `last_message`, `unread`, `updated_at`) |
| `POST` | `/chats` | Sí | Abrir chat `{ "id_usuario": N }` |
| `GET` | `/chats/{id}/mensajes` | Sí | Historial (`page`, `limit`); marca leído |
| `POST` | `/chats/{id}/mensajes` | Sí | Texto (`text` / `contenido`) |
| `POST` | `/chats/{id}/imagen` | Sí | Multipart `file` |
| `POST` | `/chats/{id}/leer` | Sí | Marcar leído |

### WebSocket

```
ws://HOST:8080/ws/chats/{id}?token=<JWT>
# o wss://panas-api.onrender.com/ws/chats/{id}?token=<JWT>
```

Cliente → servidor: `{ "type": "enviar", "text": "hola" }`  
Servidor → cliente: `{ "type": "mensaje", "mensaje": { ... } }`

---

## Notificaciones (campana)

| Método | Ruta | Auth | Descripción |
|--------|------|------|-------------|
| `GET` | `/notificaciones` | Sí | Lista (`solo_no_leidas`, `limit`) |
| `PATCH` | `/notificaciones/{id}/leer` | Sí | Marcar leída |
| `POST` | `/notificaciones/leer-todas` | Sí | Marcar todas |
| `DELETE` | `/notificaciones/{id}` | Sí | Eliminar |

Se crean al recibir solicitud, al aceptar amistad y al recibir mensaje. Si el destinatario tiene Expo Push token, también se envía push.

---

## Usuarios

| Método | Ruta | Auth | Descripción |
|--------|------|------|-------------|
| `GET` | `/usuarios` | Sí | Búsqueda / listado |
| `GET` | `/usuarios/{id}` | Sí | Perfil público |

---

## Imágenes

| Método | Ruta | Auth | Descripción |
|--------|------|------|-------------|
| `POST` | `/auth/me/avatar` | Sí | Avatar → Cloudinary |
| `POST` | `/chats/{id}/imagen` | Sí | Imagen de chat |
| `POST` | `/imagenes` | Sí | Subida genérica (si está expuesta) |

Campo multipart: `file`.

---

## Códigos HTTP comunes

| Código | Uso |
|--------|-----|
| 200 / 201 | OK / creado |
| 204 | Sin contenido (p. ej. delete) |
| 400 | Solicitud inválida |
| 401 | No autenticado / login incorrecto |
| 403 | Prohibido |
| 404 | No encontrado |
| 409 | Conflicto (usuario duplicado, etc.) |
| 500 | Error de servidor |

---

## Resumen rápido

| Módulo | Prefijo |
|--------|---------|
| Health | `/health` |
| Auth / perfil / push | `/auth/*` |
| Meet | `/descubrir/*` |
| Amistades | `/amistades/*` |
| Chats | `/chats/*` |
| Notificaciones | `/notificaciones/*` |
| WebSocket | `/ws/chats/{id}` |

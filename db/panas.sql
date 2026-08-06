-- =============================================================================
-- Panas — Esquema de base de datos (PostgreSQL)
-- Alineado con el frontend: auth, descubrir/swipe, amistades, chats, notifs.
-- =============================================================================

CREATE SCHEMA IF NOT EXISTS panas;
SET search_path TO panas;

-- -----------------------------------------------------------------------------
-- 1. Usuarios
-- -----------------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS usuarios (
    id_usuario SERIAL PRIMARY KEY,
    username VARCHAR(50) NOT NULL UNIQUE,
    email VARCHAR(100) NOT NULL UNIQUE,
    password VARCHAR(255) NOT NULL,
    nombre VARCHAR(80),
    apellido VARCHAR(50),
    bio VARCHAR(280),
    url_avatar VARCHAR(512),
    privacidad VARCHAR(20) NOT NULL DEFAULT 'publico'
        CHECK (privacidad IN ('publico', 'privado', 'solo_amigos')),
    fecha_registro TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

ALTER TABLE usuarios ADD COLUMN IF NOT EXISTS bio VARCHAR(280);
ALTER TABLE usuarios ADD COLUMN IF NOT EXISTS privacidad VARCHAR(20);
UPDATE usuarios SET privacidad = 'publico' WHERE privacidad IS NULL;
-- Asegurar check (idempotente vía DO)
DO $$
BEGIN
  ALTER TABLE usuarios
    ADD CONSTRAINT usuarios_privacidad_check
    CHECK (privacidad IN ('publico', 'privado', 'solo_amigos'));
EXCEPTION
  WHEN duplicate_object THEN NULL;
END $$;

-- -----------------------------------------------------------------------------
-- 2. Solicitudes de amistad (bandeja: aceptar / rechazar)
-- -----------------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS solicitudes_amistad (
    id_solicitud SERIAL PRIMARY KEY,
    id_remitente INTEGER NOT NULL REFERENCES usuarios (id_usuario) ON DELETE CASCADE,
    id_destinatario INTEGER NOT NULL REFERENCES usuarios (id_usuario) ON DELETE CASCADE,
    estado VARCHAR(20) NOT NULL DEFAULT 'pendiente'
        CHECK (estado IN ('pendiente', 'aceptada', 'rechazada')),
    fecha_creacion TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    fecha_respuesta TIMESTAMPTZ,
    CONSTRAINT solicitudes_distintos CHECK (id_remitente <> id_destinatario),
    CONSTRAINT solicitudes_unicas UNIQUE (id_remitente, id_destinatario)
);

CREATE INDEX IF NOT EXISTS idx_solicitudes_destinatario_estado
    ON solicitudes_amistad (id_destinatario, estado);

-- -----------------------------------------------------------------------------
-- 3. Pases de descubrir (swipe izquierda en Meet)
-- -----------------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS pases_descubrir (
    id_pase SERIAL PRIMARY KEY,
    id_usuario INTEGER NOT NULL REFERENCES usuarios (id_usuario) ON DELETE CASCADE,
    id_pasado INTEGER NOT NULL REFERENCES usuarios (id_usuario) ON DELETE CASCADE,
    fecha_pase TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT pases_distintos CHECK (id_usuario <> id_pasado),
    CONSTRAINT pases_unicos UNIQUE (id_usuario, id_pasado)
);

CREATE INDEX IF NOT EXISTS idx_pases_usuario ON pases_descubrir (id_usuario);

-- -----------------------------------------------------------------------------
-- 4. Chats 1:1 + lecturas (unread)
-- -----------------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS chats (
    id_chat SERIAL PRIMARY KEY,
    id_usuario_menor INTEGER NOT NULL REFERENCES usuarios (id_usuario) ON DELETE CASCADE,
    id_usuario_mayor INTEGER NOT NULL REFERENCES usuarios (id_usuario) ON DELETE CASCADE,
    fecha_creacion TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    ultima_lectura_menor TIMESTAMPTZ,
    ultima_lectura_mayor TIMESTAMPTZ,
    CONSTRAINT chats_distintos CHECK (id_usuario_menor < id_usuario_mayor),
    CONSTRAINT chats_unicos UNIQUE (id_usuario_menor, id_usuario_mayor)
);

ALTER TABLE chats ADD COLUMN IF NOT EXISTS ultima_lectura_menor TIMESTAMPTZ;
ALTER TABLE chats ADD COLUMN IF NOT EXISTS ultima_lectura_mayor TIMESTAMPTZ;

-- -----------------------------------------------------------------------------
-- 5. Mensajes
-- -----------------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS mensajes (
    id_mensaje SERIAL PRIMARY KEY,
    id_chat INTEGER NOT NULL REFERENCES chats (id_chat) ON DELETE CASCADE,
    id_remitente INTEGER NOT NULL REFERENCES usuarios (id_usuario) ON DELETE CASCADE,
    contenido TEXT,
    url_imagen VARCHAR(512),
    tipo VARCHAR(20) NOT NULL DEFAULT 'texto'
        CHECK (tipo IN ('texto', 'imagen')),
    fecha_envio TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT mensaje_tiene_contenido CHECK (
        (contenido IS NOT NULL AND btrim(contenido) <> '')
        OR (url_imagen IS NOT NULL AND btrim(url_imagen) <> '')
    )
);

CREATE INDEX IF NOT EXISTS idx_mensajes_chat_fecha
    ON mensajes (id_chat, fecha_envio DESC);

-- -----------------------------------------------------------------------------
-- 6. Notificaciones (campana del Header)
-- -----------------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS notificaciones (
    id_notificacion SERIAL PRIMARY KEY,
    id_usuario INTEGER NOT NULL REFERENCES usuarios (id_usuario) ON DELETE CASCADE,
    tipo VARCHAR(40) NOT NULL
        CHECK (tipo IN (
            'mensaje',
            'solicitud_amistad',
            'solicitud_aceptada',
            'sistema'
        )),
    mensaje TEXT NOT NULL,
    leida BOOLEAN NOT NULL DEFAULT FALSE,
    id_referencia INTEGER,
    fecha_creacion TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_notificaciones_usuario_fecha
    ON notificaciones (id_usuario, fecha_creacion DESC);

CREATE INDEX IF NOT EXISTS idx_notificaciones_usuario_leida
    ON notificaciones (id_usuario, leida);

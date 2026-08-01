-- =============================================================================
-- Panas — Esquema de base de datos (PostgreSQL)
-- =============================================================================
-- Dominio: usuarios, amistades (aceptar/rechazar), chats y mensajes en vivo.
-- =============================================================================

CREATE SCHEMA IF NOT EXISTS panas;
SET search_path TO panas;

-- -----------------------------------------------------------------------------
-- 1. Usuarios (credenciales, perfil y avatar)
-- -----------------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS usuarios (
    id_usuario SERIAL PRIMARY KEY,
    username VARCHAR(50) NOT NULL UNIQUE,
    email VARCHAR(100) NOT NULL UNIQUE,
    password VARCHAR(255) NOT NULL,
    nombre VARCHAR(50),
    apellido VARCHAR(50),
    bio VARCHAR(280),
    url_avatar VARCHAR(512),
    fecha_registro TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- -----------------------------------------------------------------------------
-- 2. Solicitudes de amistad (swipe: derecha = aceptar, izquierda = rechazar)
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

CREATE INDEX IF NOT EXISTS idx_solicitudes_remitente
    ON solicitudes_amistad (id_remitente);

-- -----------------------------------------------------------------------------
-- 3. Chats (1:1 entre amigos; ids ordenados para unicidad)
-- -----------------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS chats (
    id_chat SERIAL PRIMARY KEY,
    id_usuario_menor INTEGER NOT NULL REFERENCES usuarios (id_usuario) ON DELETE CASCADE,
    id_usuario_mayor INTEGER NOT NULL REFERENCES usuarios (id_usuario) ON DELETE CASCADE,
    fecha_creacion TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT chats_distintos CHECK (id_usuario_menor < id_usuario_mayor),
    CONSTRAINT chats_unicos UNIQUE (id_usuario_menor, id_usuario_mayor)
);

CREATE INDEX IF NOT EXISTS idx_chats_usuario_menor ON chats (id_usuario_menor);
CREATE INDEX IF NOT EXISTS idx_chats_usuario_mayor ON chats (id_usuario_mayor);

-- -----------------------------------------------------------------------------
-- 4. Mensajes (texto y/o imagen; persistidos para historial)
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

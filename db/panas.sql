-- =============================================================================
-- Panas — Esquema de base de datos (PostgreSQL)
-- =============================================================================
-- Estructura base lista para ampliar con el modelo de dominio del proyecto.
-- =============================================================================

CREATE SCHEMA IF NOT EXISTS panas;
SET search_path TO panas;

-- -----------------------------------------------------------------------------
-- 1. Usuarios (credenciales y perfil)
-- -----------------------------------------------------------------------------
CREATE TABLE usuarios (
    id_usuario SERIAL PRIMARY KEY,
    username VARCHAR(50) NOT NULL UNIQUE,
    email VARCHAR(100) NOT NULL UNIQUE,
    password VARCHAR(255) NOT NULL,
    nombre VARCHAR(50),
    apellido VARCHAR(50),
    url_avatar VARCHAR(255),
    fecha_registro TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

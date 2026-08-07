# Imagen para desplegar en Render / Docker.
FROM rust:1.88-bookworm AS builder
WORKDIR /app
COPY Cargo.toml Cargo.lock build.rs ./
COPY src ./src
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=builder /app/target/release/backend /app/backend
ENV HOST=0.0.0.0
ENV PORT=8080
EXPOSE 8080
CMD ["/app/backend"]

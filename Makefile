.PHONY: setup reset-db migrate-db run check db-logs sqlx-prepare dev-up

# Regenera .sqlx/ contra Postgres local (requiere Docker + esquema aplicado).
sqlx-prepare:
	@DATABASE_URL="postgres://panas:secret123@127.0.0.1:5432/panas?options=-csearch_path%3Dpanas" \
		cargo sqlx prepare

setup:
	@./scripts/setup.sh

migrate-db:
	@./scripts/migrate-db.sh

reset-db:
	@./scripts/reset-db.sh

dev-up:
	@chmod +x scripts/*.sh
	@./scripts/dev-up.sh

run:
	@cargo run

check:
	@cargo check

db-logs:
	@docker compose logs -f db

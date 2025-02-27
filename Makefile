.PHONY: shell migrate

shell:
	docker exec -it -w /blokmap-backend blokmap-dev-backend /bin/sh

migrate:
	docker exec -w /blokmap-backend blokmap-dev-backend alembic upgrade head

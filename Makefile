.PHONY: shell migrate

shell:
	docker exec -it -w /blokmap-backend blokmap-backend /bin/sh

migrate:
	docker exec -w /blokmap-backend blokmap-backend alembic upgrade head

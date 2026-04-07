.PHONY: dev infra down logs build ps clean restart

## Lance toute l'infra (postgres, redis, rabbitmq)
infra:
	docker compose up postgres redis rabbitmq -d

## Lance tous les services
dev:
	docker compose up -d

## Arrête tout
down:
	docker compose down

## Rebuild toutes les images
build:
	docker compose build

## Voir les conteneurs qui tournent
ps:
	docker compose ps

## Voir les logs en temps réel
logs:
	docker compose logs -f

## Nettoyer tout (conteneurs + volumes)
clean:
	docker compose down -v --remove-orphans

## Restart complet
restart: down dev

## Logs par service
logs-gateway:
	docker compose logs -f api-gateway

logs-scraper:
	docker compose logs -f scraper-service

logs-pricing:
	docker compose logs -f pricing-service

logs-cotation:
	docker compose logs -f cotation-service

logs-user:
	docker compose logs -f user-service
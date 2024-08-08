# TicketApp

## Dev tools

```bash
cargo install sqlx-cli --no-default-features --features native-tls,postgres
cargo install cargo-watch
npm i
```

## Dev env

```bash
# runs docker compose and launches migration
./setup_dev_env.sh
. db_env.sh
```

## Prod env

```bash
# app on http://localhost:80
docker compose -f docker-compose.yaml -f docker-compose-prod.yaml up
```

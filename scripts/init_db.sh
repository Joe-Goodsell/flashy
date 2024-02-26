#!/usr/bin/env bash

# Set debug mode
set -x

# Ensures that any command or pipeline that fails
# will immediately terminate script adn print error msgs
set -eo pipefail

if ! [ -x  "$(command -v psql)" ]; then
    echo >&2 "Error: psql not installed"
    exit 1
fi

if ! [ -x "$(command -v sqlx)" ]; then
    echo >&2 "Error: sqlx not installed"
    exit 1
fi

# DB_USER="${POSTGRES_USER:=josephgoodsell}"
# DB_PASSWORD="${POSTGRES_PASSWORD:=password}"
# DB_NAME="{$POSTGRES_DB:=cards}"
# DB_PORT="{$POSTGRES_PORT:=5432}"

DB_USER="josephgoodsell"
DB_PASSWORD="password"
DB_NAME="cards"
DB_PORT="5432"


# `-z`: if string is null
if [[ -z ${SKIP_DOCKER} ]]; then
    >&2 echo "Creating docker container..."
    docker run \
        -e POSTGRES_USER=${DB_USER} \
        -e POSTGRES_PASSWORD=${DB_PASSWORD} \
        -e POSTGRES_DB=${DB_NAME} \
        -e PGDATA=/usr/local/var/postgres \
        -p "${DB_PORT}":5432 \
        -d postgres postgres -N 1000
        # postgres -N 1000
        # Notes:
        # Run image `postgres` in detached mode
        # -d postgres \ 
        # Execute `postgres -N 1000` inside container; sets max postgres connections to 1000
        # postgres -N 1000
    >&2 echo "Done!"
else
    >&2 echo "Skipping docker run"
fi

# Start postgres, retrying if unavailable
# sets PGPASSWORD env variable to non-interactively input password in psql
>&2 echo "Connecting with psql..."
export PGPASSWORD="${DB_PASSWORD}"
until psql -h "localhost" -U "${DB_USER}" -p "${DB_PORT}" -d "${DB_NAME}" -c "\q"; do
    >&2 echo "Postgres sleeping. Retrying..."
    sleep 1
done

>&2 echo "Postgres is running on port ${DB_PORT}!"

>&2 echo "Migrating DB..."
export DATABASE_URL=postgres://${DB_USER}:${DB_PASSWORD}@localhost:${DB_PORT}/${DB_NAME}
sqlx database create
sqlx migrate run
>&2 echo "Finished."
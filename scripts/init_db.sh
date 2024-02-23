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
# DB_NAME="{$POSTGRES_DB:=newsletter}"
# DB_PORT="{$POSTGRES_PORT:=5432}"

DB_USER="josephgoodsell"
DB_PASSWORD="password"
DB_NAME="cards"
DB_PORT="5432"

if [[ -z "{SKIP_DOCKER}" ]]; then
    docker run \
        -e POSTGRES_USER=${DB_USER} \
        -e POSTGRES_PASSWORD=${DB_PASSWORD} \
        -e POSTGRES_DB=${DB_NAME} \
        -p "${DB_PORT}":5432 \
        -d postgres \
        postgres -N 1000
fi

>&2 echo "Postgres is running on port ${DB_PORT}!"

export DATABASE_URL=postgres://${DB_USER}:${DB_PASSWORD}@localhost:${DB_PORT}/${DB_NAME}
sqlx database create
sqlx migrate run
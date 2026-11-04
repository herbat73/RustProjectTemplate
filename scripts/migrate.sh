#!/usr/bin/env bash
# Usage: scripts/migrate.sh [postgres|sqlite]   (default: postgres)
# Creates the database if it does not exist, then runs the migrations.
set -euo pipefail

cd "$(dirname "$0")/.."

DB="${1:-postgres}"

# Load .env if present (DATABASE_URL may also come from the environment)
if [ -f .env ]; then
  set -a; . ./.env; set +a
fi

: "${DATABASE_URL:?DATABASE_URL is not set}"

# `sqlx database create` is a no-op when the database already exists
sqlx database create
sqlx migrate run --source "migrations/${DB}"

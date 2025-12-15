#!/bin/bash

# Set environment variables for testing
export $(grep -v '^#' .env.test | xargs)

# Create test database if it doesn't exist
psql -U postgres -tc "SELECT 1 FROM pg_database WHERE datname = 'cyber_forum_test'" | grep -q 1 || \
    psql -U postgres -c "CREATE DATABASE cyber_forum_test"

# Run migrations
sqlx migrate run --database-url "$DATABASE_URL"

# Run tests
cargo test -- --nocapture

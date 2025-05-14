#!/bin/bash

echo "Testing PostgreSQL connection"

# Check if DATABASE_URL is set
if [ -n "$DATABASE_URL" ]; then
    echo "Found DATABASE_URL: $DATABASE_URL"
else
    echo "DATABASE_URL not set"
fi

# Check other PG environment variables
for var in PGHOST PGUSER PGPASSWORD PGDATABASE PGPORT; do
    if [ -n "${!var}" ]; then
        echo "$var is set to: ${!var}"
    else
        echo "$var is not set"
    fi
done

# Try to connect to PostgreSQL
if command -v psql &> /dev/null; then
    echo "Attempting to connect with psql..."
    PGPASSWORD=$PGPASSWORD psql -h $PGHOST -U $PGUSER -d $PGDATABASE -p $PGPORT -c "SELECT version();"
else
    echo "psql not found, cannot test connection directly"
fi

echo "Test complete"
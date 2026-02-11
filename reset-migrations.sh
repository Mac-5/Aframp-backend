#!/bin/bash

# Reset Migrations Script
# This script resets the migration state to match the current migration files

set -e

echo "🔧 Resetting migration state..."

# Database name
DB_NAME="aframp"

# Check if database exists
if ! psql -lqt | cut -d \| -f 1 | grep -qw "$DB_NAME"; then
    echo "❌ Database $DB_NAME does not exist"
    exit 1
fi

echo "📊 Current migration state:"
psql -d "$DB_NAME" -c "SELECT version, description, success FROM _sqlx_migrations ORDER BY version;"

echo ""
echo "🗑️  Clearing migration tracking table..."
psql -d "$DB_NAME" -c "DELETE FROM _sqlx_migrations;"

echo ""
echo "✅ Migration tracking cleared"
echo ""
echo "Now run: sqlx migrate run --database-url postgresql:///aframp"

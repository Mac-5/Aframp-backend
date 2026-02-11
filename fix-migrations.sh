#!/bin/bash

# Fix Migration State Script
# This script fixes the migration state when tables exist but aren't tracked

set -e

echo "🔧 Fixing migration state..."

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
echo "🔍 Checking which migrations need to be marked as applied..."

# Check if migration 20260123040000 tables exist
if psql -d "$DB_NAME" -tAc "SELECT EXISTS (SELECT FROM pg_tables WHERE schemaname = 'public' AND tablename = 'payment_provider_configs');" | grep -q 't'; then
    echo "✅ Migration 20260123040000 tables exist"
    
    # Mark as applied if not already
    if ! psql -d "$DB_NAME" -tAc "SELECT EXISTS (SELECT FROM _sqlx_migrations WHERE version = 20260123040000);" | grep -q 't'; then
        echo "📝 Marking migration 20260123040000 as applied..."
        psql -d "$DB_NAME" -c "INSERT INTO _sqlx_migrations (version, description, success, checksum, execution_time, installed_on) VALUES (20260123040000, 'implement payments schema', true, E'\\x', 0, now());"
    else
        echo "ℹ️  Migration 20260123040000 already marked as applied"
    fi
fi

# Check if migration 20260124000000 tables/indexes exist
if psql -d "$DB_NAME" -tAc "SELECT EXISTS (SELECT FROM pg_indexes WHERE schemaname = 'public' AND indexname = 'idx_transactions_wallet_address');" | grep -q 't'; then
    echo "✅ Migration 20260124000000 indexes exist"
    
    # Mark as applied if not already
    if ! psql -d "$DB_NAME" -tAc "SELECT EXISTS (SELECT FROM _sqlx_migrations WHERE version = 20260124000000);" | grep -q 't'; then
        echo "📝 Marking migration 20260124000000 as applied..."
        psql -d "$DB_NAME" -c "INSERT INTO _sqlx_migrations (version, description, success, checksum, execution_time, installed_on) VALUES (20260124000000, 'indexes and constraints', true, E'\\x', 0, now());"
    else
        echo "ℹ️  Migration 20260124000000 already marked as applied"
    fi
fi

# Check if migration 20260125000000 tables exist
if psql -d "$DB_NAME" -tAc "SELECT EXISTS (SELECT FROM pg_tables WHERE schemaname = 'public' AND tablename = 'exchange_rates');" | grep -q 't'; then
    echo "✅ Migration 20260125000000 tables exist"
    
    # Mark as applied if not already
    if ! psql -d "$DB_NAME" -tAc "SELECT EXISTS (SELECT FROM _sqlx_migrations WHERE version = 20260125000000);" | grep -q 't'; then
        echo "📝 Marking migration 20260125000000 as applied..."
        psql -d "$DB_NAME" -c "INSERT INTO _sqlx_migrations (version, description, success, checksum, execution_time, installed_on) VALUES (20260125000000, 'add repository tables', true, E'\\x', 0, now());"
    else
        echo "ℹ️  Migration 20260125000000 already marked as applied"
    fi
fi

echo ""
echo "📊 Updated migration state:"
psql -d "$DB_NAME" -c "SELECT version, description, success FROM _sqlx_migrations ORDER BY version;"

echo ""
echo "✅ Migration state fixed!"
echo ""
echo "You can now run ./setup.sh again without errors."

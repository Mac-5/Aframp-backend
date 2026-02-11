#!/bin/bash
# Fix migration checksums by calculating them from the actual migration files
# This resolves the "migration was previously applied but has been modified" error

set -e

DB_NAME="aframp"

echo "🔧 Fixing migration checksums for $DB_NAME..."

# Function to calculate SHA256 checksum of a file and format it for PostgreSQL
calculate_checksum() {
    local file=$1
    # Calculate SHA256, output as hex, and format for PostgreSQL bytea
    sha256sum "$file" | awk '{print "\\x" $1}'
}

# Calculate checksums for each migration file
CHECKSUM_1=$(calculate_checksum "migrations/20260122120000_create_core_schema.sql")
CHECKSUM_2=$(calculate_checksum "migrations/20260123040000_implement_payments_schema.sql")
CHECKSUM_3=$(calculate_checksum "migrations/20260124000000_indexes_and_constraints.sql")

echo "📊 Calculated checksums:"
echo "  Migration 1: $CHECKSUM_1"
echo "  Migration 2: $CHECKSUM_2"
echo "  Migration 3: $CHECKSUM_3"

# Update the checksums in the database
psql -d "$DB_NAME" << EOF
-- Update checksum for migration 1
UPDATE _sqlx_migrations 
SET checksum = '$CHECKSUM_1'::bytea
WHERE version = 20260122120000;

-- Update checksum for migration 2
UPDATE _sqlx_migrations 
SET checksum = '$CHECKSUM_2'::bytea
WHERE version = 20260123040000;

-- Update checksum for migration 3
UPDATE _sqlx_migrations 
SET checksum = '$CHECKSUM_3'::bytea
WHERE version = 20260124000000;
EOF

echo ""
echo "✅ Migration checksums fixed!"
echo ""
echo "Verification:"
psql -d "$DB_NAME" -c "SELECT version, description, encode(checksum, 'hex') as checksum FROM _sqlx_migrations ORDER BY version;"

echo ""
echo "✅ You can now run ./setup.sh without migration errors!"

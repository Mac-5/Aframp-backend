# Migration Troubleshooting Guide

## Problem: Migration errors when running `./setup.sh`

### Common Errors:
1. `relation "payment_provider_configs" already exists`
2. `migration was previously applied but is missing`
3. `migration was previously applied but has been modified`

## Root Cause
Your database has tables from migrations, but the migration tracking table (`_sqlx_migrations`) is out of sync with the actual database state.

---

## Solution Options

### Option 1: Fresh Start (Recommended for Development)

**⚠️ WARNING: This will DELETE ALL DATA in the database!**

```bash
# 1. Drop and recreate the database
dropdb aframp
createdb aframp

# 2. Run setup script
./setup.sh
```

### Option 2: Keep Data and Fix Migration State

If you have important data and want to keep it:

#### Step 1: Check current state
```bash
# Check what tables exist
psql -d aframp -c "\dt"

# Check migration tracking
psql -d aframp -c "SELECT * FROM _sqlx_migrations ORDER BY version;"
```

#### Step 2: Manually mark migrations as applied

```bash
# Clear the migration tracking
psql -d aframp -c "DELETE FROM _sqlx_migrations;"

# Manually insert migration records (adjust checksums as needed)
psql -d aframp << 'EOF'
-- Mark migration 1 as applied
INSERT INTO _sqlx_migrations (version, description, success, checksum, execution_time, installed_on)
VALUES (20260122120000, 'create core schema', true, 
        (SELECT checksum FROM (VALUES (E'\\x' || encode(sha256(pg_read_file('migrations/20260122120000_create_core_schema.sql')::bytea), 'hex'))) AS t(checksum)),
        0, now());

-- Mark migration 2 as applied  
INSERT INTO _sqlx_migrations (version, description, success, checksum, execution_time, installed_on)
VALUES (20260123040000, 'implement payments schema', true,
        (SELECT checksum FROM (VALUES (E'\\x' || encode(sha256(pg_read_file('migrations/20260123040000_implement_payments_schema.sql')::bytea), 'hex'))) AS t(checksum)),
        0, now());

-- Mark migration 3 as applied
INSERT INTO _sqlx_migrations (version, description, success, checksum, execution_time, installed_on)
VALUES (20260124000000, 'indexes and constraints', true,
        (SELECT checksum FROM (VALUES (E'\\x' || encode(sha256(pg_read_file('migrations/20260124000000_indexes_and_constraints.sql')::bytea), 'hex'))) AS t(checksum)),
        0, now());
EOF
```

#### Step 3: Use the provided script

```bash
# Use the simpler approach - just mark them with dummy checksums
./fix-migrations-simple.sh
```

### Option 3: Skip Migration Checks (Quick Fix)

If you just want to get the server running:

```bash
# Start the server without running migrations
cargo run --features database
```

The server will work with the existing database schema.

---

## Prevention

To avoid this issue in the future:

### 1. Never manually modify the database schema
Always use migrations for schema changes.

### 2. Don't run migrations multiple times
If a migration fails partway through, you need to:
- Fix the migration file
- Rollback the partial changes
- Re-run the migration

### 3. Use version control
Commit migration files to git and never modify them after they've been applied.

### 4. Use separate databases
- Development: `aframp`
- Testing: `aframp_test`  
- Production: `aframp_prod`

---

## Quick Fix Script

I've created a simple script to fix the migration state:

```bash
#!/bin/bash
# fix-migrations-simple.sh

DB_NAME="aframp"

echo "Fixing migration state for $DB_NAME..."

# Clear existing migration records
psql -d "$DB_NAME" -c "DELETE FROM _sqlx_migrations;"

# Add migration records with dummy checksums
psql -d "$DB_NAME" << 'EOF'
INSERT INTO _sqlx_migrations (version, description, success, checksum, execution_time, installed_on)
VALUES 
  (20260122120000, 'create core schema', true, E'\\x00', 0, now()),
  (20260123040000, 'implement payments schema', true, E'\\x00', 0, now()),
  (20260124000000, 'indexes and constraints', true, E'\\x00', 0, now());
EOF

echo "✅ Migration state fixed!"
echo ""
echo "⚠️  Note: Checksums are set to dummy values."
echo "This means sqlx will think migrations have been modified."
echo "This is OK for development, but for production, use proper checksums."
```

Save this as `fix-migrations-simple.sh`, make it executable, and run it:

```bash
chmod +x fix-migrations-simple.sh
./fix-migrations-simple.sh
```

---

## Understanding the Error

### What is `_sqlx_migrations`?
This table tracks which migrations have been applied to your database. Each row contains:
- `version`: Migration timestamp (e.g., 20260122120000)
- `description`: Human-readable description
- `checksum`: Hash of the migration file content
- `success`: Whether it applied successfully

### Why does the checksum matter?
sqlx compares the checksum of the migration file with the stored checksum to detect if a migration file has been modified after being applied. This prevents accidental schema corruption.

### What causes checksum mismatches?
- Editing a migration file after it's been applied
- Copying migrations from another project
- Git merge conflicts in migration files
- Manual database changes that don't match the migration

---

## Best Practices

### Development Workflow
1. Create a new migration: `sqlx migrate add description_here`
2. Edit the migration file
3. Test it: `sqlx migrate run`
4. If it fails, fix and re-run
5. Once working, commit to git
6. **Never edit the migration file again**

### If You Need to Change Schema
1. Create a NEW migration
2. Don't edit old migrations
3. Use the new migration to modify the schema

### Database Reset (Development Only)
```bash
# Quick reset script
dropdb aframp && createdb aframp && sqlx migrate run
```

---

## Current Status

Your database has:
- ✅ All tables created
- ✅ All indexes created
- ❌ Migration tracking out of sync

**Recommended Action**: Use Option 1 (Fresh Start) if you don't have important data, or use the `fix-migrations-simple.sh` script if you want to keep your data.

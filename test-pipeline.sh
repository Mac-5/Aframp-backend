#!/bin/bash
set -e

echo "=== Testing CI/CD Pipeline Locally ==="

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
NC='\033[0m'

# Check prerequisites
echo "Checking prerequisites..."
command -v docker >/dev/null 2>&1 || { echo -e "${RED}Docker not found${NC}"; exit 1; }
command -v cargo >/dev/null 2>&1 || { echo -e "${RED}Cargo not found${NC}"; exit 1; }

# Start services
echo -e "\n${GREEN}Starting PostgreSQL and Redis...${NC}"
docker run -d --name aframp-postgres-test \
  -e POSTGRES_PASSWORD=postgres \
  -e POSTGRES_DB=aframp_test \
  -p 5432:5432 \
  postgres:14 >/dev/null 2>&1 || echo "PostgreSQL already running"

docker run -d --name aframp-redis-test \
  -p 6379:6379 \
  redis:6 >/dev/null 2>&1 || echo "Redis already running"

# Wait for services
echo "Waiting for services to be ready..."
sleep 5

# Run tests
echo -e "\n${GREEN}Running tests...${NC}"
export DATABASE_URL=postgresql://postgres:postgres@localhost:5432/aframp_test
export REDIS_URL=redis://localhost:6379
cargo test --features database -- --test-threads=4

# Run clippy
echo -e "\n${GREEN}Running clippy...${NC}"
cargo clippy --features database -- -D warnings

# Security audit
echo -e "\n${GREEN}Running security audit...${NC}"
cargo install cargo-audit 2>/dev/null || true
cargo audit

# Build Docker image
echo -e "\n${GREEN}Building Docker image...${NC}"
docker build -t aframp-backend:test .

# Scan with Trivy (if available)
if command -v trivy >/dev/null 2>&1; then
  echo -e "\n${GREEN}Scanning image with Trivy...${NC}"
  trivy image aframp-backend:test
else
  echo -e "\n${RED}Trivy not installed, skipping vulnerability scan${NC}"
fi

# Cleanup
echo -e "\n${GREEN}Cleaning up...${NC}"
docker stop aframp-postgres-test aframp-redis-test 2>/dev/null || true
docker rm aframp-postgres-test aframp-redis-test 2>/dev/null || true

echo -e "\n${GREEN}=== All checks passed! ===${NC}"

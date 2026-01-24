# CI/CD Pipeline Testing Guide

## Files Created

1. **Dockerfile** - Multi-stage build for Rust backend
2. **.dockerignore** - Optimizes Docker build context
3. **.github/workflows/ci-cd.yml** - Main CI/CD pipeline
4. **.github/workflows/validate.yml** - Quick validation workflow
5. **test-pipeline.sh** - Local testing script

## Testing the Pipeline

### Option 1: Local Testing (Recommended First)

```bash
./test-pipeline.sh
```

This script will:
- Start PostgreSQL and Redis containers
- Run cargo tests with database feature
- Run clippy checks
- Run cargo audit for security
- Build Docker image
- Scan with Trivy (if installed)
- Clean up containers

### Option 2: GitHub Actions Validation

1. Commit and push the changes:
```bash
git add .
git checkout -b test-cicd
git commit -m "Add CI/CD pipeline"
git push origin test-cicd
```

2. Go to GitHub Actions tab
3. Run "Pipeline Validation" workflow manually
4. Create a PR to trigger the full CI/CD pipeline

### Option 3: Manual Docker Build Test

```bash
# Build the image
docker build -t aframp-backend:test .

# Run the container (requires env vars)
docker run -p 8000:8000 \
  -e DATABASE_URL=postgresql://user:pass@host/db \
  -e REDIS_URL=redis://host:6379 \
  aframp-backend:test
```

## Pipeline Stages

### On Pull Request:
- ✅ Run tests (with PostgreSQL + Redis)
- ✅ Run clippy
- ✅ Run cargo audit

### On Main Branch Push:
- ✅ All PR checks
- ✅ Build Docker image
- ✅ Push to GitHub Container Registry
- ✅ Scan with Trivy
- ✅ Deploy to staging (auto)
- ✅ Deploy to production (manual approval)

## Next Steps

1. **Test locally first**: Run `./test-pipeline.sh`
2. **Configure GitHub secrets**:
   - Go to Settings → Secrets and variables → Actions
   - Add deployment credentials if needed
3. **Set up environments**:
   - Go to Settings → Environments
   - Create `staging` and `production`
   - Add protection rules for production
4. **Update deployment commands**:
   - Edit `.github/workflows/ci-cd.yml`
   - Replace placeholder deployment commands with actual ones
5. **Add notifications**:
   - Configure Slack webhook or email in failure steps

## Troubleshooting

**Tests fail locally:**
- Ensure PostgreSQL and Redis are running
- Check DATABASE_URL and REDIS_URL are correct

**Docker build fails:**
- Check Rust version compatibility
- Verify all dependencies are in Cargo.toml

**GitHub Actions fails:**
- Check workflow logs in Actions tab
- Verify secrets are configured correctly
- Ensure branch protection rules allow workflow runs

## Features Implemented

✅ Automated testing with parallelization
✅ Cargo dependency caching
✅ Multi-stage Docker builds
✅ Image tagging (SHA + semantic versions)
✅ Security scanning (cargo audit + Trivy)
✅ Staging deployment automation
✅ Production deployment with manual approval
✅ Health check placeholders
✅ Failure notification hooks
✅ Rollback capability via environment history

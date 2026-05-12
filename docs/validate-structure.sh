#!/bin/bash

# Script de validation de la structure de documentation SSHive

set -e

DOCS_DIR="$(cd "$(dirname "$0")" && pwd)"
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo "=== SSHive Documentation Structure Validation ==="
echo ""

MISSING=0
FOUND=0

# Configuration files
echo "Checking configuration files..."
for file in astro.config.mjs package.json tsconfig.json .gitignore src/content/config.ts; do
  if [ -f "$DOCS_DIR/$file" ]; then
    echo "  ${GREEN}✓${NC} $file"
    ((FOUND++))
  else
    echo "  ${RED}✗${NC} $file (missing)"
    ((MISSING++))
  fi
done

echo ""
echo "Checking content files..."
for file in \
  src/content/docs/index.mdx \
  src/content/docs/installation.md \
  src/content/docs/quickstart.md \
  src/content/docs/guide/services.md \
  src/content/docs/guide/keys.md \
  src/content/docs/guide/deployment.md \
  src/content/docs/guide/health.md \
  src/content/docs/guide/tokens.md \
  src/content/docs/reference/configuration.md \
  src/content/docs/reference/security.md \
  src/content/docs/reference/changelog.md \
  src/assets/logo.svg \
  src/styles/custom.css; do
  if [ -f "$DOCS_DIR/$file" ]; then
    echo "  ${GREEN}✓${NC} $file"
    ((FOUND++))
  else
    echo "  ${RED}✗${NC} $file (missing)"
    ((MISSING++))
  fi
done

echo ""
echo "Checking build files..."
for file in .github/workflows/docs.yml README.md QUICK_START.md STRUCTURE.md MANIFEST.txt test-build.sh validate-structure.sh .env.example; do
  if [ -f "$DOCS_DIR/$file" ]; then
    echo "  ${GREEN}✓${NC} $file"
    ((FOUND++))
  else
    echo "  ${RED}✗${NC} $file (missing)"
    ((MISSING++))
  fi
done

echo ""
echo "=== Validation Summary ==="
echo "Found: $FOUND files"
if [ $MISSING -gt 0 ]; then
  echo -e "${RED}Missing: $MISSING files${NC}"
  exit 1
else
  echo -e "${GREEN}All files present!${NC}"
  echo ""
  echo "Next steps:"
  echo "  cd docs"
  echo "  npm install"
  echo "  npm run dev    (development)"
  echo "  npm run build  (production)"
fi

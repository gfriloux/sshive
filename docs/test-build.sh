#!/bin/bash
set -e

echo "Testing SSHive documentation build..."
echo ""

cd "$(dirname "$0")"

echo "1. Installing dependencies..."
npm install --silent 2>&1 | tail -5

echo ""
echo "2. Building documentation..."
npm run build 2>&1 | tail -20

echo ""
echo "✓ Build successful!"
echo ""
echo "Output directory: $(pwd)/dist"
echo "Next steps:"
echo "  - npm run dev    (development server)"
echo "  - npm run preview (preview production build)"

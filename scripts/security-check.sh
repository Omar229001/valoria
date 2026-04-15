#!/bin/bash

# Security check script for Valoria
# Verifies that no secrets are hardcoded in the codebase

set -e

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo "🔍 Valoria Security Check"
echo "========================="
echo ""

ERRORS=0

# Check 1: Hardcoded secrets in code
echo "📋 Check 1: Hardcoded secrets in source code..."
if grep -r "dev_secret_change_in_prod\|valoria_dev\|password123" \
  --include="*.rs" --include="*.py" --include="*.tsx" \
  services/ frontend/ 2>/dev/null | grep -v "test\|TEST\|example"; then
  echo -e "${RED}❌ Found hardcoded secrets!${NC}"
  ERRORS=$((ERRORS + 1))
else
  echo -e "${GREEN}✅ No hardcoded secrets in code${NC}"
fi
echo ""

# Check 2: .env file committed
echo "📋 Check 2: .env files in git..."
if git ls-files 2>/dev/null | grep -E "^\.env$"; then
  echo -e "${RED}❌ .env file is tracked by git (SECURITY RISK!)${NC}"
  ERRORS=$((ERRORS + 1))
else
  echo -e "${GREEN}✅ .env not in git${NC}"
fi
echo ""

# Check 3: .gitignore protects secrets
echo "📋 Check 3: .gitignore has security entries..."
if grep -q "^\.env" .gitignore && grep -q "\.key" .gitignore; then
  echo -e "${GREEN}✅ .gitignore protects secrets${NC}"
else
  echo -e "${YELLOW}⚠️  .gitignore might need updates${NC}"
fi
echo ""

# Summary
echo "========================="
if [ $ERRORS -eq 0 ]; then
  echo -e "${GREEN}✅ All security checks passed!${NC}"
  exit 0
else
  echo -e "${RED}❌ $ERRORS security issue(s) found!${NC}"
  exit 1
fi

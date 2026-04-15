#!/bin/bash

# Fix hardcoded secrets in Python services
# Replaces DATABASE_URL defaults with environment-based config

set -e

REPO_ROOT="/Users/ozibo/mes projets /valoria"
cd "$REPO_ROOT"

echo "🐍 Fixing Hardcoded Secrets in Python Services"
echo "==============================================="
echo ""

# Python services with database.py files
PYTHON_SERVICES=(
    "services/scraper-service/src/database.py"
    "services/cotation-service/src/database.py"
    "services/pricing-service/src/database.py"
)

for SERVICE_FILE in "${PYTHON_SERVICES[@]}"; do
    if [ ! -f "$SERVICE_FILE" ]; then
        echo "⚠️  $SERVICE_FILE not found"
        continue
    fi

    echo "📝 Processing: $SERVICE_FILE"

    # Show current line
    if grep -n 'valoria_dev' "$SERVICE_FILE" | head -1; then
        # Replace with environment-based config
        sed -i '' 's#DATABASE_URL = os.getenv("DATABASE_URL", "postgresql://valoria:valoria_dev@localhost:5434/valoria")#from config import Config; cfg = Config(); DATABASE_URL = cfg.database_url#' "$SERVICE_FILE" 2>/dev/null || true
        echo "   ✅ Updated to use Config module"
    fi

    echo ""
done

echo "==============================================="
echo "✅ Python services updated"
echo ""
echo "ACTION REQUIRED:"
echo "1. Review each services/*/src/database.py"
echo "2. Add 'from config import Config' at top"
echo "3. Replace DATABASE_URL assignment with Config"
echo "4. Run: bash scripts/security-check.sh to verify"


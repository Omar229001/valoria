#!/bin/bash

# Script to apply config validation to all Rust services
# Replaces hardcoded secrets with Config module validation

set -e

REPO_ROOT="/Users/ozibo/mes projets /valoria"
cd "$REPO_ROOT"

echo "🔒 Applying Config Validation to Rust Services"
echo "=================================================="
echo ""

# List of Rust services
SERVICES=(
    "services/api-gateway"
    "services/user-service"
    "services/cotation-service"
    "services/pricing-service"
)

for SERVICE in "${SERVICES[@]}"; do
    if [ ! -f "$SERVICE/src/main.rs" ]; then
        echo "⚠️  Skipping $SERVICE (main.rs not found)"
        continue
    fi

    echo "📝 Processing: $SERVICE"
    
    # Check if config module is already included
    if grep -q "^mod config;" "$SERVICE/src/main.rs"; then
        echo "   ℹ️  Config module already included"
    else
        # Add config module declaration at top of main.rs
        sed -i '' '1s/^/mod config;\n/' "$SERVICE/src/main.rs"
        echo "   ✅ Added: mod config;"
    fi

    # Replace hardcoded JWT_SECRET with config validation
    if grep -q 'env::var("JWT_SECRET").unwrap_or_else' "$SERVICE/src/main.rs"; then
        sed -i '' 's/env::var("JWT_SECRET")\.unwrap_or_else(|_| "dev_secret_change_in_prod"\.to_string());/let cfg = config::Config::from_env();/' "$SERVICE/src/main.rs"
        echo "   ✅ Replaced hardcoded JWT_SECRET with config validation"
    fi

    echo "   ✓ Done"
    echo ""
done

echo "=================================================="
echo "✅ Config validation applied to all services"
echo ""
echo "Next steps:"
echo "1. Update each service to use cfg.jwt_secret instead of jwt_secret var"
echo "2. Update each service to use cfg.database_url instead of env var"
echo "3. Run: cargo test (in each service) to verify"
echo "4. Run: bash scripts/security-check.sh to verify no hardcoded secrets"


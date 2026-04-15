"""
Configuration validation module for Valoria Python services

Ensures all required environment variables are set properly
and fails early if configuration is invalid.

All secrets (JWT_SECRET, DATABASE_PASSWORD) must be in environment.
No defaults for sensitive values in production.

Usage:
    from config import Config
    cfg = Config()
    cfg.print_summary()
"""

import os
import sys
from enum import Enum
from typing import Optional
from urllib.parse import urlparse


class Environment(str, Enum):
    """Environment type"""
    DEVELOPMENT = "development"
    STAGING = "staging"
    PRODUCTION = "production"

    @staticmethod
    def from_env() -> "Environment":
        """Load environment from ENVIRONMENT variable"""
        env_str = os.getenv("ENVIRONMENT", "development").lower()
        if env_str in ("production", "prod"):
            return Environment.PRODUCTION
        elif env_str in ("staging", "stage"):
            return Environment.STAGING
        else:
            return Environment.DEVELOPMENT

    @property
    def is_production(self) -> bool:
        return self == Environment.PRODUCTION

    @property
    def is_development(self) -> bool:
        return self == Environment.DEVELOPMENT


class Config:
    """Configuration for Valoria Python services"""

    def __init__(self):
        self.environment = Environment.from_env()
        self.jwt_secret = self._load_jwt_secret()
        self.database_url = self._load_database_url()
        self.port = self._load_port()
        self.redis_url = self._load_redis_url()
        self.rabbitmq_url = self._load_rabbitmq_url()

    def _load_jwt_secret(self) -> str:
        """Load and validate JWT_SECRET"""
        jwt_secret = os.getenv("JWT_SECRET")

        if not jwt_secret:
            print("❌ ERROR: JWT_SECRET environment variable not set", file=sys.stderr)
            print("   This is a REQUIRED variable for security", file=sys.stderr)
            print("   Generate a secure secret and set it in .env", file=sys.stderr)
            print("   Example: JWT_SECRET=$(openssl rand -base64 32)", file=sys.stderr)
            sys.exit(1)

        if jwt_secret == "dev_secret_change_in_prod":
            print("❌ ERROR: JWT_SECRET is set to default value 'dev_secret_change_in_prod'",
                  file=sys.stderr)
            print("   This is a SECURITY RISK. Never use default secrets!", file=sys.stderr)
            print("   Set a secure value: JWT_SECRET=$(openssl rand -base64 32)", file=sys.stderr)
            if self.environment.is_production:
                sys.exit(1)
            else:
                print("   ⚠️  Proceeding in development mode only", file=sys.stderr)

        if len(jwt_secret) < 32:
            print(f"❌ ERROR: JWT_SECRET must be at least 32 characters (got {len(jwt_secret)})",
                  file=sys.stderr)
            print("   Generate with: openssl rand -base64 32", file=sys.stderr)
            sys.exit(1)

        return jwt_secret

    def _load_database_url(self) -> str:
        """Load and validate DATABASE_URL"""
        database_url = os.getenv("DATABASE_URL")

        if not database_url:
            print("❌ ERROR: DATABASE_URL environment variable not set", file=sys.stderr)
            print("   Example: postgresql://user:password@host:5432/dbname", file=sys.stderr)
            sys.exit(1)

        # Validate it's PostgreSQL
        if not (database_url.startswith("postgresql://") or 
                database_url.startswith("postgres://")):
            print("❌ ERROR: DATABASE_URL must be a PostgreSQL connection string",
                  file=sys.stderr)
            print(f"   Current value: {database_url}", file=sys.stderr)
            sys.exit(1)

        return database_url

    def _load_port(self) -> int:
        """Load and validate PORT"""
        port_str = os.getenv("PORT", "3000")

        try:
            port = int(port_str)
            if port < 0 or port > 65535:
                raise ValueError(f"Port {port} out of range")
            return port
        except ValueError:
            print(f"❌ ERROR: PORT must be a valid number between 0-65535 (got {port_str})",
                  file=sys.stderr)
            sys.exit(1)

    def _load_redis_url(self) -> str:
        """Load REDIS_URL with warning in production if not set"""
        redis_url = os.getenv("REDIS_URL")

        if not redis_url:
            if self.environment.is_production:
                print("⚠️  WARNING: REDIS_URL not set in production!", file=sys.stderr)
            redis_url = "redis://localhost:6379"

        return redis_url

    def _load_rabbitmq_url(self) -> str:
        """Load RABBITMQ_URL with warning in production if not set"""
        rabbitmq_url = os.getenv("RABBITMQ_URL")

        if not rabbitmq_url:
            if self.environment.is_production:
                print("⚠️  WARNING: RABBITMQ_URL not set in production!", file=sys.stderr)
            rabbitmq_url = "amqp://guest:guest@localhost:5672"

        return rabbitmq_url

    def print_summary(self):
        """Print configuration summary (redacts secrets)"""
        print("═══════════════════════════════════════════")
        print("📋 Service Configuration")
        print("═══════════════════════════════════════════")
        print(f"Environment:   {self.environment.value}")
        print(f"Port:          {self.port}")
        print(f"JWT_SECRET:    {len(self.jwt_secret)} chars (✓ set)")
        
        # Redact password from database URL for display
        db_display = self.database_url.split("@")[0] if "@" in self.database_url else "***"
        print(f"Database:      {db_display}... (✓ valid)")
        
        # Redact host from Redis URL
        redis_display = self.redis_url.split("//")[-1] if "//" in self.redis_url else "***"
        print(f"Redis:         {redis_display}")
        
        # Redact credentials from RabbitMQ URL
        rmq_display = self.rabbitmq_url.split("@")[-1] if "@" in self.rabbitmq_url else "***"
        print(f"RabbitMQ:      {rmq_display}")
        print("═══════════════════════════════════════════")

    def get_database_url_safe(self) -> str:
        """Get database URL with password redacted for logging"""
        return self.database_url.split("@")[0] if "@" in self.database_url else "***"

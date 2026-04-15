//! Configuration validation module for Valoria services
//! 
//! Ensures all required environment variables are set properly
//! and panics early if configuration is invalid.
//!
//! All secrets (JWT_SECRET, DATABASE_PASSWORD) must be in environment.
//! No defaults for sensitive values in production.

use std::env;

#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub jwt_secret: String,
    pub port: u16,
    pub environment: Environment,
    pub redis_url: String,
    pub rabbitmq_url: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Environment {
    Development,
    Staging,
    Production,
}

impl Environment {
    pub fn from_env() -> Self {
        let env_str = env::var("ENVIRONMENT")
            .unwrap_or_else(|_| "development".to_string())
            .to_lowercase();
        
        match env_str.as_str() {
            "production" | "prod" => Environment::Production,
            "staging" | "stage" => Environment::Staging,
            _ => Environment::Development,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Environment::Development => "development",
            Environment::Staging => "staging",
            Environment::Production => "production",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "production" | "prod" => Environment::Production,
            "staging" | "stage" => Environment::Staging,
            _ => Environment::Development,
        }
    }
}

impl Config {
    /// Load configuration from environment variables
    /// Panics if required variables are missing or invalid
    pub fn from_env() -> Self {
        let environment = Environment::from_env();

        // Get JWT secret - MUST be set, MUST not be default value
        let jwt_secret = env::var("JWT_SECRET").unwrap_or_else(|_| {
            eprintln!("❌ ERROR: JWT_SECRET environment variable not set");
            eprintln!("   This is a REQUIRED variable for security");
            eprintln!("   Generate a secure secret and set it in .env");
            eprintln!("   Example: JWT_SECRET=$(openssl rand -base64 32)");
            std::process::exit(1);
        });

        // Validate JWT secret is not a default value
        if jwt_secret == "dev_secret_change_in_prod" {
            eprintln!("❌ ERROR: JWT_SECRET is set to default value 'dev_secret_change_in_prod'");
            eprintln!("   This is a SECURITY RISK. Never use default secrets in production!");
            eprintln!("   Set a secure value: JWT_SECRET=$(openssl rand -base64 32)");
            if environment == Environment::Production {
                std::process::exit(1);
            } else {
                eprintln!("   ⚠️  Proceeding in development mode only");
            }
        }

        if jwt_secret.len() < 32 {
            eprintln!("❌ ERROR: JWT_SECRET must be at least 32 characters");
            eprintln!("   Current length: {}", jwt_secret.len());
            eprintln!("   Generate with: openssl rand -base64 32");
            std::process::exit(1);
        }

        // Get database URL
        let database_url = env::var("DATABASE_URL").unwrap_or_else(|_| {
            eprintln!("❌ ERROR: DATABASE_URL environment variable not set");
            eprintln!("   Example: postgresql://user:password@host:5432/dbname");
            std::process::exit(1);
        });

        // Validate database URL
        if !database_url.starts_with("postgresql://") && !database_url.starts_with("postgres://") {
            eprintln!("❌ ERROR: DATABASE_URL must be a PostgreSQL URL");
            eprintln!("   Current value: {}", database_url);
            std::process::exit(1);
        }

        let port_str = env::var("PORT").unwrap_or_else(|_| {
            eprintln!("⚠️  PORT not set, defaulting to 3000");
            "3000".to_string()
        });

        // Validate port is a valid number
        let port: u16 = match port_str.parse() {
            Ok(p) => p,
            Err(_) => {
                eprintln!("❌ ERROR: PORT must be a valid number between 0-65535");
                eprintln!("   Current value: {}", port_str);
                std::process::exit(1);
            }
        };

        let redis_url = env::var("REDIS_URL")
            .unwrap_or_else(|_| {
                if environment == Environment::Production {
                    eprintln!("⚠️  WARNING: REDIS_URL not set in production!");
                }
                "redis://localhost:6379".to_string()
            });

        let rabbitmq_url = env::var("RABBITMQ_URL")
            .unwrap_or_else(|_| {
                if environment == Environment::Production {
                    eprintln!("⚠️  WARNING: RABBITMQ_URL not set in production!");
                }
                "amqp://guest:guest@localhost:5672".to_string()
            });

        Config {
            database_url,
            jwt_secret,
            port,
            environment,
            redis_url,
            rabbitmq_url,
        }
    }

    /// Check if running in production
    pub fn is_production(&self) -> bool {
        self.environment == Environment::Production
    }

    /// Check if running in development
    pub fn is_development(&self) -> bool {
        self.environment == Environment::Development
    }

    /// Print configuration summary (safe, no secrets)
    pub fn print_summary(&self) {
        println!("═══════════════════════════════════════════");
        println!("📋 Service Configuration");
        println!("═══════════════════════════════════════════");
        println!("Environment:   {}", self.environment.as_str());
        println!("Port:          {}", self.port);
        println!(
            "JWT_SECRET:    {} chars (✓ set)",
            self.jwt_secret.len()
        );
        println!(
            "Database:      {} (✓ valid)",
            self.database_url.split("@").next().unwrap_or("***")
        );
        println!("Redis:         {}", self.redis_url.split("//").last().unwrap_or("***"));
        println!(
            "RabbitMQ:      {}",
            self.rabbitmq_url.split("@").next().unwrap_or("***")
        );
        println!("═══════════════════════════════════════════");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_environment_from_str() {
        assert_eq!(Environment::from_str("production"), Environment::Production);
        assert_eq!(Environment::from_str("prod"), Environment::Production);
        assert_eq!(Environment::from_str("staging"), Environment::Staging);
        assert_eq!(Environment::from_str("development"), Environment::Development);
    }
}

# 🤝 Contributing to Valoria

Merci de vouloir contribuer à Valoria! Ce guide t'explique comment faire.

---

## 📋 Table des matières

1. [Code of Conduct](#code-of-conduct)
2. [Avant de commencer](#avant-de-commencer)
3. [Setup développement](#setup-développement)
4. [Workflow](#workflow)
5. [Guidelines](#guidelines)
6. [Processus PR](#processus-pr)

---

## 📜 Code of Conduct

En participant à ce projet, tu t'engages à :

- ✅ Respecter tous les contributeurs
- ✅ Fournir un feedback constructif
- ✅ Accepter les critiques
- ✅ Se concentrer sur ce qui est bon pour la communauté
- ❌ Pas de harcèlement, discrimination ou comportement toxique

---

## 🏁 Avant de commencer

### Types de contributions

1. **Bug fixes** - Corrections de bugs
2. **Features** - Nouvelles fonctionnalités
3. **Documentation** - README, guides, commentaires
4. **Tests** - Tests unitaires, intégration
5. **Refactoring** - Amélioration du code

### Comment commencer?

1. **Forke le repo** sur GitHub
2. **Clone ton fork** en local
3. **Crée une branche** pour ta feature
4. **Fais tes changements**
5. **Teste** localement
6. **Commit** avec messages clairs
7. **Push** vers ton fork
8. **Ouvre une PR** vers le repo main

---

## 🛠️ Setup développement

### Prérequis

```bash
# Rust 1.77+
rustup update
rustc --version

# Python 3.11+
python3 --version

# Node.js 18+
node --version

# Docker
docker --version
```

### Clone et setup

```bash
# Clone ton fork
git clone https://github.com/<your-username>/valoria.git
cd valoria

# Ajoute le upstream pour rester à jour
git remote add upstream https://github.com/valoria/valoria.git

# Copie la config d'exemple
cp .env.example .env

# Lance tous les services
make dev

# Vérifier que tout fonctionne
make ps
```

### Développement par service

#### Rust Services

```bash
cd services/user-service

# Build
cargo build

# Run
cargo run

# Tests
cargo test

# Linting
cargo clippy

# Format
cargo fmt
```

#### Python Services

```bash
cd services/scraper-service

# Virtual env
python -m venv venv
source venv/bin/activate  # Windows: venv\Scripts\activate

# Install deps
pip install -r requirements.txt

# Run
uvicorn src.main:app --reload

# Tests
pytest

# Linting
black src/
flake8 src/
```

#### Frontend

```bash
cd frontend

# Install
npm install

# Dev server
npm run start

# Build
npm run build

# Linting
npm run lint
```

---

## 🔄 Workflow

### 1. Crée une branche

```bash
# Mets à jour main
git checkout main
git pull upstream main

# Crée une branche descriptive
git checkout -b fix/login-validation
git checkout -b feature/price-history
git checkout -b docs/api-guide
```

**Conventions de noms:**
- `fix/description` - Bug fix
- `feature/description` - Nouvelle feature
- `docs/description` - Documentation
- `test/description` - Tests
- `refactor/description` - Refactoring

### 2. Fais tes changements

Écris du code clean:

```rust
// ✅ BON: Clair et documenté
/// Valide un token JWT et retourne les claims.
fn validate_jwt(token: &str, secret: &str) -> Result<Claims> {
    let mut validation = Validation::default();
    validation.validate_exp = true;
    decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &validation,
    ).map(|data| data.claims)
}

// ❌ MAUVAIS: Pas de type hints, pas documenté
fn validate_jwt(token, secret) {
    // WTF does this do?
    let validation = Validation::default();
    ...
}
```

### 3. Teste localement

```bash
# Tests Rust
cd services/user-service
cargo test

# Tests Python
cd services/scraper-service
pytest

# Tests Frontend
cd frontend
npm test

# Format
cargo fmt
black src/
eslint src/

# Lint
cargo clippy
flake8 src/
```

### 4. Commit tes changements

```bash
# Stage tes fichiers
git add .

# Commit avec message clair
git commit -m "Add JWT validation tests for expired tokens"

# Ou plusieurs commits logiques
git commit -m "Refactor user authentication module"
git commit -m "Add unit tests for bcrypt hashing"
```

**Conventions de commits:**
- `feat: Add X` - Nouvelle feature
- `fix: Resolve X` - Bug fix
- `docs: Update X` - Documentation
- `test: Add tests for X` - Tests
- `refactor: Improve X` - Refactoring
- `chore: Update deps` - Maintenance

**Exemples:**

```bash
git commit -m "feat: Add price history endpoint for cotations"
git commit -m "fix: Handle null values in scraper results"
git commit -m "docs: Add API rate limiting documentation"
git commit -m "test: Add tests for JWT expiration"
git commit -m "refactor: Extract database connection logic to module"
```

### 5. Garde ta branche à jour

```bash
# Récupère les changements upstream
git fetch upstream

# Rebase sur main (si il y a des conflits)
git rebase upstream/main

# Ou merge si préféré
git merge upstream/main
```

### 6. Push vers ton fork

```bash
git push origin fix/login-validation
```

---

## ✍️ Guidelines

### Code Style

#### Rust

```rust
// Format avec: cargo fmt
// Lint avec: cargo clippy

// Utilisez les Result types
fn risky_operation() -> Result<String, MyError> {
    Ok("success".to_string())
}

// Documentez les fonctions publiques
/// Valide un email au format RFC 5322.
///
/// # Arguments
/// * `email` - L'adresse email à valider
///
/// # Returns
/// `true` si valide, `false` sinon
pub fn is_valid_email(email: &str) -> bool {
    // implementation
}

// Utilisez les enums pour les alternatives
enum FuelType {
    Essence,
    Diesel,
    Electrique,
}
```

#### Python

```python
# Format avec: black src/
# Lint avec: flake8 src/

# Type hints obligatoires
def scrape_listings(
    brand: str, 
    model: str, 
    year_min: int, 
    year_max: int
) -> List[CarListing]:
    """Scrape car listings from all sources.
    
    Args:
        brand: Vehicle brand (e.g., 'Renault')
        model: Vehicle model (e.g., 'Clio')
        year_min: Minimum year
        year_max: Maximum year
        
    Returns:
        List of CarListing objects
    """
    # implementation
    pass

# Utilisez Pydantic pour la validation
from pydantic import BaseModel, Field

class CarListing(BaseModel):
    brand: str
    model: str
    year: int = Field(ge=1900, le=2024)
    mileage: int = Field(ge=0)
```

#### TypeScript/React

```typescript
// Format avec: eslint
// Type hints obligatoires
interface User {
  id: number;
  name: string;
  email: string;
  role: 'user' | 'admin';
}

// Composants avec types
interface LoginPageProps {
  onLogin: (token: string) => void;
}

const LoginPage: React.FC<LoginPageProps> = ({ onLogin }) => {
  return <div>...</div>;
};

// Fonctions avec return types
async function fetchUser(id: number): Promise<User> {
  const response = await fetch(`/api/users/${id}`);
  return response.json();
}
```

### Documentation

- **Docstrings** obligatoires pour les fonctions publiques
- **Comments** seulement si nécessaire (le code doit être self-documenting)
- **Type hints** toujours
- **Examples** pour les APIs publiques

### Tests

- Écrivez des tests pour les nouvelles features
- Minimum 70% coverage
- Tests unitaires + intégration

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_jwt_expired() {
        let expired_token = "eyJ..."; // Expired JWT
        let secret = "test_secret";
        
        let result = validate_jwt(expired_token, secret);
        
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().kind(), &ErrorKind::ExpiredSignature);
    }
}
```

---

## 📝 Processus PR

### 1. Ouvre une Pull Request

```bash
# Pousse ta branche
git push origin fix/login-validation

# Va sur GitHub et clique "New Pull Request"
```

### 2. Titre et description

**Titre:** Clair et concis

```
Fix: Validate JWT expiration correctly
Add: Price history endpoint
Docs: Update API documentation
```

**Description:** Complète et contextualisée

```markdown
## Description
Fixes the bug where expired JWT tokens were not being rejected.

## Changes
- Modified JWT validation function to check expiration time
- Added tests for expired token handling
- Updated error messages to be more descriptive

## Testing
- Added unit tests in `test_jwt_validation`
- Tested locally with expired tokens
- All existing tests pass

## Related Issues
Closes #123
```

### 3. Checklist avant submit

- [ ] Code compilé/testé localement
- [ ] Tests ajoutés et passants
- [ ] Linting passant (`cargo clippy`, `black`, `eslint`)
- [ ] Documentation mise à jour
- [ ] Commits avec messages clairs
- [ ] Pas de `node_modules`, `.env`, ou secrets en git
- [ ] Pas de conflit avec main

### 4. Code Review

Mainteneurs vont:
- Vérifier le code
- Tester localement
- Laisser des commentaires
- Demander des changements si nécessaire

**Comment répondre aux critiques:**

```
❌ "This function is too long and hard to understand"

✅ "I see your point. I'll split it into smaller functions:
    - parse_results()
    - validate_results()
    - save_results()
    
    This will make it clearer and easier to test."
```

### 5. Merge

Une fois approuvé:

```bash
# Update ta branche
git pull upstream main

# Rebase si conflits
git rebase upstream/main

# Push
git push origin fix/login-validation
```

Mainteneur clique "Merge" ✓

### 6. Cleanup

```bash
# Après merge, supprime ta branche
git checkout main
git pull upstream main
git branch -d fix/login-validation
git push origin --delete fix/login-validation
```

---

## 🔍 Détails spécifiques par service

### Rust Services

**Ajouter une dépendance:**

```toml
# Dans Cargo.toml
[dependencies]
serde = { version = "1.0", features = ["derive"] }
```

Puis:

```bash
cargo build
git add Cargo.toml Cargo.lock
```

### Python Services

**Ajouter une dépendance:**

```bash
pip install new-package
pip freeze > requirements.txt
git add requirements.txt
```

### Frontend

**Ajouter une dépendance:**

```bash
npm install new-package
git add package.json package-lock.json
```

---

## ❓ Questions?

- 📖 Lire la [documentation](../README.md)
- 📡 Consulter la [doc API](./API.md)
- 🏛️ Étudier la [doc architecture](./ARCHITECTURE.md)
- 💬 Ouvrir une discussion dans Issues

---

## 🎉 Merci!

Toute contribution, peu importe sa taille, est appréciée.

Ensemble, on construit une plateforme meilleure pour tous! 🚗✨

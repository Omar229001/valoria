import numpy as np
from sklearn.ensemble import GradientBoostingRegressor
from sklearn.preprocessing import LabelEncoder
import joblib
import os

# Mapping des caractéristiques catégorielles
BRANDS = ["renault", "peugeot", "citroen", "volkswagen", "toyota",
          "ford", "bmw", "mercedes", "audi", "opel", "autre"]
FUELS = ["essence", "diesel", "hybride", "electrique"]
TRANSMISSIONS = ["manuelle", "automatique"]
CONDITIONS = ["excellent", "bon", "moyen", "mauvais"]

def encode_feature(value: str, categories: list) -> int:
    value = value.lower()
    return categories.index(value) if value in categories else len(categories) - 1

def prepare_features(brand: str, model: str, year: int, mileage: int,
                     fuel: str, transmission: str, condition: str) -> np.ndarray:
    current_year = 2025
    age = current_year - year

    brand_enc = encode_feature(brand, BRANDS)
    fuel_enc = encode_feature(fuel, FUELS)
    trans_enc = encode_feature(transmission, TRANSMISSIONS)
    cond_enc = encode_feature(condition, CONDITIONS)

    return np.array([[brand_enc, age, mileage, fuel_enc, trans_enc, cond_enc]])

def generate_training_data(n_samples: int = 1000):
    np.random.seed(42)
    X, y = [], []

    for _ in range(n_samples):
        brand = np.random.randint(0, len(BRANDS))
        age = np.random.randint(0, 15)
        mileage = np.random.randint(5000, 200000)
        fuel = np.random.randint(0, len(FUELS))
        transmission = np.random.randint(0, len(TRANSMISSIONS))
        condition = np.random.randint(0, len(CONDITIONS))

        # Prix de base selon l'âge et le kilométrage
        base_price = 25000
        base_price -= age * 1200
        base_price -= mileage * 0.05
        base_price += (3 - condition) * (-1500)
        base_price += transmission * 1500
        base_price += np.random.normal(0, 800)
        base_price = max(500, base_price)

        X.append([brand, age, mileage, fuel, transmission, condition])
        y.append(base_price)

    return np.array(X), np.array(y)

def train_model():
    X, y = generate_training_data(2000)
    model = GradientBoostingRegressor(
        n_estimators=100,
        max_depth=4,
        learning_rate=0.1,
        random_state=42
    )
    model.fit(X, y)
    return model

# Modèle chargé au démarrage
_model = None

def get_model():
    global _model
    if _model is None:
        model_path = "/app/model.pkl"
        if os.path.exists(model_path):
            _model = joblib.load(model_path)
        else:
            _model = train_model()
            joblib.dump(_model, model_path)
    return _model

def predict_price(brand: str, model_name: str, year: int,
                  mileage: int, fuel: str, transmission: str,
                  condition: str) -> dict:
    features = prepare_features(brand, model_name, year,
                                mileage, fuel, transmission, condition)
    ml_model = get_model()
    price = float(ml_model.predict(features)[0])

    # Marge d'incertitude
    margin = price * 0.08
    confidence = max(0.6, 1.0 - (mileage / 300000) - ((2025 - year) / 30))

    return {
        "estimated_price": round(price, 2),
        "min_price": round(price - margin, 2),
        "max_price": round(price + margin, 2),
        "confidence": round(confidence, 2)
    }
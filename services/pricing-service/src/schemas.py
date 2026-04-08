from pydantic import BaseModel
from typing import Optional

class CarFeatures(BaseModel):
    brand: str           # Ex: "Renault"
    model: str           # Ex: "Clio"
    year: int            # Ex: 2019
    mileage: int         # Ex: 45000 (km)
    fuel: str            # Ex: "essence", "diesel", "hybride"
    transmission: str    # Ex: "manuelle", "automatique"
    condition: str       # Ex: "excellent", "bon", "moyen", "mauvais"
    city: str            # Ex: "Paris"

class PriceResponse(BaseModel):
    estimated_price: float
    min_price: float
    max_price: float
    confidence: float    # 0 à 1
    currency: str = "EUR"
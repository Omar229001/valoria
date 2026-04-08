from pydantic import BaseModel
from typing import Optional
from datetime import datetime


class CotationRequest(BaseModel):
    brand: str          # Ex: "Renault"
    model: str          # Ex: "Clio"
    year: int           # Ex: 2019
    mileage: int        # Ex: 45000 (km)
    fuel: str           # Ex: "essence", "diesel", "hybride"
    transmission: str   # Ex: "manuelle", "automatique"
    condition: str      # Ex: "excellent", "bon", "moyen", "mauvais"
    city: Optional[str] = None


class MarketStats(BaseModel):
    listings_count: int
    median_price: Optional[float] = None
    min_price: Optional[float] = None
    max_price: Optional[float] = None
    avg_price: Optional[float] = None


class CotationResponse(BaseModel):
    cotation_id: Optional[int] = None
    brand: str
    model: str
    year: int
    mileage: int
    fuel: str
    transmission: str
    condition: str
    # Prix du modèle ML
    ml_estimated_price: float
    # Données marché réelles
    market: MarketStats
    # Prix final recommandé (achat)
    estimated_price: float
    min_price: float
    max_price: float
    confidence: float       # 0 à 1
    currency: str = "EUR"
    method: str             # "ml_only", "market_only", "blend"
    created_at: Optional[datetime] = None

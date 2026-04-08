import os
import httpx
from fastapi import FastAPI, HTTPException, Query
from contextlib import asynccontextmanager
from typing import Optional, List
from src.schemas import CotationRequest, CotationResponse, MarketStats
from src.engine import compute_market_stats, blend_prices
from src.database import init_db, save_cotation, get_history

PRICING_SERVICE_URL = os.getenv("PRICING_SERVICE_URL", "http://pricing-service:8000")
SCRAPER_SERVICE_URL = os.getenv("SCRAPER_SERVICE_URL", "http://scraper-service:8000")


@asynccontextmanager
async def lifespan(app: FastAPI):
    init_db()
    yield


app = FastAPI(
    title="Valoria Cotation Service",
    description="Orchestre le scraping et le ML pour estimer le prix d'achat d'un véhicule",
    version="1.0.0",
    lifespan=lifespan
)


@app.get("/health")
def health():
    return {"status": "ok", "service": "cotation-service"}


@app.post("/cotation", response_model=CotationResponse)
def create_cotation(request: CotationRequest):
    """
    Calcule une cotation complète pour un véhicule :
    1. Récupère les annonces marché (scraper-service)
    2. Obtient l'estimation ML (pricing-service)
    3. Combine les deux pour un prix d'achat recommandé
    """

    # ── 1. Données marché depuis le scraper ──────────────────────────────────
    market_listings = []
    try:
        year_min = request.year - 2
        year_max = request.year + 1
        with httpx.Client(timeout=10.0) as client:
            resp = client.get(
                f"{SCRAPER_SERVICE_URL}/listings",
                params={
                    "brand": request.brand,
                    "model": request.model,
                    "year_min": year_min,
                    "year_max": year_max,
                }
            )
            if resp.status_code == 200:
                market_listings = resp.json()
                print(f"[Cotation] {len(market_listings)} annonces marché trouvées")
    except Exception as e:
        print(f"[Cotation] Impossible de joindre le scraper: {e}")

    market = compute_market_stats(market_listings)

    # ── 2. Estimation ML depuis le pricing-service ───────────────────────────
    ml_result = None
    try:
        with httpx.Client(timeout=10.0) as client:
            resp = client.post(
                f"{PRICING_SERVICE_URL}/predict",
                json={
                    "brand": request.brand,
                    "model": request.model,
                    "year": request.year,
                    "mileage": request.mileage,
                    "fuel": request.fuel,
                    "transmission": request.transmission,
                    "condition": request.condition,
                    "city": request.city or "Paris",
                }
            )
            if resp.status_code == 200:
                ml_result = resp.json()
                print(f"[Cotation] ML price: {ml_result['estimated_price']}€ (confidence: {ml_result['confidence']})")
    except Exception as e:
        print(f"[Cotation] Impossible de joindre le pricing-service: {e}")

    if ml_result is None:
        raise HTTPException(
            status_code=503,
            detail="Le pricing-service est indisponible. Relance docker compose up -d."
        )

    # ── 3. Calcul du prix final (blend ML + marché) ──────────────────────────
    blend = blend_prices(
        ml_price=ml_result["estimated_price"],
        market=market,
        condition=request.condition,
        ml_confidence=ml_result["confidence"],
    )

    # ── 4. Sauvegarde en base ────────────────────────────────────────────────
    cotation_data = {
        "brand": request.brand,
        "model": request.model,
        "year": request.year,
        "mileage": request.mileage,
        "fuel": request.fuel,
        "transmission": request.transmission,
        "condition": request.condition,
        "city": request.city,
        "ml_estimated_price": ml_result["estimated_price"],
        "market_listings_count": market.listings_count,
        "market_median_price": market.median_price,
        "market_min_price": market.min_price,
        "market_max_price": market.max_price,
        **blend,
    }

    cotation_id, created_at = save_cotation(cotation_data)

    # ── 5. Réponse ────────────────────────────────────────────────────────────
    return CotationResponse(
        cotation_id=cotation_id,
        brand=request.brand,
        model=request.model,
        year=request.year,
        mileage=request.mileage,
        fuel=request.fuel,
        transmission=request.transmission,
        condition=request.condition,
        ml_estimated_price=ml_result["estimated_price"],
        market=market,
        created_at=created_at,
        **blend,
    )


@app.get("/cotation/history")
def cotation_history(
    brand: Optional[str] = Query(None),
    model: Optional[str] = Query(None),
    limit: int = Query(20, ge=1, le=100),
):
    """Retourne l'historique des cotations."""
    return get_history(brand=brand, model=model, limit=limit)


@app.get("/")
def root():
    return {
        "service": "cotation-service",
        "version": "1.0.0",
        "endpoints": ["/health", "/cotation", "/cotation/history"]
    }

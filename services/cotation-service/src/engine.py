import statistics
from typing import List, Optional
from src.schemas import MarketStats

# Coefficients de condition appliqués sur le prix marché
CONDITION_COEFF = {
    "excellent": 1.10,
    "bon":       1.00,
    "moyen":     0.88,
    "mauvais":   0.75,
}


def compute_market_stats(listings: List[dict]) -> MarketStats:
    """Calcule les statistiques de marché à partir des annonces scrapers."""
    prices = [l["price"] for l in listings if l.get("price") and l["price"] > 0]

    if not prices:
        return MarketStats(listings_count=0)

    return MarketStats(
        listings_count=len(prices),
        median_price=round(statistics.median(prices), 0),
        min_price=round(min(prices), 0),
        max_price=round(max(prices), 0),
        avg_price=round(statistics.mean(prices), 0),
    )


def blend_prices(
    ml_price: float,
    market: MarketStats,
    condition: str,
    ml_confidence: float,
) -> dict:
    """
    Combine le prix ML et le prix marché pour produire le prix final recommandé.

    Stratégie de pondération :
      - 0 annonce   → ML pur (confiance réduite)
      - 1-4 annonces → ML 70% + marché 30%
      - 5-9 annonces → ML 40% + marché 60%
      - 10+ annonces → ML 20% + marché 80%
    """
    condition_coeff = CONDITION_COEFF.get(condition.lower(), 1.0)
    n = market.listings_count

    if n == 0:
        # Pas de données marché : on se base uniquement sur le ML
        final_price = ml_price * condition_coeff
        confidence = ml_confidence * 0.6  # On pénalise l'absence de données réelles
        method = "ml_only"
        spread = 0.15  # ±15%

    elif n < 5:
        # Peu de données : ML dominant
        market_price = market.median_price * condition_coeff
        final_price = 0.70 * ml_price + 0.30 * market_price
        confidence = ml_confidence * 0.75
        method = "blend"
        spread = 0.12

    elif n < 10:
        # Données suffisantes : marché dominant
        market_price = market.median_price * condition_coeff
        final_price = 0.40 * ml_price + 0.60 * market_price
        confidence = ml_confidence * 0.85 + 0.05  # Bonus données réelles
        method = "blend"
        spread = 0.10

    else:
        # Nombreuses données : marché très dominant
        market_price = market.median_price * condition_coeff
        final_price = 0.20 * ml_price + 0.80 * market_price
        confidence = min(0.95, ml_confidence * 0.90 + 0.10)
        method = "blend"
        spread = 0.08

    # Clamp confidence entre 0 et 1
    confidence = max(0.1, min(1.0, confidence))

    final_price = round(final_price, 0)
    min_price = round(final_price * (1 - spread), 0)
    max_price = round(final_price * (1 + spread), 0)

    return {
        "estimated_price": final_price,
        "min_price": min_price,
        "max_price": max_price,
        "confidence": round(confidence, 3),
        "method": method,
    }

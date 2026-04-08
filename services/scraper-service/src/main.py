from fastapi import FastAPI, BackgroundTasks
from contextlib import asynccontextmanager
from src.schemas import ScrapeRequest, ScrapeResponse, CarListing
from src.database import init_db, save_listings, get_listings
from src.scrapers.lacentrale import LaCentraleScraper
from src.scrapers.leboncoin import LeBonCoinScraper
from src.scrapers.autoscout24 import AutoScout24Scraper
from typing import List

SCRAPERS = {
    "lacentrale": LaCentraleScraper,
    "leboncoin": LeBonCoinScraper,
    "autoscout24": AutoScout24Scraper,
}

@asynccontextmanager
async def lifespan(app: FastAPI):
    init_db()
    yield

app = FastAPI(
    title="Valoria Scraper Service",
    description="Service de collecte de prix de véhicules d'occasion",
    version="1.0.0",
    lifespan=lifespan
)

@app.get("/health")
def health():
    return {"status": "ok", "service": "scraper-service"}

@app.post("/scrape", response_model=List[ScrapeResponse])
def scrape(request: ScrapeRequest, background_tasks: BackgroundTasks):
    """Lance le scraping sur tous les sites en arrière-plan"""
    for source in SCRAPERS:
        background_tasks.add_task(
            run_scrape, source, request.brand,
            request.model, request.year_min, request.year_max
        )
    return [ScrapeResponse(listings_found=0, listings_saved=0, source=s) for s in SCRAPERS]

@app.post("/scrape/sync", response_model=List[ScrapeResponse])
def scrape_sync(request: ScrapeRequest, sources: str = "all"):
    """Lance le scraping de façon synchrone sur tous les sites"""
    results = []
    targets = list(SCRAPERS.keys()) if sources == "all" else sources.split(",")

    for source in targets:
        if source not in SCRAPERS:
            continue
        scraper_class = SCRAPERS[source]
        with scraper_class() as scraper:
            try:
                listings = scraper.search(
                    request.brand, request.model,
                    request.year_min, request.year_max
                )
                saved = save_listings([l.model_dump() for l in listings])
                results.append(ScrapeResponse(
                    listings_found=len(listings),
                    listings_saved=saved,
                    source=source
                ))
                print(f"[{source}] {len(listings)} annonces trouvées, {saved} sauvegardées")
            except Exception as e:
                print(f"[{source}] Erreur: {e}")
                results.append(ScrapeResponse(listings_found=0, listings_saved=0, source=source))

    return results

@app.get("/listings", response_model=List[CarListing])
def get_car_listings(brand: str, model: str, year_min: int, year_max: int):
    """Récupère les annonces depuis la base de données"""
    return get_listings(brand, model, year_min, year_max)

@app.get("/debug/autoscout24")
def debug_autoscout24():
    """Inspecte la structure HTML réelle d'AutoScout24"""
    from playwright.sync_api import sync_playwright
    with sync_playwright() as p:
        browser = p.chromium.launch(headless=True, args=["--no-sandbox", "--disable-dev-shm-usage"])
        context = browser.new_context(
            user_agent="Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36",
            locale="fr-FR"
        )
        page = context.new_page()
        page.goto(
            "https://www.autoscout24.fr/lst/renault/clio?atype=C&cy=F&fregfrom=2019&fregto=2022",
            wait_until="domcontentloaded", timeout=30000
        )
        page.wait_for_timeout(3000)

        # Accepter cookies
        try:
            page.click("button[data-testid='as24-cmp-accept-all-button']", timeout=3000)
            page.wait_for_timeout(2000)
        except Exception:
            pass

        # Inspecter les éléments disponibles
        structure = page.evaluate("""
            () => {
                const articles = document.querySelectorAll('article.cldt-summary-full-item');
                const first = articles[0];
                if (!first) return { error: "No articles found" };
                return {
                    total_articles: articles.length,
                    full_html: first.innerHTML.substring(0, 3000),
                    full_text: first.innerText
                };
            }
        """)
        browser.close()
        return structure

def run_scrape(source: str, brand: str, model: str, year_min: int, year_max: int):
    scraper_class = SCRAPERS[source]
    with scraper_class() as scraper:
        try:
            listings = scraper.search(brand, model, year_min, year_max)
            save_listings([l.model_dump() for l in listings])
        except Exception as e:
            print(f"[{source}] Erreur background: {e}")

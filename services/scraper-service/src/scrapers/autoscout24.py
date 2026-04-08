import re
from typing import List, Optional
from datetime import datetime
from src.scrapers.base import BaseScraper
from src.schemas import CarListing

class AutoScout24Scraper(BaseScraper):

    BASE_URL = "https://www.autoscout24.fr/lst"

    def search(self, brand: str, model: str, year_min: int, year_max: int) -> List[CarListing]:
        listings = []
        url = (
            f"{self.BASE_URL}/{brand.lower()}/{model.lower()}"
            f"?atype=C&cy=F&damaged=X"
            f"&fregfrom={year_min}&fregto={year_max}&ustate=N%2CU&size=20"
        )
        try:
            self._page.goto(url, wait_until="domcontentloaded", timeout=30000)
            self._page.wait_for_timeout(3000)

            # Accepter les cookies
            try:
                self._page.click("button[data-testid='as24-cmp-accept-all-button']", timeout=3000)
                self._page.wait_for_timeout(2000)
            except Exception:
                pass

            for page_num in range(1, 4):
                try:
                    self._page.wait_for_selector("article.cldt-summary-full-item", timeout=10000)
                except Exception:
                    break

                cards_data = self._page.evaluate("""
                    () => {
                        const articles = document.querySelectorAll('article.cldt-summary-full-item');
                        return Array.from(articles).map(card => {
                            // Prix via data-testid
                            const priceEl = card.querySelector('[data-testid="regular-price"]');
                            const price = priceEl ? priceEl.innerText : null;

                            // URL de l'annonce
                            const linkEl = card.querySelector('a[href*="/offres/"]');
                            const url = linkEl ? linkEl.href : null;

                            // Texte complet de la carte pour extraire les détails
                            const text = card.innerText;

                            return { price, url, text };
                        });
                    }
                """)

                for card_data in cards_data:
                    listing = self._parse_card_data(card_data, brand, model)
                    if listing:
                        listings.append(listing)

                # Page suivante
                try:
                    next_btn = self._page.locator("a[aria-label='Aller à la page suivante']")
                    if next_btn.count() > 0:
                        next_btn.first.click()
                        self._page.wait_for_timeout(3000)
                        self._wait()
                    else:
                        break
                except Exception:
                    break

        except Exception as e:
            print(f"[AutoScout24] Erreur: {e}")

        return listings

    def _parse_card_data(self, data: dict, brand: str, model: str) -> Optional[CarListing]:
        try:
            text = data.get("text") or ""

            price = self._extract_price(data.get("price") or "")
            year = self._extract_year(text)
            mileage = self._extract_mileage(text)
            fuel = self._extract_fuel(text)
            transmission = self._extract_transmission(text)
            city = self._extract_city(text)
            url = data.get("url")

            # Validation stricte
            if not price or price < 500 or price > 150000:
                return None
            if not year or year < 2000 or year > 2026:
                return None
            if not mileage or mileage < 100 or mileage > 500000:
                return None

            return CarListing(
                brand=brand.capitalize(),
                model=model.capitalize(),
                year=year,
                mileage=mileage,
                fuel=fuel or "essence",
                transmission=transmission or "manuelle",
                price=price,
                city=city,
                source="autoscout24",
                url=url,
                scraped_at=datetime.now()
            )
        except Exception as e:
            print(f"[AutoScout24] Erreur parsing: {e}")
            return None

    def _extract_price(self, text: str) -> Optional[float]:
        if not text:
            return None
        # Supprime €, espaces insécables (\u202f), espaces normaux
        cleaned = re.sub(r"[€\s\u202f\xa0]", "", text)
        cleaned = re.sub(r"[^\d]", "", cleaned)
        if cleaned and 3 <= len(cleaned) <= 6:
            return float(cleaned)
        return None

    def _extract_year(self, text: str) -> Optional[int]:
        # Format "10/2020" ou "01/2019"
        match = re.search(r"\b(0[1-9]|1[0-2])/(20[0-9]{2})\b", text)
        if match:
            return int(match.group(2))
        return None

    def _extract_mileage(self, text: str) -> Optional[int]:
        # Format "104 000 km" ou "104\u202f000 km"
        # On cherche un nombre suivi de km, en excluant les années
        matches = re.finditer(r"([\d][\d\s\u202f\xa0]*)\s*km", text, re.I)
        for match in matches:
            cleaned = re.sub(r"[\s\u202f\xa0]", "", match.group(1))
            val = int(cleaned)
            # Exclure les années (1900-2100) et valeurs aberrantes
            if 500 <= val <= 400000:
                return val
        return None

    def _extract_fuel(self, text: str) -> Optional[str]:
        text_lower = text.lower()
        for fuel in ["diesel", "hybride", "électrique", "electrique", "essence", "gpl"]:
            if fuel in text_lower:
                return fuel
        return None

    def _extract_transmission(self, text: str) -> Optional[str]:
        text_lower = text.lower()
        if "automatique" in text_lower or "automatic" in text_lower:
            return "automatique"
        return "manuelle"

    def _extract_city(self, text: str) -> Optional[str]:
        # Format "FR-62136 LESTREM" → on extrait la ville
        match = re.search(r"FR-\d{5}\s+([A-Z\s\-]+)", text)
        if match:
            return match.group(1).strip().capitalize()
        return None

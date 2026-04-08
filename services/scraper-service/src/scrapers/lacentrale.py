import re
from typing import List, Optional
from datetime import datetime
from bs4 import BeautifulSoup
from src.scrapers.base import BaseScraper
from src.schemas import CarListing

class LaCentraleScraper(BaseScraper):

    BASE_URL = "https://www.lacentrale.fr/listing"

    def search(self, brand: str, model: str, year_min: int, year_max: int) -> List[CarListing]:
        listings = []
        for page in range(1, 4):
            url = (
                f"{self.BASE_URL}?"
                f"makesModelsCommercialNames={brand.upper()}%3A{model.upper()}"
                f"&yearMin={year_min}&yearMax={year_max}&page={page}"
            )
            try:
                self._page.goto(url, wait_until="domcontentloaded", timeout=30000)
                self._page.wait_for_timeout(3000)

                # Fermer les popups si présents
                try:
                    self._page.click("button[aria-label*='fermer']", timeout=3000)
                except Exception:
                    pass
                try:
                    self._page.click("#didomi-notice-agree-button", timeout=3000)
                except Exception:
                    pass

                soup = BeautifulSoup(self._page.content(), "html.parser")
                cards = soup.find_all("div", class_=re.compile(r"searchCard|SearchCard|listing"))

                if not cards:
                    break

                for card in cards:
                    listing = self._parse_card(card, brand, model)
                    if listing:
                        listings.append(listing)

                self._wait()

            except Exception as e:
                print(f"[LaCentrale] Erreur page {page}: {e}")
                break

        return listings

    def _parse_card(self, card, brand: str, model: str) -> Optional[CarListing]:
        try:
            text = card.get_text(" ", strip=True)

            price = self._extract_price(text)
            year = self._extract_year(text)
            mileage = self._extract_mileage(text)
            fuel = self._extract_fuel(text)
            transmission = self._extract_transmission(text)
            city = self._extract_city(card)
            url = self._extract_url(card)

            if not all([price, year, mileage]):
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
                source="lacentrale",
                url=url,
                scraped_at=datetime.now()
            )
        except Exception:
            return None

    def _extract_price(self, text: str) -> Optional[float]:
        match = re.search(r"(\d[\d\s]{2,})\s*€", text)
        if match:
            return float(re.sub(r"\s", "", match.group(1)))
        return None

    def _extract_year(self, text: str) -> Optional[int]:
        match = re.search(r"\b(201[0-9]|202[0-9])\b", text)
        return int(match.group()) if match else None

    def _extract_mileage(self, text: str) -> Optional[int]:
        match = re.search(r"(\d[\d\s]+)\s*km", text, re.I)
        if match:
            return int(re.sub(r"\s", "", match.group(1)))
        return None

    def _extract_fuel(self, text: str) -> Optional[str]:
        text = text.lower()
        for fuel in ["diesel", "essence", "hybride", "électrique", "electrique"]:
            if fuel in text:
                return fuel
        return None

    def _extract_transmission(self, text: str) -> Optional[str]:
        text = text.lower()
        if "automatique" in text:
            return "automatique"
        if "manuelle" in text:
            return "manuelle"
        return None

    def _extract_city(self, card) -> Optional[str]:
        el = card.find(class_=re.compile(r"city|ville|location|departement", re.I))
        return el.get_text(strip=True) if el else None

    def _extract_url(self, card) -> Optional[str]:
        link = card.find("a", href=True)
        if link:
            href = link["href"]
            return f"https://www.lacentrale.fr{href}" if href.startswith("/") else href
        return None

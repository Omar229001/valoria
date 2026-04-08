import re
from typing import List, Optional
from datetime import datetime
from bs4 import BeautifulSoup
from src.scrapers.base import BaseScraper
from src.schemas import CarListing

class LeBonCoinScraper(BaseScraper):

    BASE_URL = "https://www.leboncoin.fr/recherche"

    def search(self, brand: str, model: str, year_min: int, year_max: int) -> List[CarListing]:
        listings = []
        url = (
            f"{self.BASE_URL}?category=2"
            f"&brand={brand.capitalize()}"
            f"&model={model.capitalize()}"
            f"&regdate={year_min}-{year_max}"
        )
        try:
            self._page.goto(url, wait_until="domcontentloaded", timeout=30000)
            self._page.wait_for_timeout(4000)

            # Accepter les cookies
            try:
                self._page.click("button[data-testid='didomi-notice-agree-button']", timeout=3000)
                self._page.wait_for_timeout(2000)
            except Exception:
                pass

            for page_num in range(1, 4):
                soup = BeautifulSoup(self._page.content(), "html.parser")
                cards = soup.find_all("li", attrs={"data-test-id": re.compile(r"aditem|ad-item")})

                if not cards:
                    cards = soup.find_all("article")

                for card in cards:
                    listing = self._parse_card(card, brand, model)
                    if listing:
                        listings.append(listing)

                # Page suivante
                try:
                    next_btn = self._page.locator("a[aria-label*='suivant'], a[data-testid*='next']")
                    if next_btn.count() > 0:
                        next_btn.first.click()
                        self._page.wait_for_timeout(3000)
                        self._wait()
                    else:
                        break
                except Exception:
                    break

        except Exception as e:
            print(f"[LeBonCoin] Erreur: {e}")

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
                source="leboncoin",
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
        return "manuelle"

    def _extract_city(self, card) -> Optional[str]:
        el = card.find(attrs={"data-test-id": re.compile(r"aditem_location|location")})
        if el:
            return el.get_text(strip=True)
        el = card.find(class_=re.compile(r"location|ville|city", re.I))
        return el.get_text(strip=True) if el else None

    def _extract_url(self, card) -> Optional[str]:
        link = card.find("a", href=True)
        if link:
            href = link["href"]
            return f"https://www.leboncoin.fr{href}" if href.startswith("/") else href
        return None

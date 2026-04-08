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
            self._page.wait_for_timeout(4000)

            # Accepter les cookies
            try:
                self._page.click("button[data-testid='as24-cmp-accept-all-button']", timeout=4000)
                self._page.wait_for_timeout(2000)
            except Exception:
                pass

            for page_num in range(1, 4):
                try:
                    self._page.wait_for_selector("article.cldt-summary-full-item", timeout=10000)
                except Exception:
                    print(f"[AutoScout24] Aucun article trouvé à la page {page_num}")
                    break

                # Attendre que tous les articles soient chargés
                self._page.wait_for_timeout(2000)

                cards_data = self._page.evaluate("""
                    () => {
                        const articles = document.querySelectorAll('article.cldt-summary-full-item');
                        return Array.from(articles).map(card => {
                            // Essayer plusieurs sélecteurs de prix
                            const priceSelectors = [
                                '[data-testid="regular-price"]',
                                '[data-testid="dealer-price"]',
                                '.cldt-price-section',
                                '.cldt-price',
                                'strong.cldt-price-block-price',
                                '[class*="price"]'
                            ];
                            let priceEl = null;
                            for (const sel of priceSelectors) {
                                priceEl = card.querySelector(sel);
                                if (priceEl) break;
                            }
                            const price = priceEl ? priceEl.innerText.trim() : null;

                            // URL de l'annonce : cherche uniquement les liens /annonces/
                            // On NE fait PAS de fallback générique pour éviter les doublons
                            const linkEl = card.querySelector('a[href*="/annonces/"], a[href*="/offres/"]');
                            let url = linkEl ? linkEl.href : null;

                            // Si pas trouvé, essayer via l'id ou data-guid de l'article
                            if (!url) {
                                const guid = card.getAttribute('data-guid') || card.id;
                                if (guid) url = 'https://www.autoscout24.fr/annonces/' + guid;
                            }

                            // Texte complet de la carte
                            const text = card.innerText;

                            // Debug : logguer les 3 premiers liens trouvés dans la carte
                            const allLinks = Array.from(card.querySelectorAll('a[href]'))
                                .map(a => a.getAttribute('href'))
                                .filter(h => h && !h.startsWith('#') && !h.startsWith('javascript'))
                                .slice(0, 3);

                            return { price, url, text, allLinks };
                        });
                    }
                """)

                print(f"[AutoScout24] Page {page_num}: {len(cards_data)} cartes trouvées")

                # Debug : afficher les URLs des 3 premières cartes
                for dbg in cards_data[:3]:
                    print(f"[AutoScout24] DEBUG url='{dbg.get('url')}' | links={dbg.get('allLinks')}")

                for i, card_data in enumerate(cards_data):
                    listing = self._parse_card_data(card_data, brand, model, i)
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

    def _parse_card_data(self, data: dict, brand: str, model: str, idx: int = 0) -> Optional[CarListing]:
        try:
            raw_text = data.get("text") or ""
            # Normaliser : remplacer tous les whitespace (y compris \u202f, \xa0) par des espaces simples
            text = re.sub(r'[\s\u202f\xa0]+', ' ', raw_text).strip()

            raw_price = data.get("price") or ""
            price = self._extract_price(raw_price)
            year = self._extract_year(text)
            mileage = self._extract_mileage(text)
            fuel = self._extract_fuel(text)
            transmission = self._extract_transmission(text)
            city = self._extract_city(text)
            url = data.get("url")

            # Log de debug pour comprendre les échecs
            if not price or price < 500 or price > 150000:
                print(f"[AutoScout24] #{idx} Prix invalide: raw='{raw_price}' → {price} | extrait: {text[:80]}")
                return None
            if not year or year < 2000 or year > 2026:
                print(f"[AutoScout24] #{idx} Année invalide: {year} | extrait: {text[:80]}")
                return None
            if not mileage or mileage < 50 or mileage > 500000:
                print(f"[AutoScout24] #{idx} Kilométrage invalide: {mileage} | extrait: {text[:80]}")
                return None

            print(f"[AutoScout24] #{idx} OK → {year}, {mileage}km, {price}€, {fuel}, {city}")
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
            print(f"[AutoScout24] Erreur parsing #{idx}: {e}")
            return None

    def _extract_price(self, text: str) -> Optional[float]:
        if not text:
            return None
        # Supprime tout sauf les chiffres
        cleaned = re.sub(r"[^\d]", "", text)
        if cleaned and 3 <= len(cleaned) <= 6:
            return float(cleaned)
        return None

    def _extract_year(self, text: str) -> Optional[int]:
        # Format "10/2020" ou "01/2019" (MM/YYYY)
        match = re.search(r"\b(0[1-9]|1[0-2])/(20[0-9]{2})\b", text)
        if match:
            return int(match.group(2))
        # Fallback: juste une année seule (ex: "2020")
        match = re.search(r"\b(20[12][0-9])\b", text)
        if match:
            return int(match.group(1))
        return None

    def _extract_mileage(self, text: str) -> Optional[int]:
        """
        Cherche des patterns comme "82 229 km" ou "104000 km" ou "5 200 km"
        Après normalisation, les séparateurs de milliers sont des espaces simples.
        """
        # Pattern: 1-3 chiffres, optionnellement suivi de groupes de (espace + 3 chiffres), puis km
        matches = re.finditer(r"\b(\d{1,3}(?:\s\d{3})*)\s*km\b", text, re.I)
        for match in matches:
            cleaned = re.sub(r"\s", "", match.group(1))
            if cleaned.isdigit():
                val = int(cleaned)
                if 50 <= val <= 500000:
                    return val
        # Fallback: nombre brut avant km (ex: "104000 km")
        matches2 = re.finditer(r"\b(\d{4,6})\s*km\b", text, re.I)
        for match in matches2:
            val = int(match.group(1))
            # Exclure les années (1900-2100)
            if 50 <= val <= 500000 and not (1900 <= val <= 2100):
                return val
        return None

    def _extract_fuel(self, text: str) -> Optional[str]:
        text_lower = text.lower()
        for fuel in ["diesel", "hybride", "électrique", "electrique", "essence", "gpl", "mild hybrid", "full hybrid"]:
            if fuel in text_lower:
                return fuel
        return None

    def _extract_transmission(self, text: str) -> Optional[str]:
        text_lower = text.lower()
        if "automatique" in text_lower or "automatic" in text_lower:
            return "automatique"
        return "manuelle"

    def _extract_city(self, text: str) -> Optional[str]:
        # Format "FR-62136 LESTREM"
        match = re.search(r"FR-\d{5}\s+([A-Z][A-Z\s\-]+?)(?:\s|$)", text)
        if match:
            return match.group(1).strip().capitalize()
        return None

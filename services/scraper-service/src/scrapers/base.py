import time
import random
from abc import ABC, abstractmethod
from typing import List
from playwright.sync_api import sync_playwright
from src.schemas import CarListing

class BaseScraper(ABC):

    def __init__(self):
        self._playwright = None
        self._browser = None
        self._page = None

    def __enter__(self):
        self._playwright = sync_playwright().start()
        self._browser = self._playwright.chromium.launch(
            headless=True,
            args=[
                "--no-sandbox",
                "--disable-setuid-sandbox",
                "--disable-dev-shm-usage",
                "--disable-blink-features=AutomationControlled"
            ]
        )
        context = self._browser.new_context(
            viewport={"width": 1280, "height": 800},
            user_agent="Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) "
                       "AppleWebKit/537.36 (KHTML, like Gecko) "
                       "Chrome/120.0.0.0 Safari/537.36",
            locale="fr-FR"
        )
        self._page = context.new_page()
        return self

    def __exit__(self, *args):
        if self._browser:
            self._browser.close()
        if self._playwright:
            self._playwright.stop()

    def _wait(self, min_sec=1.5, max_sec=4.0):
        time.sleep(random.uniform(min_sec, max_sec))

    def _safe_text(self, element) -> str:
        try:
            return element.inner_text().strip()
        except Exception:
            return ""

    @abstractmethod
    def search(self, brand: str, model: str, year_min: int, year_max: int) -> List[CarListing]:
        pass

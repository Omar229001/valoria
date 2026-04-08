from pydantic import BaseModel
from typing import Optional
from datetime import datetime

class CarListing(BaseModel):
    brand: str
    model: str
    year: int
    mileage: int
    fuel: str
    transmission: str
    price: float
    city: Optional[str] = None
    source: str
    url: Optional[str] = None
    scraped_at: Optional[datetime] = None

class ScrapeRequest(BaseModel):
    brand: str
    model: str
    year_min: int
    year_max: int

class ScrapeResponse(BaseModel):
    listings_found: int
    listings_saved: int
    source: str

import os
import psycopg2
from psycopg2.extras import RealDictCursor

DATABASE_URL = os.getenv("DATABASE_URL", "postgresql://valoria:valoria_dev@localhost:5434/valoria")

def get_connection():
    return psycopg2.connect(DATABASE_URL, cursor_factory=RealDictCursor)

def init_db():
    conn = get_connection()
    cursor = conn.cursor()
    cursor.execute("""
        CREATE TABLE IF NOT EXISTS car_listings (
            id SERIAL PRIMARY KEY,
            brand VARCHAR(100),
            model VARCHAR(100),
            year INTEGER,
            mileage INTEGER,
            fuel VARCHAR(50),
            transmission VARCHAR(50),
            price FLOAT,
            city VARCHAR(100),
            source VARCHAR(100),
            url TEXT,
            scraped_at TIMESTAMP DEFAULT NOW(),
            UNIQUE(url)
        )
    """)
    conn.commit()
    cursor.close()
    conn.close()

def save_listings(listings: list) -> int:
    conn = get_connection()
    cursor = conn.cursor()
    saved = 0
    for listing in listings:
        try:
            cursor.execute("""
                INSERT INTO car_listings 
                (brand, model, year, mileage, fuel, transmission, price, city, source, url)
                VALUES (%(brand)s, %(model)s, %(year)s, %(mileage)s, %(fuel)s,
                        %(transmission)s, %(price)s, %(city)s, %(source)s, %(url)s)
                ON CONFLICT (url) DO NOTHING
            """, listing)
            saved += cursor.rowcount
        except Exception:
            continue
    conn.commit()
    cursor.close()
    conn.close()
    return saved

def get_listings(brand: str, model: str, year_min: int, year_max: int) -> list:
    conn = get_connection()
    cursor = conn.cursor()
    cursor.execute("""
        SELECT * FROM car_listings
        WHERE LOWER(brand) = LOWER(%s)
        AND LOWER(model) LIKE LOWER(%s)
        AND year BETWEEN %s AND %s
        ORDER BY scraped_at DESC
    """, (brand, f"%{model}%", year_min, year_max))
    results = cursor.fetchall()
    cursor.close()
    conn.close()
    return [dict(r) for r in results]

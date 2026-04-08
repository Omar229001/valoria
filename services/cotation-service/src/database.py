import os
import psycopg2
from psycopg2.extras import RealDictCursor
from datetime import datetime

DATABASE_URL = os.getenv("DATABASE_URL", "postgresql://valoria:valoria_dev@localhost:5434/valoria")


def get_connection():
    return psycopg2.connect(DATABASE_URL, cursor_factory=RealDictCursor)


def init_db():
    conn = get_connection()
    cursor = conn.cursor()
    cursor.execute("""
        CREATE TABLE IF NOT EXISTS cotations (
            id SERIAL PRIMARY KEY,
            brand VARCHAR(100) NOT NULL,
            model VARCHAR(100) NOT NULL,
            year INTEGER NOT NULL,
            mileage INTEGER NOT NULL,
            fuel VARCHAR(50),
            transmission VARCHAR(50),
            condition VARCHAR(50),
            city VARCHAR(100),
            ml_estimated_price FLOAT,
            market_listings_count INTEGER DEFAULT 0,
            market_median_price FLOAT,
            market_min_price FLOAT,
            market_max_price FLOAT,
            estimated_price FLOAT NOT NULL,
            min_price FLOAT NOT NULL,
            max_price FLOAT NOT NULL,
            confidence FLOAT NOT NULL,
            method VARCHAR(50),
            created_at TIMESTAMP DEFAULT NOW()
        )
    """)
    conn.commit()
    cursor.close()
    conn.close()


def save_cotation(data: dict) -> int:
    conn = get_connection()
    cursor = conn.cursor()
    cursor.execute("""
        INSERT INTO cotations (
            brand, model, year, mileage, fuel, transmission, condition, city,
            ml_estimated_price, market_listings_count,
            market_median_price, market_min_price, market_max_price,
            estimated_price, min_price, max_price, confidence, method
        ) VALUES (
            %(brand)s, %(model)s, %(year)s, %(mileage)s, %(fuel)s,
            %(transmission)s, %(condition)s, %(city)s,
            %(ml_estimated_price)s, %(market_listings_count)s,
            %(market_median_price)s, %(market_min_price)s, %(market_max_price)s,
            %(estimated_price)s, %(min_price)s, %(max_price)s,
            %(confidence)s, %(method)s
        )
        RETURNING id, created_at
    """, data)
    row = cursor.fetchone()
    conn.commit()
    cursor.close()
    conn.close()
    return row["id"], row["created_at"]


def get_history(brand: str = None, model: str = None, limit: int = 20) -> list:
    conn = get_connection()
    cursor = conn.cursor()

    if brand and model:
        cursor.execute("""
            SELECT * FROM cotations
            WHERE LOWER(brand) = LOWER(%s) AND LOWER(model) LIKE LOWER(%s)
            ORDER BY created_at DESC LIMIT %s
        """, (brand, f"%{model}%", limit))
    elif brand:
        cursor.execute("""
            SELECT * FROM cotations
            WHERE LOWER(brand) = LOWER(%s)
            ORDER BY created_at DESC LIMIT %s
        """, (brand, limit))
    else:
        cursor.execute("""
            SELECT * FROM cotations
            ORDER BY created_at DESC LIMIT %s
        """, (limit,))

    results = cursor.fetchall()
    cursor.close()
    conn.close()
    return [dict(r) for r in results]

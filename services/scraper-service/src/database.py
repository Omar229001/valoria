import os
import psycopg2
from psycopg2.extras import RealDictCursor

DATABASE_URL = os.getenv("DATABASE_URL", "postgresql://valoria:valoria_dev@localhost:5434/valoria")

def get_connection():
    return psycopg2.connect(DATABASE_URL, cursor_factory=RealDictCursor)

def init_db():
    conn = get_connection()
    cursor = conn.cursor()

    # Supprimer l'ancienne contrainte UNIQUE sur url si elle existe (migration)
    cursor.execute("""
        DO $$
        BEGIN
            IF EXISTS (
                SELECT 1 FROM information_schema.table_constraints
                WHERE table_name = 'car_listings'
                  AND constraint_type = 'UNIQUE'
                  AND constraint_name = 'car_listings_url_key'
            ) THEN
                ALTER TABLE car_listings DROP CONSTRAINT car_listings_url_key;
            END IF;
        END
        $$;
    """)

    # Créer la table sans contrainte UNIQUE globale sur url
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
            scraped_at TIMESTAMP DEFAULT NOW()
        )
    """)

    # Index partiel : unicité sur url SEULEMENT quand url n'est pas null
    cursor.execute("""
        CREATE UNIQUE INDEX IF NOT EXISTS car_listings_url_unique
        ON car_listings (url)
        WHERE url IS NOT NULL
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
            url = listing.get("url")

            if url:
                # URL présente → déduplique par URL (index partiel)
                cursor.execute("""
                    INSERT INTO car_listings
                    (brand, model, year, mileage, fuel, transmission, price, city, source, url)
                    VALUES (%(brand)s, %(model)s, %(year)s, %(mileage)s, %(fuel)s,
                            %(transmission)s, %(price)s, %(city)s, %(source)s, %(url)s)
                    ON CONFLICT (url) WHERE url IS NOT NULL DO NOTHING
                """, listing)
                if cursor.rowcount > 0:
                    saved += 1
            else:
                # Pas d'URL → vérification manuelle avant insert
                cursor.execute("""
                    SELECT COUNT(*) as cnt FROM car_listings
                    WHERE brand = %(brand)s
                      AND model = %(model)s
                      AND year = %(year)s
                      AND mileage = %(mileage)s
                      AND ROUND(price::numeric, 0) = ROUND(%(price)s::numeric, 0)
                      AND source = %(source)s
                """, listing)
                row = cursor.fetchone()
                count = row["cnt"] if row else 0
                if count == 0:
                    cursor.execute("""
                        INSERT INTO car_listings
                        (brand, model, year, mileage, fuel, transmission, price, city, source, url)
                        VALUES (%(brand)s, %(model)s, %(year)s, %(mileage)s, %(fuel)s,
                                %(transmission)s, %(price)s, %(city)s, %(source)s, %(url)s)
                    """, listing)
                    saved += 1

        except Exception as e:
            print(f"[DB] Erreur save_listings: {e} | listing: {listing}")
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

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
        CREATE TABLE IF NOT EXISTS price_predictions (
            id SERIAL PRIMARY KEY,
            brand VARCHAR(100),
            model VARCHAR(100),
            year INTEGER,
            mileage INTEGER,
            fuel VARCHAR(50),
            transmission VARCHAR(50),
            condition VARCHAR(50),
            city VARCHAR(100),
            estimated_price FLOAT,
            min_price FLOAT,
            max_price FLOAT,
            confidence FLOAT,
            created_at TIMESTAMP DEFAULT NOW()
        )
    """)
    conn.commit()
    cursor.close()
    conn.close()

def save_prediction(car_data: dict, prediction: dict):
    conn = get_connection()
    cursor = conn.cursor()
    cursor.execute("""
        INSERT INTO price_predictions 
        (brand, model, year, mileage, fuel, transmission, condition, city,
         estimated_price, min_price, max_price, confidence)
        VALUES (%(brand)s, %(model)s, %(year)s, %(mileage)s, %(fuel)s,
                %(transmission)s, %(condition)s, %(city)s,
                %(estimated_price)s, %(min_price)s, %(max_price)s, %(confidence)s)
    """, {**car_data, **prediction})
    conn.commit()
    cursor.close()
    conn.close()
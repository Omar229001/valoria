from fastapi import FastAPI, HTTPException
from contextlib import asynccontextmanager
from src.schemas import CarFeatures, PriceResponse
from src.model import predict_price
from src.database import init_db, save_prediction

@asynccontextmanager
async def lifespan(app: FastAPI):
    init_db()
    yield

app = FastAPI(
    title="Valoria Pricing Service",
    description="Service de cotation de véhicules d'occasion",
    version="1.0.0",
    lifespan=lifespan
)

@app.get("/health")
def health():
    return {"status": "ok", "service": "pricing-service"}

@app.post("/predict", response_model=PriceResponse)
def predict(car: CarFeatures):
    try:
        result = predict_price(
            brand=car.brand,
            model_name=car.model,
            year=car.year,
            mileage=car.mileage,
            fuel=car.fuel,
            transmission=car.transmission,
            condition=car.condition
        )

        save_prediction(
            car_data=car.model_dump(),
            prediction=result
        )

        return PriceResponse(**result)
    except Exception as e:
        raise HTTPException(status_code=500, detail=str(e))

@app.get("/")
def root():
    return {
        "service": "pricing-service",
        "version": "1.0.0",
        "endpoints": ["/health", "/predict"]
    }
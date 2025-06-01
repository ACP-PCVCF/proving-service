from fastapi import FastAPI
from pydantic import BaseModel
import random
import logging

app = FastAPI()

logging.basicConfig(level=logging.INFO)
class SensorRequest(BaseModel):
    shipment_id: str

class SensorResponse(BaseModel):
    distance_km: float
    shipment_id: str

@app.post("/api/v1/sensor-data", response_model=SensorResponse)
def get_sensor_data(req: SensorRequest):
    distance = round(random.uniform(10.0, 500.0), 2)
    logging.info(f"Request received: shipment_id={req.shipment_id}, generated distance={distance} km")

    return {"shipment_id": req.shipment_id, "distance_km": distance}

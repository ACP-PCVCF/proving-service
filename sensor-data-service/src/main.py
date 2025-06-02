from fastapi import FastAPI
from pydantic import BaseModel
from typing import Optional
import random, uuid, json, os, base64
import logging

from cryptography.hazmat.primitives import serialization, hashes
from cryptography.hazmat.primitives.asymmetric import padding


# Logging Setup

logging.basicConfig(level=logging.INFO)
logger = logging.getLogger("sensor-data-service")


# FastAPI App Init

app = FastAPI()


# Models

class Distance(BaseModel):
    actual: Optional[float] = None
    gcd: Optional[float] = None
    sfd: Optional[float] = None

class SensorData(BaseModel):
    distance: Distance

class TceSensorData(BaseModel):
    tceId: str
    camundaProcessInstanceKey: str
    camundaActivityId: str
    sensorkey: str
    signedSensorData: str
    sensorData: SensorData

class SensorRequest(BaseModel):
    shipment_id: str
    tceId: str
    camundaProcessInstanceKey: str
    camundaActivityId: str


# Settings

SENSOR_IDS = ["sensor1", "sensor2", "sensor3", "sensor4", "sensor5"]
KEY_DIR = "./keys"


# Helper Functions

def load_private_key(sensor_id: str):
    path = os.path.join(KEY_DIR, f"{sensor_id}_private.pem")
    logger.info(f"Loading private key for sensor: {sensor_id}")
    with open(path, "rb") as f:
        return serialization.load_pem_private_key(f.read(), password=None)

def sign_data(data: dict, private_key) -> str:
    message = json.dumps(data, sort_keys=True).encode("utf-8")
    signature = private_key.sign(
        message,
        padding.PSS(
            mgf=padding.MGF1(hashes.SHA256()),
            salt_length=padding.PSS.MAX_LENGTH
        ),
        hashes.SHA256()
    )
    return base64.b64encode(signature).decode("utf-8")


# POST Endpoint

@app.post("/api/v1/sensor-data", response_model=TceSensorData)
def post_sensor_data(req: SensorRequest):
    logger.info(f"Incoming request for shipment: {req.shipment_id}")
    logger.info(f"Process Instance Key: {req.camundaProcessInstanceKey} | Activity ID: {req.camundaActivityId} | tceId: {req.tceId}")

    sensor_id = random.choice(SENSOR_IDS)
    logger.info(f"Selected sensor: {sensor_id}")

    distance_val = round(random.uniform(10.0, 500.0), 2)
    logger.info(f"Generated distance: {distance_val} km")

    sensor_data = SensorData(distance=Distance(actual=distance_val))

    private_key = load_private_key(sensor_id)
    signature = sign_data(sensor_data.dict(), private_key)

    logger.info("Sensor data signed successfully.")
    logger.info("Returning signed sensor data...")

    return TceSensorData(
        tceId=req.tceId,
        camundaProcessInstanceKey=req.camundaProcessInstanceKey,
        camundaActivityId=req.camundaActivityId,
        sensorkey=sensor_id,
        signedSensorData=signature,
        sensorData=sensor_data
    )

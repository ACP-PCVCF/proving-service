from pydantic import BaseModel

from models.product_footprint import Distance


class SensorData(BaseModel):
    distance: Distance


class TceSensorData(BaseModel):
    tceId: str
    camundaProcessInstanceKey: str | int
    camundaActivityId: str
    sensorkey: str
    signedSensorData: str
    sensorData: SensorData

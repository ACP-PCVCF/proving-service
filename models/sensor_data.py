from pydantic import BaseModel


class SensorData(BaseModel):
    pass


class TceSensorData(BaseModel):
    tceId: str
    camundaProcessInstanceKey: str | int
    camundaActivityId: str
    sensorkey: str
    signedSensorData: str
    sensorData: SensorData

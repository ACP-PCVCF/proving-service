from typing import Optional
from pydantic import BaseModel
from models.product_footprint import ProductFootprint
from models.hoc_toc_data import TocData, HocData
from models.sensor_data import TceSensorData


class ProofingDocument(BaseModel):
    productFootprint: ProductFootprint
    tocData: list[TocData]
    hocData: list[HocData]
    signedSensorData: Optional[list[TceSensorData]] = None

from typing import Optional
from pydantic import BaseModel
from enum import Enum


class CertificationEnum(str, Enum):
    ISO14083_2023 = "ISO14083:2023"
    GLECv2 = "GLECv2"
    GLECv3 = "GLECv3"
    GLECv3_1 = "GLECv3.1"


class TransportMode(str, Enum):
    ROAD = "road"
    AIR = "air"
    SEA = "sea"
    RAIL = "rail"


class EnergyCarriers(BaseModel):
    energyCarrier: str
    relativeShare: str
    emissionFactorWTW: str
    emissionFactorTTW: str


class TocData(BaseModel):
    tocId: str
    certifications: list[CertificationEnum]
    description: str
    mode: TransportMode
    loadFactor: str
    emptyDistanceFactor: str
    temperatureControl: str
    truckLoadingSequence: str
    airShippingOption: Optional[str]
    flightLength: Optional[str]
    energyCarriers: list[EnergyCarriers]
    co2eIntensityWTW: str
    co2eIntensityTTW: str
    transportActivityUnit: str


class HocData(BaseModel):
    hocId: str
    passhubType: str
    energyCarriers: list[EnergyCarriers]
    co2eIntensityWTW: str
    co2eIntensityTTW: str
    hubActivityUnit: str

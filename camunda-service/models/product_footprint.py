from typing import List, Optional
from pydantic import BaseModel, Field


class Distance(BaseModel):
    actual: Optional[float] = None
    gcd: Optional[float] = None
    sfd: Optional[float] = None


class TceData(BaseModel):
    tceId: str
    prevTceIds: List[str] = []
    hocId: Optional[str] = None
    tocId: Optional[str] = None
    shipmentId: str
    mass: float
    co2eWTW: Optional[float] = None
    co2eTTW: Optional[float] = None
    transportActivity: Optional[float] = None
    distance: Optional[Distance] = None


class ExtensionData(BaseModel):
    mass: float
    shipmentId: str
    tces: List[TceData] = Field(default_factory=list)


class Extension(BaseModel):
    specVersion: str = "2.0.0"
    dataSchema: str
    data: ExtensionData


class ProductFootprint(BaseModel):
    id: str
    specVersion: str = "2.0.0"
    version: int = 0
    created: str
    status: str = "Active"
    companyName: str
    companyIds: List[str]
    productDescription: str
    productIds: List[str]
    productCategoryCpc: int
    productNameCompany: str
    pcf: Optional[float] = None
    comment: str = ""
    extensions: List[Extension] = Field(default_factory=list)

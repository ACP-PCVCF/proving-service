import datetime
import json
import random
import uuid
from typing import Dict, List, Optional, Union
import requests
import os

from cryptography.hazmat.primitives.asymmetric.rsa import RSAPrivateKey

#from models.data_models import SignedTCEData
from models.sensor_data import TceSensorData
from utils.data_utils import convert_sets_to_lists, create_crypto_keys, sign_data
from utils.logging_utils import log_service_call


# This is a mock implementation, in production this would be replaced with real data
CITIES = [
    "Berlin", "Hamburg", "München", "Köln", "Frankfurt", "Stuttgart", "Düsseldorf", 
    "Dortmund", "Essen", "Leipzig", "Bremen", "Dresden", "Hannover", "Nürnberg", 
    "Duisburg", "Bochum", "Wuppertal", "Bielefeld", "Bonn", "Münster", "Karlsruhe", 
    "Mannheim", "Augsburg", "Wiesbaden",
]
PACKAGING_TYPES = ["Pallet", "Container-TEU", "Bulk", "Box", "Crate", None]
INCOTERMS_LIST = ["EXW", "FCA", "CPT", "CIP", "DAP", "DPU", "DDP", "FAS", "FOB", "CFR", "CIF", None]
TEMP_CONTROL_OPTIONS = ["ambient", "refrigerated", None]

def generate_iso_timestamp(base_time=None, add_hours=0):
    """Generiert einen ISO 8601 Timestamp in UTC."""
    if base_time is None:
        base_time = datetime.datetime.now(datetime.timezone.utc)
    time_to_format = base_time + datetime.timedelta(hours=add_hours)
    return time_to_format.isoformat(timespec="seconds").replace("+00:00", "Z")

def generate_location_data(city_name: str) -> Dict:
    """Generate location data for a given city."""
    return {
        "type": "PhysicalLocation",
        "locationName": city_name,
        "countryCode": "DE",
        "postalCode": f"{random.randint(10000, 99999)}",
        "city": city_name,
    }


def generate_package_data() -> Dict:
    """Generate package type and amount data."""
    result = {}
    package_type = random.choice(PACKAGING_TYPES)

    if package_type:
        result["packagingOrTrEqType"] = package_type

        if package_type == "Container-TEU":
            result["packagingOrTrEqAmount"] = (
                "2.00" if random.random() < 0.7 else "2.25"
            )
        elif package_type == "Pallet":
            result["packagingOrTrEqAmount"] = f"{random.randint(1, 26)}.00"
        else:
            result["packagingOrTrEqAmount"] = f"{random.uniform(1, 10):.2f}"

    return result


def calculate_emissions(transport_activity_val: float) -> Dict:
    """Calculate various emission values based on transport activity."""
    emissions = {}

    # Base CO2 calculations
    co2_factor_ttw_per_tkm = random.uniform(0.06, 0.12)
    co2e_ttw_val = transport_activity_val * co2_factor_ttw_per_tkm
    wtw_multiplier = random.uniform(1.15, 1.25)
    co2e_wtw_val = co2e_ttw_val * wtw_multiplier

    emissions["co2_factor_ttw_per_tkm"] = f"{co2_factor_ttw_per_tkm:.3f}"
    emissions["co2eTTW"] = f"{co2e_ttw_val:.3f}"
    emissions["wtw_multiplier"] = f"{wtw_multiplier:.3f}"
    emissions["co2eWTW"] = f"{co2e_wtw_val:.3f}"

    # Optional emissions with probability factors
    if random.random() < 0.8:
        emissions["noxTTW"] = (
            f"{(transport_activity_val * random.uniform(0.0005, 0.003)):.4f}"
        )
    if random.random() < 0.3:
        emissions["soxTTW"] = (
            f"{(transport_activity_val * random.uniform(0.00001, 0.00005)):.5f}"
        )
    if random.random() < 0.5:
        emissions["ch4TTW"] = (
            f"{(transport_activity_val * random.uniform(0.00002, 0.0001)):.5f}"
        )
    if random.random() < 0.7:
        emissions["pmTTW"] = (
            f"{(transport_activity_val * random.uniform(0.00003, 0.00015)):.5f}"
        )

    return emissions

class SensorDataService:
    """Service for retrieving and generating transport emission data."""
    
    def __init__(self):
        log_service_call("SensorDataService", "__init__")
        self.base_url = os.getenv("SENSOR_SERVICE_API_URL", "http://localhost:8000")

    def call_service_sensordata(self, variables) -> TceSensorData:
        shipment_id = variables.get("shipment_id", "unknown")
        tce_id = variables.get("tceId")
        process_instance_key = variables.get("camundaProcessInstanceKey")
        activity_id = variables.get("camundaActivityId")

        payload = {
            "shipment_id": shipment_id,
            "tceId": tce_id,
            "camundaProcessInstanceKey": process_instance_key,
            "camundaActivityId": activity_id
        }

        log_service_call(
            service_name="SensorDataService",
            method_name="call_service_sensordata",
            message="Sending sensor data request",
            payload=payload
        )

        try:
            response = requests.post(
                f"{self.base_url}/api/v1/sensor-data",
                json=payload,
                timeout=10
            )
            log_service_call(
                service_name="SensorDataService",
                method_name="call_service_sensordata",
                message=f"Received response {response.status_code}",
                payload=response.json() if response.ok else response.text
            )

            response.raise_for_status()
            return TceSensorData(**response.json())

        except requests.RequestException as e:
            log_service_call(
                service_name="SensorDataService",
                method_name="call_service_sensordata",
                message=f"HTTP request failed: {str(e)}",
                payload=payload
            )
            raise



    # def get_mock_sensor_data(
    #         self,
    #         shipment_id: str,
    #         mass_kg: float = None,
    #         distance_km: float = None,
    #         prev_tce_id: Optional[Union[str, List[str]]] = None,
    #         start_time: Optional[datetime.datetime] = None,
    # ) -> Dict[str, SignedTCEData]:
    #     """
    #     Generate or retrieve transport carbon emission (TCE) data for a shipment.
        
    #     Args:
    #         shipment_id: Unique identifier for the shipment
    #         mass_kg: Mass of shipment in kilograms (random if not provided)
    #         distance_km: Distance of transport in kilometers (random if not provided)
    #         prev_tce_id: Previous TCE ID for linking records
    #         start_time: Starting time for the transport
            
    #     Returns:
    #         Dictionary containing the TCE data and cryptographic signatures
    #     """

        
    #     # Use provided values or generate random ones
    #     mass_kg = mass_kg or random.uniform(1000, 20000)
    #     distance_km = distance_km or random.uniform(10, 1000)
        
    #     # Generate cryptographic keys
    #     activity_private_key, activity_public_key_pem = create_crypto_keys()
    #     tce_id = f"TCE_{uuid.uuid4()}"
        
    #     # Initialize base TCE data
    #     tce_data = {
    #         "tceId": tce_id,
    #         "shipmentId": shipment_id,
    #         "mass": f"{mass_kg:.2f}",
    #         "distance": {
    #             "value": f"{distance_km:.2f}",
    #             "unit": "km",
    #             "dataSource": "Simulated",
    #         },
    #     }

    #     # Calculate transport activity and emissions
    #     transport_activity_val = (distance_km * mass_kg) / 1000.0
    #     tce_data["transportActivity"] = f"{transport_activity_val:.3f}"

    #     # Add emission data
    #     tce_data.update(calculate_emissions(transport_activity_val))

    #     # Add TOC or HOC ID (Transport Operator Code or Handling Operator Code)
    #     if random.choice([True, False]):
    #         tce_data["tocId"] = f"TOC_{uuid.uuid4()}"
    #         tce_data["hocId"] = None
    #     else:
    #         tce_data["hocId"] = f"HOC_{uuid.uuid4()}"
    #         tce_data["tocId"] = None

    #     # Ensure at least one operator ID exists
    #     if tce_data.get("tocId") is None and tce_data.get("hocId") is None:
    #         tce_data["tocId"] = f"TOC_fallback_{uuid.uuid4()}"

    #     # Add previous TCE IDs if provided
    #     if prev_tce_id:
    #         tce_data["prevTceIds"] = (
    #             [prev_tce_id] if not isinstance(prev_tce_id, list) else prev_tce_id
    #         )

    #     # Add consignment ID with 70% probability
    #     if random.random() < 0.7:
    #         tce_data["consignmentId"] = f"CON_{uuid.uuid4()}"

    #     # Add packaging data
    #     tce_data.update(generate_package_data())

    #     # Generate origin and destination
    #     origin_city = random.choice(CITIES)
    #     destination_city_options = [c for c in CITIES if c != origin_city]
    #     if not destination_city_options:
    #         destination_city_options = CITIES
    #     destination_city = random.choice(destination_city_options)

    #     tce_data["origin"] = generate_location_data(origin_city)
    #     tce_data["destination"] = generate_location_data(destination_city)

    #     # Calculate departure and arrival times
    #     avg_speed_kmh = random.uniform(60, 80)
    #     duration_hours = int((distance_km / avg_speed_kmh) * random.uniform(1, 1.2))
    #     departure_time_iso = generate_iso_timestamp(start_time)
    #     tce_data["departureAt"] = departure_time_iso
    #     dt_departure = datetime.datetime.fromisoformat(
    #         departure_time_iso.replace("Z", "+00:00")
    #     )
    #     tce_data["arrivalAt"] = generate_iso_timestamp(
    #         dt_departure, add_hours=duration_hours
    #     )

    #     # Set flight and voyage numbers to None
    #     tce_data["flightNo"] = None
    #     tce_data["voyageNo"] = None

    #     # Add incoterms randomly
    #     incoterm = random.choice(INCOTERMS_LIST)
    #     if incoterm:
    #         tce_data["incoterms"] = incoterm

    #     # Add temperature control with random selection
    #     temp_control = random.choice(TEMP_CONTROL_OPTIONS)
    #     if temp_control:
    #         tce_data["temperatureControl"] = temp_control

    #     # Clean up data structure
    #     tce_data_no_sets = convert_sets_to_lists(tce_data)
    #     cleaned_tce_data = {k: v for k, v in tce_data_no_sets.items() if v is not None}

    #     # Serialize, sign, and prepare result
    #     message_json_str = json.dumps(
    #         cleaned_tce_data, sort_keys=True, separators=(",", ":")
    #     )
    #     signature_hex = sign_data(activity_private_key, message_json_str)

    #     result_dict = {
    #         "activityDataJson": message_json_str,
    #         "activitySignature": signature_hex,
    #         "activityPublicKeyPem": activity_public_key_pem,
    #     }

    #     return {tce_id: result_dict}
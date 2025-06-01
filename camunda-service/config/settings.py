import os

# Zeebe connection settings
ZEEBE_ADDRESS = "localhost:26500"

# API endpoints
#PROOFING_SERVICE_URL = "http://localhost:8000/api/proofing"
#SENSOR_DATA_SERVICE_URL = "http://localhost:8001/api/sensordata"

# Authentication

# Logging
LOG_LEVEL = os.environ.get("LOG_LEVEL", "INFO")

# File paths
ACTIVITIES_OUTPUT_PATH = os.environ.get("ACTIVITIES_OUTPUT_PATH", "activities.json")

# Service timeouts (seconds)
REQUEST_TIMEOUT = int(os.environ.get("REQUEST_TIMEOUT", "30"))
from cryptography.hazmat.primitives.asymmetric import rsa
from cryptography.hazmat.primitives import serialization
import os

SENSOR_IDS = ["sensor1", "sensor2", "sensor3", "sensor4", "sensor5"]
KEY_DIR = "keys"
os.makedirs(KEY_DIR, exist_ok=True)

for sensor_id in SENSOR_IDS:
    private_key = rsa.generate_private_key(
        public_exponent=65537,
        key_size=2048,
    )

    with open(f"{KEY_DIR}/{sensor_id}_private.pem", "wb") as f:
        f.write(private_key.private_bytes(
            encoding=serialization.Encoding.PEM,
            format=serialization.PrivateFormat.PKCS8,
            encryption_algorithm=serialization.NoEncryption()
        ))

    print(f"Key for {sensor_id} saved!")

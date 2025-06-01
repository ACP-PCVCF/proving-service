import sqlite3
import json
from config.database_config import DatabaseConfig
from typing import Optional, Dict, Any


class HocTocDatabase:
    def __init__(self):
        self.db_path = DatabaseConfig.DB_PATH
        self.timeout = DatabaseConfig.TIMEOUT
        self.init_database()

    def init_database(self):
        """Initialize the database with required tables."""
        conn = sqlite3.connect(self.db_path)
        cursor = conn.cursor()

        # Create HOC (Hub of Consumption) table
        cursor.execute('''
            CREATE TABLE IF NOT EXISTS hoc_data (
                hoc_id TEXT PRIMARY KEY,
                passhub_type TEXT,
                energy_carriers TEXT,  -- JSON string
                co2e_intensity_wtw TEXT,
                co2e_intensity_ttw TEXT,
                hub_activity_unit TEXT
            )
        ''')

        # Create TOC (Transport Operation Category) table
        cursor.execute('''
            CREATE TABLE IF NOT EXISTS toc_data (
                toc_id TEXT PRIMARY KEY,
                certifications TEXT,  -- JSON string
                description TEXT,
                mode TEXT,
                load_factor TEXT,
                empty_distance_factor TEXT,
                temperature_control TEXT,
                truck_loading_sequence TEXT,
                air_shipping_option TEXT,
                flight_length TEXT,
                energy_carriers TEXT,  -- JSON string
                co2e_intensity_wtw TEXT,
                co2e_intensity_ttw TEXT,
                transport_activity_unit TEXT
            )
        ''')

        conn.commit()
        conn.close()

    def populate_from_mock_data(self, mock_data_function):
        """Populate database from your existing mock data."""
        conn = sqlite3.connect(self.db_path)
        cursor = conn.cursor()

        all_ids = ["100", "101", "102", "103",
                   "200", "201", "202", "203", "204"]

        for id_val in all_ids:
            data = mock_data_function(id_val)
            if data is None:
                continue

            if "hocId" in data:
                cursor.execute('''
                    INSERT OR REPLACE INTO hoc_data 
                    (hoc_id, passhub_type, energy_carriers, co2e_intensity_wtw, 
                     co2e_intensity_ttw, hub_activity_unit)
                    VALUES (?, ?, ?, ?, ?, ?)
                ''', (
                    data["hocId"],
                    data["passhubType"],
                    json.dumps(data["energyCarriers"]),
                    data["co2eIntensityWTW"],
                    data["co2eIntensityTTW"],
                    data["hubActivityUnit"]
                ))

            elif "tocId" in data:
                cursor.execute('''
                    INSERT OR REPLACE INTO toc_data 
                    (toc_id, certifications, description, mode, load_factor, 
                     empty_distance_factor, temperature_control, truck_loading_sequence,
                     air_shipping_option, flight_length, energy_carriers, 
                     co2e_intensity_wtw, co2e_intensity_ttw, transport_activity_unit)
                    VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
                ''', (
                    data["tocId"],
                    json.dumps(data.get("certifications", [])),
                    data["description"],
                    data["mode"],
                    data["loadFactor"],
                    data["emptyDistanceFactor"],
                    data.get("temperatureControl"),
                    data.get("truckLoadingSequence"),
                    data.get("airShippingOption"),
                    data.get("flightLength"),
                    json.dumps(data["energyCarriers"]),
                    data["co2eIntensityWTW"],
                    data["co2eIntensityTTW"],
                    data["transportActivityUnit"]
                ))

        conn.commit()
        conn.close()

    def get_hoc_data(self, hoc_id: str) -> Optional[Dict[str, Any]]:
        """Get HOC data by ID."""
        conn = sqlite3.connect(self.db_path)
        cursor = conn.cursor()

        cursor.execute('SELECT * FROM hoc_data WHERE hoc_id = ?', (hoc_id,))
        row = cursor.fetchone()
        conn.close()

        if row:
            return {
                "hocId": row[0],
                "passhubType": row[1],
                "energyCarriers": json.loads(row[2]),
                "co2eIntensityWTW": row[3],
                "co2eIntensityTTW": row[4],
                "hubActivityUnit": row[5]
            }
        return None

    def get_toc_data(self, toc_id: str) -> Optional[Dict[str, Any]]:
        """Get TOC data by ID."""
        conn = sqlite3.connect(self.db_path)
        cursor = conn.cursor()

        cursor.execute('SELECT * FROM toc_data WHERE toc_id = ?', (toc_id,))
        row = cursor.fetchone()
        conn.close()

        if row:
            return {
                "tocId": row[0],
                "certifications": json.loads(row[1]),
                "description": row[2],
                "mode": row[3],
                "loadFactor": row[4],
                "emptyDistanceFactor": row[5],
                "temperatureControl": row[6],
                "truckLoadingSequence": row[7],
                "airShippingOption": row[8],
                "flightLength": row[9],
                "energyCarriers": json.loads(row[10]),
                "co2eIntensityWTW": row[11],
                "co2eIntensityTTW": row[12],
                "transportActivityUnit": row[13]
            }
        return None

#!/bin/bash

INPUT_TOPIC="shipments"
KAFKA_CONTAINER="host-kafka-1"
BROKER="localhost:9092"
JSON_FILE="shipment_3.json"

# Prüfen, ob die JSON-Datei existiert
if [ ! -f "$JSON_FILE" ]; then
  echo "Fehler: Die Datei '$JSON_FILE' wurde nicht gefunden."
  exit 1
fi

JSON_CONTENT=$(cat "$JSON_FILE" | tr -d '\n\r')

echo "Sende Test-Nachricht an Topic $INPUT_TOPIC ..."
printf '%s\n' "$JSON_CONTENT" | \
  docker exec -i $KAFKA_CONTAINER \
    kafka-console-producer --broker-list $BROKER --topic $INPUT_TOPIC > /dev/null

echo "Nachricht gesendet."

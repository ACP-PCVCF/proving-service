#!/bin/bash

INPUT_TOPIC="shipments"
KAFKA_CONTAINER="host-kafka-1"
BROKER="localhost:9092"
#JSON_FILE="shipment_1.json"

if [ -z "$1" ]; then
  echo "Fehler: Kein String als Argument übergeben."
  exit 1
fi

# Prüfen, ob die JSON-Datei existiert
if [ ! -f "$1" ]; then
  echo "Fehler: Die Datei '$1' wurde nicht gefunden."
  exit 1
fi

JSON_CONTENT=$(cat "$1" | tr -d '\n\r')

echo "Sende Test-Nachricht an Topic $INPUT_TOPIC ..."
printf '%s\n' "$JSON_CONTENT" | \
  docker exec -i $KAFKA_CONTAINER \
    kafka-console-producer --broker-list $BROKER --topic $INPUT_TOPIC > /dev/null

echo "Nachricht gesendet."

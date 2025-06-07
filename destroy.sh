#!/bin/bash
set -e

echo "Deleting Proving Service..."
kubectl delete -f k8s/proving-service.yaml --ignore-not-found

echo "Deleting Kafka topic job..."
kubectl delete -f k8s/kafka-topic-job.yaml --ignore-not-found

echo "Uninstalling Kafka Helm release..."
helm uninstall kafka -n proving-system || true

echo "Done."

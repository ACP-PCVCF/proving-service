#!/bin/bash

set -e

echo "Start Kafka..."
helm install kafka bitnami/kafka \
  --namespace proving-system \
  --create-namespace \
  -f k8s/kafka-values.yaml

echo "Waiting for Kafka to be ready..."
kubectl wait --for=condition=Ready pod -l app.kubernetes.io/name=kafka -n proving-system --timeout=120s

echo "Execute Kafka topics job..."
kubectl apply -f k8s/kafka-topic-job.yaml

echo "Waiting for Kafka topics job to be done..."
kubectl wait --for=condition=complete job/create-kafka-topics -n proving-system --timeout=60s

echo "Building Docker image..."
eval $(minikube docker-env)
docker build --platform linux/amd64 -t proving-service:latest .

echo "Starting Proving Service in Kubernetes..."
kubectl apply -f k8s/proving-service.yaml

echo "Done."
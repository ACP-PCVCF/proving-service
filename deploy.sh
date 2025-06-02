#!/bin/bash
set -e

NAMESPACE="proving-system"

# Start Minikube if not running
if ! minikube status | grep -q "Running"; then
  echo "Minikube is not running. Starting it..."
  minikube start --memory=8192 --cpus=4 --driver=docker
else
  echo "Minikube is already running."
fi

# Use Minikube Docker context
echo "Switching to Minikube Docker context..."
eval $(minikube docker-env)

# Build Docker images
echo "Building Docker images..."
docker build -t sensor-data-service:latest ./sensor-data-service
docker build -t camunda-service:latest ./camunda-service
#docker build --platform=linux/amd64 -t proving-service:latest ./proving-service

# Deploy services to Kubernetes
echo "Applying Kubernetes manifests..."
kubectl apply -f ./sensor-data-service/k8s/sensor-service.yaml -n $NAMESPACE
kubectl apply -f ./camunda-service/k8s/camunda-service.yaml -n $NAMESPACE
#kubectl apply -f ./proving-service/k8s/proving-service-deployment.yaml -n $NAMESPACE

echo "All services deployed to namespace $NAMESPACE."
kubectl get pods -n $NAMESPACE

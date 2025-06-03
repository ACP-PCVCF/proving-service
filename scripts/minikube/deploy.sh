#!/bin/bash
set -e

NAMESPACE="proving-system"

echo "Checking if Minikube is running..."
if ! minikube status | grep -q "Running"; then
  echo "Starting Minikube..."
  minikube start --memory=8192 --cpus=4 --driver=docker
else
  echo "Minikube is already running."
fi

echo "Switching to Minikube Docker context..."
eval "$(minikube docker-env)"

echo "Rebuilding Docker images with latest code..."
docker build -t sensor-data-service:latest ./sensor-data-service 
docker build -t camunda-service:latest ./camunda-service
# docker build --platform=linux/amd64 -t proving-service:latest ./proving-service

echo "Applying updated manifests..."
kubectl apply -f ./sensor-data-service/k8s/sensor-data-service.yaml -n $NAMESPACE
kubectl apply -f ./camunda-service/k8s/camunda-service.yaml -n $NAMESPACE
# kubectl apply -f ./proving-service/k8s/proving-service-deployment.yaml -n $NAMESPACE

echo "Triggering rollout restarts..."
kubectl rollout restart deployment/sensor-data-service -n $NAMESPACE
kubectl rollout restart deployment/camunda-service -n $NAMESPACE
# kubectl rollout restart deployment/proving-service -n $NAMESPACE

echo "Waiting for updated pods to become ready..."
kubectl rollout status deployment/sensor-data-service -n $NAMESPACE
kubectl rollout status deployment/camunda-service -n $NAMESPACE
# kubectl rollout status deployment/proving-service -n $NAMESPACE

echo "Rollout complete. Current pods in '$NAMESPACE':"
kubectl get pods -n $NAMESPACE

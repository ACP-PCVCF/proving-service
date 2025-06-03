#!/bin/bash
set -e

NAMESPACE="proving-system"
KIND_CLUSTER="kind"

echo "Checking if Kind cluster '$KIND_CLUSTER' is running..."
if ! kind get clusters | grep -q "$KIND_CLUSTER"; then
  echo "Kind cluster '$KIND_CLUSTER' not found. Start it first using your setup script."
  exit 1
fi

echo "Rebuilding Docker images with latest code..."
docker build -t sensor-data-service:latest ./sensor-data-service 
docker build -t camunda-service:latest ./camunda-service
# docker build -t proving-service:latest ./proving-service

echo "Loading Docker images into Kind cluster..."
kind load docker-image sensor-data-service:latest --name "$KIND_CLUSTER"
kind load docker-image camunda-service:latest --name "$KIND_CLUSTER"
# kind load docker-image proving-service:latest --name "$KIND_CLUSTER"

echo "Applying updated Kubernetes manifests..."
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

#!/bin/bash
set -e

NAMESPACE="proving-system"
KIND_CLUSTER="kind-cluster"

# Check if kind cluster exists
if ! kind get clusters | grep -q "$KIND_CLUSTER"; then
  echo "Creating Kind cluster named '$KIND_CLUSTER'..."
  kind create cluster --name "$KIND_CLUSTER"
else
  echo "Kind cluster '$KIND_CLUSTER' already exists."
fi

echo "Checking if Camunda Helm repo is added..."
if ! helm repo list | grep -q camunda; then
  echo "Adding Camunda Helm repo..."
  helm repo add camunda https://helm.camunda.io
else
  echo "Camunda Helm repo already present."
fi

echo "Updating Helm repos..."
helm repo update

echo "Checking if Camunda is already installed..."
if ! helm list -n $NAMESPACE | grep -q camunda; then
  echo "Installing Camunda in namespace $NAMESPACE..."
  helm install camunda camunda/camunda-platform \
    -n $NAMESPACE --create-namespace \
    -f ./camunda-platform-core-kind-values.yaml
else
  echo "Camunda already installed in $NAMESPACE."
fi

echo "Waiting for Camunda pods to be created..."
until kubectl get pods -n $NAMESPACE 2>/dev/null | grep -q "camunda"; do
  echo "Still waiting for Camunda pods..."
  sleep 2
done

echo "Waiting for Camunda pods to be ready..."
kubectl wait --for=condition=ready pod --all -n $NAMESPACE --timeout=300s

echo "Building Docker images..."
docker build -t sensor-data-service:latest ./sensor-data-service 
docker build -t camunda-service:latest ./camunda-service
# docker build -t proving-service:latest ./proving-service

echo "Loading images into Kind cluster..."
kind load docker-image sensor-data-service:latest --name "$KIND_CLUSTER"
kind load docker-image camunda-service:latest --name "$KIND_CLUSTER"
# kind load docker-image proving-service:latest --name "$KIND_CLUSTER"

echo "Deploying services to Kubernetes..."
kubectl apply -f ./sensor-data-service/k8s/sensor-data-service.yaml -n $NAMESPACE
kubectl apply -f ./camunda-service/k8s/camunda-service.yaml -n $NAMESPACE
# kubectl apply -f ./proving-service/k8s/proving-service-deployment.yaml -n $NAMESPACE

echo "Waiting for service pods to be ready..."
kubectl wait --for=condition=ready pod --all -n $NAMESPACE --timeout=180s

echo "All services deployed in '$NAMESPACE'. Current pods:"
kubectl get pods -n $NAMESPACE

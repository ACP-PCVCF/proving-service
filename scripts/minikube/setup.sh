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

echo "Checking if Camunda Helm repo is added..."
if ! helm repo list | grep -q camunda; then
  echo "Adding Camunda Helm repo..."
  helm repo add camunda https://helm.camunda.io
else
  echo "Camunda Helm repo already added."
fi

echo "Updating Helm repos..."
helm repo update

echo "Checking if Camunda is already installed..."
if ! helm list -n $NAMESPACE | grep -q camunda; then
  echo "Installing Camunda in namespace $NAMESPACE..."
  helm install camunda camunda/camunda-platform \
    -n $NAMESPACE --create-namespace \
    -f ./camunda-platform/camunda-platform-core-kind-values.yaml
else
  echo "Camunda is already installed in $NAMESPACE."
fi

echo "Waiting for Camunda pods to be created..."
until kubectl get pods -n $NAMESPACE 2>/dev/null | grep -q "camunda"; do
  echo "Still waiting for Camunda pods..."
  sleep 2
done

echo "Waiting for all Camunda pods to be ready..."
kubectl wait --for=condition=ready pod --all -n $NAMESPACE --timeout=300s

echo "Building Docker images..."
#docker build -t sensor-data-service:latest ./sensor-data-service 
#docker build -t camunda-service:latest ./camunda-service
# docker build --platform=linux/amd64 -t proving-service:latest ./proving-service

echo "Deploying services to Kubernetes..."
#kubectl apply -f ./sensor-data-service/k8s/sensor-data-service.yaml -n $NAMESPACE
#kubectl apply -f ./camunda-service/k8s/camunda-service.yaml -n $NAMESPACE
# kubectl apply -f ./proving-service/k8s/proving-service-deployment.yaml -n $NAMESPACE

echo "Waiting for all deployed service pods to be ready..."
kubectl wait --for=condition=ready pod --all -n $NAMESPACE --timeout=180s

echo "All services deployed successfully to namespace '$NAMESPACE'."
kubectl get pods -n $NAMESPACE

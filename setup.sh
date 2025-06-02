#!/bin/bash
set -e

NAMESPACE="proving-system"

echo "Starting Minikube cluster..."
minikube start --memory=8192 --cpus=4 --driver=docker

echo "Adding and updating Camunda Helm repo..."
helm repo add camunda https://helm.camunda.io
helm repo update

echo "Installing Camunda in namespace $NAMESPACE..."
helm install camunda camunda/camunda-platform \
  -n $NAMESPACE --create-namespace \
  -f ./camunda-platform-core-kind-values.yaml

echo "Waiting for Camunda pods to be ready..."
kubectl wait --for=condition=ready pod --all -n $NAMESPACE --timeout=3000s

echo "All pods are ready. Showing pod status:"
kubectl get pods -n $NAMESPACE

echo "Setup complete. You can now run ./deploy.sh"

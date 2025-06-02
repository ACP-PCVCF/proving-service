#!/bin/bash

set -e

NAMESPACE="proving-system"

echo "Uninstalling Helm release 'camunda'..."
helm uninstall camunda -n $NAMESPACE || echo "Helm release 'camunda' not found or already removed."

echo "Deleting deployment: proofing-service..."
kubectl delete deployment proofing-service -n $NAMESPACE --ignore-not-found

echo "Deleting service: proofing-service..."
kubectl delete service proofing-service -n $NAMESPACE --ignore-not-found

echo "Deleting deployment: sensor-data-service..."
kubectl delete deployment sensor-data-service -n $NAMESPACE --ignore-not-found

echo "Deleting service: sensor-data-service..."
kubectl delete service sensor-data-service -n $NAMESPACE --ignore-not-found

echo "Deleting deployment: camunda-service..."
kubectl delete deployment camunda-service -n $NAMESPACE --ignore-not-found

echo "Deleting service: camunda-service..."
kubectl delete service camunda-service -n $NAMESPACE --ignore-not-found

echo "Stopping Minikube..."
minikube stop

echo "Cleanup completed."

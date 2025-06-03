#!/bin/bash

set -e

NAMESPACE="proving-system"

echo "Uninstalling Helm release 'camunda'..."
helm uninstall camunda -n $NAMESPACE || echo "Helm release 'camunda' not found or already removed."

echo "Deleting service deployments..."
kubectl delete deployment proofing-service sensor-data-service camunda-service -n $NAMESPACE --ignore-not-found

echo "Deleting services..."
kubectl delete service proofing-service sensor-data-service camunda-service -n $NAMESPACE --ignore-not-found

echo "Deleting configmaps, secrets and PVCs in $NAMESPACE..."
kubectl delete configmap --all -n $NAMESPACE --ignore-not-found
kubectl delete secret --all -n $NAMESPACE --ignore-not-found
kubectl delete pvc --all -n $NAMESPACE --ignore-not-found

echo "Deleting namespace $NAMESPACE..."
kubectl delete namespace $NAMESPACE --ignore-not-found

echo "Stopping Minikube..."
minikube stop

echo "Deleting Minikube cluster..."
minikube delete

echo "Cleanup completed."

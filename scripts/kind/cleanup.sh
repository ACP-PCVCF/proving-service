#!/bin/bash
set -e

NAMESPACE="proving-system"
KIND_CLUSTER="kind"

echo "Uninstalling Helm release 'camunda' (if exists)..."
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

echo "Checking if Kind cluster '$KIND_CLUSTER' exists..."
if kind get clusters | grep -q "$KIND_CLUSTER"; then
  echo "Deleting Kind cluster '$KIND_CLUSTER'..."
  kind delete cluster --name "$KIND_CLUSTER"
else
  echo "Kind cluster '$KIND_CLUSTER' does not exist or already deleted."
fi

echo "Cleanup completed for Kind environment."

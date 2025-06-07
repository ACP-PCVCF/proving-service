#!/bin/bash
set -e

echo "Applying Zookeeper Deployment and Service..."
kubectl apply -f k8s/zookeeper-deployment.yaml
kubectl apply -f k8s/zookeeper-service.yaml

echo "Applying Kafka Deployment and Service..."
kubectl apply -f k8s/kafka-deployment.yaml
kubectl apply -f k8s/kafka-service.yaml

echo "Waiting for Kafka and Zookeeper to be ready..."
kubectl wait --for=condition=available --timeout=120s deployment/zookeeper
kubectl wait --for=condition=available --timeout=120s deployment/kafka

echo "Applying Proofing Service Deployment and Service..."
kubectl apply -f k8s/proofing-deployment.yaml
kubectl apply -f k8s/proofing-service.yaml

echo "Waiting for Proofing Service to be ready..."
kubectl wait --for=condition=available --timeout=120s deployment/proofing

echo "Creating Kafka topics with Job..."
kubectl apply -f k8s/create-topics-job.yaml

echo "Waiting for topic creation job to finish..."
kubectl wait --for=condition=complete --timeout=60s job/create-topics

echo "All services started and topics created successfully!"


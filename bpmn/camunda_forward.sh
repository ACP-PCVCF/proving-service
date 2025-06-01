#!/bin/bash

# Camunda Platform Port Forward Script
# This script sets up port forwarding for all Camunda Platform services

echo "🚀 Starting Camunda Platform port forwarding..."
echo "📋 Services will be available at:"
echo "   - Zeebe Gateway (gRPC): localhost:26500"
echo "   - Zeebe Gateway (HTTP): localhost:8088"
echo "   - Operate: localhost:8081"
echo "   - Tasklist: localhost:8082"
echo ""

# Function to handle cleanup on script exit
cleanup() {
    echo ""
    echo "🛑 Stopping all port-forward processes..."
    # Kill all kubectl port-forward processes started by this script
    jobs -p | xargs -r kill
    echo "✅ All port-forward processes stopped"
    exit 0
}

# Set up trap to handle Ctrl+C and script exit
trap cleanup SIGINT SIGTERM EXIT

# Start port forwarding in background
echo "🔌 Setting up port forwards..."

# Zeebe Gateway - gRPC
echo "Starting Zeebe Gateway (gRPC) on port 26500..."
kubectl port-forward svc/camunda-platform-zeebe-gateway 26500:26500 -n default &
ZEEBE_GRPC_PID=$!

# Zeebe Gateway - HTTP
echo "Starting Zeebe Gateway (HTTP) on port 8088..."
kubectl port-forward svc/camunda-platform-zeebe-gateway 8088:8080 -n default &
ZEEBE_HTTP_PID=$!

# Operate
echo "Starting Operate on port 8081..."
kubectl port-forward svc/camunda-platform-operate 8081:80 -n default &
OPERATE_PID=$!

# Tasklist
echo "Starting Tasklist on port 8082..."
kubectl port-forward svc/camunda-platform-tasklist 8082:80 -n default &
TASKLIST_PID=$!

# Wait a moment for connections to establish
sleep 2

echo ""
echo "✅ All port forwards are running!"
echo ""
echo "🌐 Access your services at:"
echo "   - Zeebe Gateway (gRPC): localhost:26500"
echo "   - Zeebe Gateway (HTTP): localhost:8088"
echo "   - Operate Web UI: http://localhost:8081"
echo "   - Tasklist Web UI: http://localhost:8082"
echo ""
echo "📝 Press Ctrl+C to stop all port forwards"
echo ""

# Keep the script running and show status
while true; do
    sleep 10
    # Check if any port-forward processes have died
    if ! kill -0 $ZEEBE_GRPC_PID 2>/dev/null; then
        echo "⚠️  Zeebe Gateway (gRPC) port-forward stopped unexpectedly"
    fi
    if ! kill -0 $ZEEBE_HTTP_PID 2>/dev/null; then
        echo "⚠️  Zeebe Gateway (HTTP) port-forward stopped unexpectedly"
    fi
    if ! kill -0 $OPERATE_PID 2>/dev/null; then
        echo "⚠️  Operate port-forward stopped unexpectedly"
    fi
    if ! kill -0 $TASKLIST_PID 2>/dev/null; then
        echo "⚠️  Tasklist port-forward stopped unexpectedly"
    fi
done
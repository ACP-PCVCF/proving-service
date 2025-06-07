# Set up Kubernetes Cluster

## 1. Add Camunda Helm Repository
```bash
helm repo add camunda https://helm.camunda.io
helm repo update
```

## 2. Start Minikube
Start a Minikube cluster with sufficient resources. You may adjust the values based on your machine/driver:
```bash
minikube start --memory=8192 --cpus=4 --driver=docker
```

## 3. Install Camunda via Helm
Install Camunda into the camunda namespace (or another of your choice) using your custom configuration file:

```bash
helm install camunda camunda/camunda-platform \
  -n proving-system --create-namespace \
  -f camunda-platform/camunda-platform-core-kind-values.yaml
```
Notes:
- If you do not use the default namespace, you must always include -n camunda (or your chosen namespace) in kubectl commands.
- The YAML configuration file should be available locally or from your cloud storage. Adjust the path if needed.

Sidenote: to uninstall use
```bash
helm uninstall camunda -n proving-system
```

## 4. Verify Camunda Installation
Wait a few minutes for all services to initialize. Then check the status:

```bash
kubectl get pods -n proving-system
```
You should see multiple Camunda components like ```camunda-zeebe```, ```camunda-operate```, and others showing ```STATUS: Running```.

If some show ```Pending``` or ```ContainerCreating```, wait until they are fully up.

## 5. Port Forward Zeebe Gateway for Local Access
To connect the Camunda Modeler to Zeebe, forward the gateway port:

```bash
kubectl port-forward svc/camunda-zeebe-gateway 26500:26500 -n proving-system
```
Keep this terminal open while deploying models from the Camunda Modeler.

## 6. Deploy BPMN Models from Camunda Modeler
In the Camunda Modeler:

1. Open your BPMN file.

2. Select Camunda 8 → Self-Managed.

3. Use the following connection settings:

- Zeebe Gateway Address: localhost:26500

- Authentication: None

4. Click Deploy Current Diagram.
   
5. Start process instances (only after you deployed the other Microservices to the Cluster).

## 7. Access Camunda Operate
To view and manage process instances, forward the Camunda Operate service:

```bash
kubectl port-forward svc/camunda-operate 8081:80 -n proving-system
```
Then open your browser at: http://localhost:8081

## 8. Build and Load Images in Minikube
Before deploying, make sure to build your images inside Minikube's Docker context (we don't need to upload to DockerHub with Minikube, we keep the images local):

Kubernetes does not run source code directly, it runs containers that are created from Docker images. That means every time you create a new service or want to deploy one, you first need to build its Docker image.

Make sure that the commands have to be executed in the right folder of the dedicated repository.

If you use a Macbook that is based on an ARM64 architecture ((M1, M2, M3, M4) make sure to add ```--platform=linux/amd64``` to build your proving-service Dockerfile (see below). The Risc Zero Toolchain doesn't support ```linux/aarch64``` but only can installed on ```linux/amd64``` platforms.

```bash
eval $(minikube docker-env)
docker build -t sensor-data-service:latest .
docker build -t camunda-service:latest .
docker build --platform=linux/amd64 -t proving-service:latest .
```
Then apply your Kubernetes YAML files (you might need to switch folders):

```bash
kubectl apply -f sensor-deployment.yaml -n proving-system
kubectl apply -f camunda-service-deployment.yaml -n proving-system
kubectl apply -f proving-service.yaml -n proving-system
```
Always use -n proving-system (or the name that you chose) if you are working outside the default namespace.

## 9. Monitor Logs and Status
```bash
kubectl get pods -n proving-system
kubectl logs deployment/camunda-service -n proving-system
kubectl logs deployment/sensor-data-service -n proving-system
kubectl logs deployment/proving-service -n proving-system
```

## 10. Update Docker Images After Code Changes

If you change the code in one of your services (e.g., the Camunda worker or sensor service), you must rebuild the corresponding Docker image.

Kubernetes uses the image specified in your deployment YAML, so the container will not reflect your code changes unless the image has been updated.

### To rebuild the image and apply changes in Minikube:

1. Re-enter the Minikube Docker context:
   ```bash
   eval $(minikube docker-env)
   ```
2. Rebuild your image(s), for example:
   ```bash
   docker build -t camunda-service:latest ./camunda-service
   ```
3. Restart the deployment to use the updated image:
   ```bash
   kubectl rollout restart deployment camunda-service -n proving-system
   ```
## 11. Delete deployments or services

```bash
kubectl delete deployment proving-service -n proving-system
kubectl delete service proving-service -n proving-system
```

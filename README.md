# Cluster setup instructions

## Initial Setup (run once)
### Prerequisites
- Docker
- Minikube
- kubectl
- Helm
- Git
- Bash shell

### 1. Clone this repository
```bash
git clone git@github.com:ACP-PCVCF/integration-repo.git
cd integration-repo
```

### 2. Run setup
This will start the cluster and install Camunda:
```bash
chmod +x setup.sh 
./setup.sh
```

## Deploy Services (repeatable)
After any changes to your services or code, simply run:

```bash
chmod +x setup.sh 
./deploy.sh
```
This script:
- Starts Minikube (if not running)
- Switches to the correct Docker context
- Builds the Docker images
- Applies the Kubernetes manifests

## Git Subtrees – How We Use Them
This repository integrates multiple service repositories using Git Subtrees.

Each service (i.e., sensor-data-service, camunda-service, and proofing-service) lives in its own dedicated Git repository, but is pulled into this integration repository via subtree under its respective folder.
This allows us to deploy and test all services together without changing how each service is developed.

### Developer Workflow
If you're working on one of the individual services:
1. Keep working in the original service repository as usual.
2. Commit and push your changes to the service's main branch.

### Updating the integration repo
After changes have been pushed to a service repository, someone (usually the integrator) will pull the updates into this integration repository using:
```bash
git fetch sensor-data-service
git subtree pull --prefix=sensor-data-service sensor-data-service main --squash

git fetch camunda-service
git subtree pull --prefix=camunda-service camunda-service main --squash

git fetch proofing-service
git subtree pull --prefix=proofing-service proofing-service main --squash
```
Repeat as needed for the services you want to update.

This keeps the integration repository up to date with the latest service code, and ready for deployment and testing.

### Different branch versions
Additionally, since subtrees reference a specific branch of the original service repository, you can choose which branch to track for each service.

For example, the integration repository may pull from the main branch of sensor-data-service, but from a develop branch of proofing-service, depending on your integration or staging needs:

```bash
git subtree pull --prefix=sensor-data-service sensor-data-service main --squash
git subtree pull --prefix=proofing-service proofing-service develop --squash
```
Please use different branches in this integration repository if you need additional branch combinations that don't involve all main branches.

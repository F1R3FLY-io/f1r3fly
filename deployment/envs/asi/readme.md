# Overview
This document describes and contains the deployment for ASI:Create project

# Installion Helm chart with custom values
## DEV
### Deploy
```sh
cd deployment/envs/asi || true
helm upgrade --install f1r3fly-asi-dev ../../../docker/helm/f1r3fly -n f1r3fly-asi-dev --create-namespace -f ./helm-values.yaml
```
### Check logs
#### Bootstrap node
```sh
# Check pod creation
kubectl describe pod f1r3fly-asi-dev0-0 -n f1r3fly-asi-dev
```
```sh
kubectl logs f1r3fly-asi-dev0-0 -n f1r3fly-asi-dev | tail -n 50
```
#### Observer node
```sh
# Check pod creation
kubectl describe pod f1r3fly-asi-dev1-0 -n f1r3fly-asi-dev
```
```sh
kubectl logs f1r3fly-asi-dev1-0 -n f1r3fly-asi-dev | tail -n 50
```

### Undeploy
```sh
helm uninstall f1r3fly-asi-dev -n f1r3fly-asi-dev
```
## STG
### Deploy
```sh
cd deployment/envs/asi || true
helm upgrade --install f1r3fly-asi-stg ../../../docker/helm/f1r3fly -n f1r3fly-asi-stg --create-namespace -f ./helm-values.yaml
```
### Undeploy
```sh
helm uninstall f1r3fly-asi-stg -n f1r3fly-asi-stg
```
## PRD
### Deploy
```sh
cd deployment/envs/asi || true
helm upgrade --install f1r3fly-asi-prd ../../../docker/helm/f1r3fly -n f1r3fly-asi-prd --create-namespace -f ./helm-values.yaml
```
### Undeploy
```sh
helm uninstall f1r3fly-asi-prd -n f1r3fly-asi-prd
```






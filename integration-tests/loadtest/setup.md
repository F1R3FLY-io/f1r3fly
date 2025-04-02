# Commands

Helm install:

```sh
cd integration-tests/loadtest || true
helm upgrade --install f1r3fly-loadtest ../../docker/helm/f1r3fly --namespace loadtest --create-namespace --values loadtest-values.yaml
```

if local:
```sh
cd integration-tests/loadtest || true
helm upgrade --install f1r3fly-loadtest ../../docker/helm/f1r3fly --namespace loadtest --create-namespace --values ../../docker/minikube/minikube-values.yaml --values loadtest-values.yaml
```

## Expose
```sh
echo "expose boot"
kubectl port-forward svc/f1r3fly-loadtest0 -n loadtest 40403:40403 &
kubectl port-forward svc/f1r3fly-loadtest0 -n loadtest 40405:40405 &
echo "expose readonly"
kubectl port-forward svc/f1r3fly-loadtest1 -n loadtest 40413:40403 &
kubectl port-forward svc/f1r3fly-loadtest1 -n loadtest 40415:40405 &
```

### Expose dashboard
```sh
kubectl port-forward svc/kubernetes-dashboard -n kubernetes-dashboard 8443:443
```
Create a token for view account
```sh
kubectl -n kubernetes-dashboard create token kube-ds-viewer
```
View pods at https://localhost:8443/#/pod?namespace=loadtest

### stop expose
```sh
ps aux | grep port-forward | grep -v grep | awk '{print $2}' | xargs kill
```

Uninstall:
```sh
helm uninstall f1r3fly-loadtest -n loadtest
```
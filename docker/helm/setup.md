# Helm handbook

Helm Chart for deploying F1r3fly nodes. Currently Helm Chart generate separate StatefulSets for each node. It supports up to 1000 nodes (limited by Service template now. It can be fixed if we can use 50000-60000 NodePorts or others)

## Installing Helm chart
- boot plus 8 validators:
```sh
helm upgrade --install f1r3fly-8nodes ./f1r3fly -n f1r3fly-8nodes --create-namespace
```
- boot plus 32 validators:
```sh
helm upgrade --install f1r3fly-32nodes ./f1r3fly -n f1r3fly-32nodes --create-namespace --set replicaCount=33
```

- boot plus 4 validators
```sh
helm upgrade --install f1r3fly-4nodes ./f1r3fly -n f1r3fly-4nodes --create-namespace --set replicaCount=5
```

## Uninstalling Helm chart
- 4 nodes
```sh
helm uninstall f1r3fly-4nodes -n f1r3fly-4nodes
kubectl delete ns f1r3fly-4nodes
```

- 8 nodes
```sh
helm uninstall f1r3fly-8nodes -n f1r3fly-8nodes
kubectl delete ns f1r3fly-8nodes
```

- 32 nodes
```sh
helm uninstall f1r3fly-32nodes -n f1r3fly-32nodes
kubectl delete ns f1r3fly-32nodes
```

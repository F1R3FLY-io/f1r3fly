# Helm handbook

Helm Chart for deploying F1r3fly nodes. Currently Helm Chart generate separate StatefulSets for each node. It supports up to 1000 nodes (limited by Service template now. It can be fixed if we can use 50000-60000 NodePorts or others)

## Installing Helm chart
- 8 nodes:
```sh
helm upgrade --install f1r3fly-8shard ./f1r3fly -n f1r3fly-8shard --create-namespace
```
- 32 nodes:
```sh
helm upgrade --install f1r3fly-32shard ./f1r3fly -n f1r3fly-32shard --create-namespace --set replicaCount=32
```

## Uninstalling Helm chart
- 8 nodes
```sh
helm uninstall f1r3fly-8shard -n f1r3fly-8node
kubectl delete ns f1r3fly-8node
```
- 32 nodes
```sh
helm uninstall f1r3fly-32shard -n f1r3fly-32shard
kubectl delete ns f1r3fly-32shard
```

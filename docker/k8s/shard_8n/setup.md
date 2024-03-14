# Kubernetes deployment

## Namespace
Create a separate namespace
```sh
kubectl create ns f1r3fly-8n

# and switch kubectl context to the new namespace
kubectl config set-context --namespace f1r3fly-8n --current
# now no need to add `-n f1r3fly-8n` to each command
```

## Deployment
Create all resources
```sh
kubectl apply -f .
```

## Redeploy
Delete and create resources back
```sh
kubectl delete -f . && kubectl apply -f .
```
If ConfigMap changed only, Pod won't be restarted. Pod will be restarted if StatufulSet changed. Delete Pod if configuration changed.
```sh
kubectl delete po validator1-0 # k8s will re-create a pod and pod will re-read configuration files
```

## Undeploy
Delete namespace and k8s will delete all resources
```sh
kubectl delete ns f1r3fly-8n
```

## Usefull links/tools
- [kubectl cheat sheet](https://kubernetes.io/docs/reference/kubectl/quick-reference/)
- [Autocompletion and shortcuts for ZSH](https://github.com/ohmyzsh/ohmyzsh/tree/master/plugins/kubectl)

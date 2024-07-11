# Kubernetes deployment

First, complete steps to create minikube instance: [minikube local](../minikube/setup.md)

## Namespace
Create a separate namespace
```sh
kubectl create ns f1r3fly-rspaceplusplus

# and switch kubectl context to the new namespace
kubectl config set-context --namespace f1r3fly-rspaceplusplus --current
# now no need to add `-n rspaceplusplus` to each command
```

## Deployment
Create all resources
```sh
kubectl apply -f .
```

Check status of pods
```
kubectl get pods -A
```

## Communication

Get list of services
```
kubectl get svc -n f1r3fly-rspaceplusplus
```

Get endpoint URL for services (leave this running)
```
minikube service bootstrap --namespace=f1r3fly-rspaceplusplus --url
```

In new terminal, test deploy with previously listed host and port (using second listed host + port):
```
grpcurl -plaintext  -d '{"deployer":"BChaWFkQyEoVg1gVvpccZyX07jjo2U741fCFbqmbn2wHdTNnN1eDSKEr4qqfJ0u6oJ9+V3INID3MTmc3RQ7aDcg=","term":"new return(`rho:rchain:deployId`) in { return!((42, true, \"Hello from blockchain!\")) }","sig":"MEQCIBPMx5E+15R+UM9ZV3STGtYegpwGmaizAtq8QtQshZK9AiBvShfjac6p3INXp4j6t9Nl66OrfK5KjRAcd+J4EmJdmA==","sigAlgorithm":"secp256k1","phloPrice":"1","phloLimit":"500000","shardId":"root"}' --import-path ./node/target/protobuf_external --import-path ./models/src/main/protobuf --proto DeployServiceV1.proto <host>:<port> casper.v1.DeployService.doDeploy
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
kubectl delete ns f1r3fly-rspaceplusplus
```

## Usefull links/tools
- [kubectl cheat sheet](https://kubernetes.io/docs/reference/kubectl/quick-reference/)
- [Autocompletion and shortcuts for ZSH](https://github.com/ohmyzsh/ohmyzsh/tree/master/plugins/kubectl)

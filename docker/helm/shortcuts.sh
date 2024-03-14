#!/bin/sh

# TODO: delete after tests

NAMESPACE=test

install () {
    helm upgrade --install f1r3fly f1r3fly -n $NAMESPACE --create-namespace
}

delete () {
    helm uninstall f1r3fly -n $NAMESPACE
}

reinstall () {
    delete
    install
}

template () {
    helm template f1r3fly -n $NAMESPACE > rendered.yaml
}

get () {
    kubectl get all -n $NAMESPACE
}

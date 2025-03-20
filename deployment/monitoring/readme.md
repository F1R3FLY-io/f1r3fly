# Installing Grafana and Prometheus

## Prerequisites

- A running F1r3fly cluster
- A running Oracle Cloud Infrastructure (OCI) cluster

## Installation

1. Install the Prometheus and Grafana Helm charts

```bash
helm repo add prometheus-community https://prometheus-community.github.io/helm-charts
helm repo add grafana https://grafana.github.io/helm-charts 

helm repo update

helm upgrade --install prometheus prometheus-community/prometheus --namespace monitoring --create-namespace \
--set prometheus.prometheusSpec.serviceMonitorSelectorNilUsesHelmValues=false \
--set prometheus.prometheusSpec.serviceMonitorNamespaceSelector.matchLabels.monitoring=true \
--set prometheus.prometheusSpec.serviceMonitorNamespaceSelector.matchExpressions\[0\].key=monitoring \
--set prometheus.prometheusSpec.serviceMonitorNamespaceSelector.matchExpressions\[0\].operator=Exists

# Install Prometheus Operator CRDs first
kubectl apply -f https://raw.githubusercontent.com/prometheus-operator/prometheus-operator/main/example/prometheus-operator-crd/monitoring.coreos.com_servicemonitors.yaml

# Create ClusterRole and ClusterRoleBinding for Prometheus to access ServiceMonitors in all namespaces
cat <<EOF | kubectl apply -f -
apiVersion: rbac.authorization.k8s.io/v1
kind: ClusterRole
metadata:
  name: prometheus-servicemonitor
rules:
- apiGroups: ["monitoring.coreos.com"]
  resources: ["servicemonitors"]
  verbs: ["get", "list", "watch"]
---
apiVersion: rbac.authorization.k8s.io/v1
kind: ClusterRoleBinding
metadata:
  name: prometheus-servicemonitor
roleRef:
  apiGroup: rbac.authorization.k8s.io
  kind: ClusterRole
  name: prometheus-servicemonitor
subjects:
- kind: ServiceAccount
  name: prometheus-server
  namespace: monitoring
EOF

helm install grafana grafana/grafana --namespace monitoring --create-namespace
```

2. Expose Prometheus and Grafana to the public

```bash
# Expose Prometheus
kubectl expose service prometheus-server --type=NodePort --target-port=9090 --name=prometheus-server-ext --namespace monitoring

# Expose Grafana
kubectl expose service grafana --type=NodePort --target-port=3000 --name=grafana-ext --namespace monitoring
```

2.1 Expose using minikube


```bash
minikube service grafana-ext --url -n monitoring
```

```bash
minikube service prometheus-server --url -n monitoring
```

3. Get the Grafana `admin` password

```bash
kubectl get secret --namespace monitoring grafana -o jsonpath="{.data.admin-password}" | base64 --decode
```

4. Add Prometheus as a data source in Grafana
Use the following configuration:
- URL: `http://prometheus-server-ext.monitoring.svc.cluster.local`

5. Configure Prometheus to scrape F1R3FLY metrics

Create a ServiceMonitor for F1R3FLY:
```bash
cat <<EOF | kubectl apply -f -
apiVersion: monitoring.coreos.com/v1
kind: ServiceMonitor
metadata:
  name: f1r3fly-metrics
  namespace: monitoring
  labels:
    release: prometheus
spec:
  selector:
    matchLabels:
      app.kubernetes.io/name: f1r3fly
  namespaceSelector:
    matchNames:
      - f1r3fly
  endpoints:
    - port: 40403-0  # Ensure this matches the correct port name
      path: /metrics
      interval: 15s
      scheme: http
EOF
```
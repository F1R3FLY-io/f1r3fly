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

# Install Prometheus with configuration to discover ServiceMonitors in all namespaces
helm upgrade --install prometheus prometheus-community/prometheus \
  --namespace monitoring \
  --create-namespace \
  --set prometheus.prometheusSpec.serviceMonitorSelectorNilUsesHelmValues=false \
  --set prometheus.prometheusSpec.serviceMonitorSelector={} \
  --set prometheus.prometheusSpec.serviceMonitorNamespaceSelector={} \
  --set 'prometheus-node-exporter.extraArgs[0]=--collector.cpu' \
  --set kubelet.serviceMonitor.https=true \
  --set kubelet.serviceMonitor.cAdvisor=true \
  --set 'kubelet.serviceMonitor.extraArgs[0]="--collector.cpu"' \
  --set 'kubelet.serviceMonitor.extraArgs[1]="--collector.cpu.disable-average-cpu=true"'

helm install grafana grafana/grafana --namespace monitoring --create-namespace
```

2. Expose Prometheus and Grafana
```bash
# Expose Prometheus
kubectl expose service prometheus-server --type=NodePort --target-port=9090 --name=prometheus-server-ext --namespace monitoring

# Expose Grafana
kubectl expose service grafana --type=NodePort --target-port=3000 --name=grafana-ext --namespace monitoring
```
  a. Port forward from minkube to localhost
```bash
minikube service grafana-ext --url -n monitoring
```
```bash
minikube service prometheus-server-ext --url -n monitoring
```
  b. Port forwarding from remote cloud to localhost
```sh
# Port forward Prometheus
kubectl port-forward svc/prometheus-server-ext -n monitoring 9090:80
```
```sh
# Port forward Grafana
kubectl port-forward svc/grafana-ext -n monitoring 3000:80
```
3. Get the Grafana `admin` password

```bash
kubectl get secret --namespace monitoring grafana -o jsonpath="{.data.admin-password}" | base64 --decode 
```

4. Add Prometheus as a data source in Grafana
   1. Log in
   2. Go to **Connections** tab
   3. Search **prometheus**
   4. Click **Add new data source**
   5. Set URL: `http://prometheus-server-ext.monitoring.svc.cluster.local`
   6. Click **Save and Test**

5. Import CPU/Memory Usage Dashboard
   1. In Grafana, click on the "+" icon in the left sidebar
   2. Select "Import dashboard"
   3. Enter Dashboard ID: `15055`
   4. Click "Load"
   5. Select your Prometheus data source from the dropdown
   6. Click "Import"

   Note: If importing by ID fails, you can:
   1. Visit [Monitor Pod CPU and Memory usage dashboard](https://grafana.com/grafana/dashboards/15055-monitor-pod-cpu-and-memory-usage/)
   2. Click "Download JSON"
   3. In Grafana, click "Upload JSON file" instead of entering the ID
   4. Select the downloaded JSON file
   5. Select your Prometheus data source
   6. Click "Import"

This will import the [Monitor Pod CPU and Memory usage](https://grafana.com/grafana/dashboards/15055-monitor-pod-cpu-and-memory-usage/) dashboard that provides detailed metrics for:
- Pod CPU usage
- Pod Memory usage
- Resource allocation and consumption

# Uninstall
```sh
# Uninstall Prometheus
helm uninstall prometheus -n monitoring

# Uninstall Grafana
helm uninstall grafana -n monitoring

# Delete the monitoring namespace (optional)
kubectl delete namespace monitoring
```
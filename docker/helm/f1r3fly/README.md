# F1R3FLY Helm Chart

This [Helm](https://helm.sh/) chart deploys F1R3FLY nodes in a Kubernetes environment. It supports both deployable nodes (validators) and read-only observer nodes.

## Overview

The chart deploys:
- Bootstrap node (first deployable node)
- Additional validator nodes (optional)
- Observer nodes (optional)

## Configuration

### Main Configuration Sections

#### Shard Configuration (`shardConfig`)
```yaml
shardConfig:
  # Number of deployable nodes (validators)
  deployableReplicas: 1

  # Number of read-only observer nodes
  readOnlyReplicas: 0

  # Network settings
  network:
    id: "testnet"  # Network identifier
    faultToleranceThreshold: 0.0  # Block finalization threshold

  # Synchrony constraint threshold for block validation
  syncConstraintThreshold: 0.34

  # Bootstrap node configuration
  bootstrap:
    nodeId: "..."  # Bootstrap node ID
    tls:
      nodeKeyPath: "configs/tls/node.key.pem"
      nodeCertificatePath: "configs/tls/node.certificate.pem"
```

#### Node Keys (`nodeKeys`)
```yaml
nodeKeys:
  - publicKey: "..."  # Validator public key
    privateKey: "..."  # Validator private key
```

### Environment-Specific Configuration

#### Test Environment
```yaml
# Test environment configuration
shardConfig:
  deployableReplicas: 1  # Single node setup
  readOnlyReplicas: 0
  network:
    id: "testnet"
    faultToleranceThreshold: 0.0
  syncConstraintThreshold: 0.34
  bootstrap:
    nodeId: "138410b5da898936ec1dc13fafd4893950eb191b"  # Test bootstrap node ID
    tls:
      nodeKeyPath: "configs/tls/node.key.pem"
      nodeCertificatePath: "configs/tls/node.certificate.pem"

# Include test validator keys
nodeKeys:
  - publicKey: "04ffc016579a68050d655d55df4e09f04605164543e257c8e6df10361e6068a5336588e9b355ea859c5ab4285a5ef0efdf62bc28b80320ce99e26bb1607b3ad93d"
    privateKey: "5f668a7ee96d944a4494cc947e4005e172d7ab3461ee5538f1f2a45a835e9657"

# Use test image
image:
  repository: "us-sanjose-1.ocir.io/axd0qezqa9z3/f1r3fly-io/rnode"
  tag: "latest"
```

#### Production Environment
```yaml
# Production environment configuration
shardConfig:
  deployableReplicas: 4  # Multiple validators for redundancy
  readOnlyReplicas: 2    # Observer nodes for monitoring
  network:
    id: "mainnet"
    faultToleranceThreshold: 0.1  # Stricter finalization threshold
  syncConstraintThreshold: 0.34
  bootstrap:
    nodeId: "YOUR_PRODUCTION_BOOTSTRAP_NODE_ID"  # Must be overridden
    tls:
      nodeKeyPath: "/etc/rnode/tls/node.key.pem"  # Must be overridden
      nodeCertificatePath: "/etc/rnode/tls/node.certificate.pem"  # Must be overridden

# Production validator keys (must be overridden)
nodeKeys:
  - publicKey: "YOUR_PRODUCTION_VALIDATOR_PUBLIC_KEY"  # Must be overridden
    privateKey: "YOUR_PRODUCTION_VALIDATOR_PRIVATE_KEY"  # Must be overridden

# Use production image
image:
  repository: "us-sanjose-1.ocir.io/axd0qezqa9z3/f1r3fly-io/rnode"
  tag: "prod"

# Resource limits for production
resources:
  limits:
    cpu: 2
    memory: 4G
```

### Configuration Override Examples

#### Using Custom Values File
1. Create a custom values file (e.g., `production-values.yaml`):
```yaml
# Override required production values
shardConfig:
  bootstrap:
    nodeId: "YOUR_PRODUCTION_BOOTSTRAP_NODE_ID"
    tls:
      nodeKeyPath: "/etc/rnode/tls/node.key.pem"
      nodeCertificatePath: "/etc/rnode/tls/node.certificate.pem"

nodeKeys:
  - publicKey: "YOUR_PRODUCTION_VALIDATOR_PUBLIC_KEY"
    privateKey: "YOUR_PRODUCTION_VALIDATOR_PRIVATE_KEY"

# Optional: Override other production-specific values
shardConfig:
  deployableReplicas: 4
  readOnlyReplicas: 2
  network:
    id: "mainnet"
    faultToleranceThreshold: 0.1

image:
  tag: "prod"
```

2. Install/upgrade using the custom values:
```bash
# Install with custom values
helm install f1r3fly ./f1r3fly -f production-values.yaml

# Upgrade with custom values
helm upgrade f1r3fly ./f1r3fly -f production-values.yaml
```

#### Using External Secrets
For production environments, it's recommended to use Kubernetes secrets for sensitive data:

1. Create a secret with bootstrap TLS certificates and keys:
```bash
kubectl create secret generic f1r3fly-bootstrap-tls \
  --from-file=node.key.pem=/path/to/node.key.pem \
  --from-file=node.certificate.pem=/path/to/node.certificate.pem
```

2. Reference the secret in your values file:
```yaml
shardConfig:
  bootstrap:
    tls:
      nodeKeyPath: "/etc/rnode/tls/node.key.pem"
      nodeCertificatePath: "/etc/rnode/tls/node.certificate.pem"
      secret:
        create: false  # Set to false if you've created the secret manually
        name: "f1r3fly-bootstrap-tls"  # Name of your secret (configurable)

# Note: Validator keys are currently configured directly in values.yaml
# Support for validator keys as secrets will be added in future releases
nodeKeys:
  - publicKey: "YOUR_PRODUCTION_VALIDATOR_PUBLIC_KEY"
    privateKey: "YOUR_PRODUCTION_VALIDATOR_PRIVATE_KEY"
```

Note: The bootstrap TLS secret name is configurable in `values.yaml` under `shardConfig.bootstrap.tls.secret.name`.

### Important Notes

1. **Security**:
   - In production, validator private keys should be stored in Kubernetes secrets (not yet supported)
   - TLS certificates should be properly managed and rotated
   - Consider using a secure key management service

2. **Resource Requirements**:
   - All node types (bootstrap, validator, and observer) require the same resources
   - Minimum requirements per node:
     - CPU: 1 core
     - Memory: 2GB
     - Disk: 256GB
   - Target requirements per node:
     - CPU: 1 core
     - Memory: 4GB
     - Disk: 1TB
   - These requirements are consistent across all environments

3. **Network Configuration**:
   - Default ports:
     - Protocol server: 40400
     - External gRPC: 40401
     - Internal gRPC: 40402
     - HTTP API: 40403
     - Kademlia: 40404
     - Admin HTTP: 40405
   - Port Mapping:
     - Each rnode pod port is mapped to a Kubernetes nodeport
     - Mapping is configured in `values.yaml` under `service.ports`
     - Implementation is handled in `services.yaml` template
     - Example mapping:
       ```yaml
       service:
         ports:
         - containerPort: 40400  # rnode port
           nodePort: 30000       # Kubernetes nodeport
         - containerPort: 40401
           nodePort: 30001
         # ... other port mappings
       ```
     - For multiple replicas, nodeports are automatically generated using pattern:
       - First digit from port + replica index + last digit from container port
       - Example: Port 40402 for replica 0 becomes 30002
       - Supports up to 1000 replicas
   - Ingress Configuration (Recommended for Multi-Node Clusters):
     - For multi-node Kubernetes clusters, it's recommended to use Ingress instead of NodePorts
     - This allows direct port mapping (e.g., 40403 -> 40403) and better domain management
     - Example configuration:
       ```yaml
       ingress:
         enabled: true
         className: "nginx"  # or your ingress controller class
         hosts:
           - host: rnode1.testnet.f1r3fly.io
             paths:
               - path: /
                 pathType: Prefix
                 service: f1r3fly0  # First rnode service
                 port: 40403        # External gRPC port
           - host: rnode2.testnet.f1r3fly.io
             paths:
               - path: /
                 pathType: Prefix
                 service: f1r3fly1  # Second rnode service
                 port: 40403
       ```
     - Benefits:
       - Clean port mapping (no need for port translation)
       - Better domain management for each node
       - Improved security through TLS termination
       - Easier load balancing and routing
     - Requirements:
       - Ingress controller installed in the cluster
       - DNS records configured for each node
       - TLS certificates for secure communication

4. **Storage**:
   - Uses persistent volumes for node data
   - Storage class: `oci` (configurable)

## Usage

1. Install the chart:
```bash
helm install f1r3fly ./f1r3fly
```

2. Upgrade the deployment:
```bash
helm upgrade f1r3fly ./f1r3fly
```

3. Uninstall the chart:
```bash
helm uninstall f1r3fly
```

## Dependencies

- Kubernetes 1.19+
- Helm 3.0+
- OCI-compatible container registry
- Persistent volume support 
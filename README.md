# F1R3FLY

Highly concurrent throughput Byzantine Fault Tolerants

## 🚀 Quick Start

```bash
# Install dependencies
sbt compile

# Start development
docker compose -f docker/shard.yml up

# Build project
sbt ";compile ;project node ;Docker/publishLocal ;project rchain"

# Run tests
./scripts/run_rust_tests.sh
```

### With Nix/Direnv (Recommended)

```bash
# Enter development shell with all dependencies
direnv allow

# Or manually with nix
nix develop

# Dependencies and environment are automatically configured
sbt compile
```

## 📚 Documentation-First Approach

This project follows a documentation-first methodology optimized for both human developers and LLM-assisted development. All features begin with documentation, ensuring clear requirements before implementation.

### Core Documentation Structure

- **[📋 Requirements](docs/requirements)** - User stories, business requirements, and acceptance criteria
  - `user-stories/` - Feature requirements from user perspective
  - `business-requirements/` - Business logic and constraints
  - `acceptance-criteria/` - Definition of done for features

- **[📐 Specifications](docs/specifications)** - Technical specifications and design documents
  - `visual-design/` - UI/UX mockups, wireframes, and style guides
  - `technical/` - API specifications, data schemas, and algorithms
  - `integration/` - Third-party service integration specs

- **[🏗️ Architecture](docs/architecture)** - System design and architectural decisions
  - `decisions/` - Architecture Decision Records (ADRs)
  - `diagrams/` - System component diagrams and data flows
  - `patterns/` - Established patterns and conventions

- **[Current Status](docs/ToDos.md)** - Live project status, active tasks, and priorities

### For Contributors

- **[Contributing Guide](CONTRIBUTING.md)** - Complete workflow for development
- **[Development Setup](DEVELOPER.md)** - Environment configuration
- **[Nix/Direnv Setup](#installation)** - Reproducible development environments
- **[Testing Guide](#testing)** - Testing strategies and conventions
- **[API Documentation](docs/api)** - API reference and examples

### For LLM-Assisted Development

When using AI coding assistants (Claude, GitHub Copilot, etc.), provide context from:
- **Project Context**: `CLAUDE.md` (LLM-specific instructions)
- **Requirements**: Relevant files from `docs/requirements/`
- **Specifications**: Technical specs from `docs/specifications/`
- **Architecture**: Constraints from `docs/architecture/`
- **Current Tasks**: Priorities from `docs/ToDos.md`

## 🗂️ Project Structure

```
f1r3fly/
├── docs/                  # Documentation hierarchy
│   ├── requirements/      # Business and user requirements
│   ├── specifications/    # Technical specifications
│   ├── architecture/      # System design documents
│   ├── api/              # API documentation
│   └── ToDos.md          # Current status and tasks
├── node/                 # Scala node implementation
├── rust_libraries/       # Rust components
├── rholang/             # Rholang examples and tests
├── node-cli/            # Command-line interface
├── scripts/             # Build and utility scripts
├── .github/             # GitHub configuration
│   └── workflows/       # CI/CD pipelines
├── CLAUDE.md            # LLM assistant context
└── README.md            # This file
```

## 🔄 Development Workflow

1. **📖 Documentation First**
   - Start with requirements in `docs/requirements/`
   - Create/update technical specs in `docs/specifications/`
   - Document architectural decisions in `docs/architecture/decisions/`

2. **🤖 LLM Integration**
   - Provide comprehensive context from documentation
   - Reference `CLAUDE.md` for project-specific instructions
   - Update documentation alongside code changes

3. **⚙️ Development Standards**
   - Use Nix/Direnv for consistent development environments
   - Follow test-driven development (TDD) practices
   - Maintain code coverage targets
   - Use conventional commits for version control
   - Implement CI/CD checks before merging

4. **📝 Continuous Documentation**
   - Keep `docs/ToDos.md` updated with current status
   - Update relevant documentation with each PR
   - Maintain README files at directory levels for complex modules

## 🛠️ Technical Stack

- **Languages**: Scala, Rust, Rholang
- **Frameworks**: Akka, ScalaTest
- **Testing**: ScalaTest, Rust testing framework
- **Build Tools**: SBT, Cargo, Docker
- **Package Manager**: SBT for Scala, Cargo for Rust
- **Version Control**: Git with feature branching
- **Development Environment**: Nix flakes + Direnv for reproducible environments

## What is F1R3FLY?

F1R3FLY is building a decentralized, economic, censorship-resistant, public compute infrastructure and blockchain. It will host and execute programs popularly referred to as "smart contracts". It will be trustworthy, scalable, concurrent, with proof-of-stake consensus and content delivery.

[F1R3FLY Discord](https://discord.gg/NN59aFdAHM) features project-related tutorials, documentation, project planning information, events calendar, and information for how to engage with this project.

## Installation

### Prerequisites

1. **Install Nix**: https://nixos.org/download/
   - For more information about Nix and how it works see: https://nixos.org/guides/how-nix-works/

2. **Install direnv**: https://direnv.net/#basic-installation
   - For more information about direnv and how it works see: https://direnv.net/

3. **Clone and Setup**:
   ```bash
   git clone https://github.com/F1R3FLY-io/f1r3fly.git
   cd f1r3fly
   direnv allow
   ```
   
   If you encounter the error: `error: experimental Nix feature 'nix-command' is disabled`:
   - Create the file: `~/.config/nix/nix.conf`
   - Add the line: `experimental-features = flakes nix-command`
   - Run `direnv allow` again

### Docker

```bash
docker pull f1r3flyindustries/f1r3fly-rust-node
```

See https://hub.docker.com/r/f1r3flyindustries/f1r3fly-rust-node for more information.

### Platform Packages

- **Debian/Ubuntu**: Coming Soon
- **RedHat/Fedora**: Coming Soon
- **macOS**: Coming Soon

## Building

Prerequisites: [Environment set up](#installation).

```bash
# Compile and create Docker image
docker context use default && sbt ";compile ;project node ;Docker/publishLocal ;project rchain"

# Compile and create fat jar (for local execution)
sbt ";compile ;project node ;assembly ;project rchain"

# Clean the project
sbt "clean"
```

It is recommended to have a terminal window open just for `sbt` to run various commands.

## Running

### Docker

```bash
# Start a shard
docker compose -f docker/shard.yml up

# Delete data directory for fresh start (performs genesis ceremony)
./scripts/delete_data.sh
```

### Local

```bash
# Run standalone node locally
java -Djna.library.path=./rust_libraries/release \
  --add-opens java.base/sun.security.util=ALL-UNNAMED \
  --add-opens java.base/java.nio=ALL-UNNAMED \
  --add-opens java.base/sun.nio.ch=ALL-UNNAMED \
  -jar node/target/scala-2.12/rnode-assembly-1.0.0-SNAPSHOT.jar run \
  -s --no-upnp --allow-private-addresses --synchrony-constraint-threshold=0.0

# Delete data directory for fresh start
rm -rf ~/.rnode/
```

## Usage

### Node CLI

A command-line interface for interacting with F1R3FLY nodes is available in the `node-cli` directory. Features include:

- **Deploying** Rholang code to F1R3FLY nodes
- **Proposing** blocks to create a new block containing deployed code
- **Full Deploy** operations (deploy + propose in one step)
- **Checking finalization** of blocks with automatic retries
- **Exploratory Deploy** to execute Rholang without committing to the blockchain
- **Generating Public Keys** from private keys
- **Generating Key Pairs** for creating new blockchain identities

For detailed usage instructions, see the [Node CLI README](node-cli/README.md).

### Evaluating Rholang Contracts

```bash
# Build node executable
sbt ";compile ;stage"

# Evaluate a contract
./node/target/universal/stage/bin/rnode \
  -Djna.library.path=./rust_libraries/release \
  eval ./rholang/examples/tut-ai.rho
```

### F1R3FlyFS

Check out [F1R3FlyFS](https://github.com/F1R3FLY-io/f1r3flyfs#f1r3flyfs) for a simple, easy-to-use, and fast file system built on top of F1R3FLY.

## 🧪 Testing

```bash
# Run all Rust tests
./scripts/run_rust_tests.sh

# Run Scala tests
sbt test

# Run specific test suites
sbt "project node" test
sbt "project rspace" test

# Coverage report (when available)
sbt coverage test coverageReport
```

## 🚢 Deployment

See [deployment documentation](docs/deployment) for detailed deployment instructions.

## Configuration

Most command line options can be specified in a configuration file. The default location is the data directory. An alternative location can be specified with `--config-file <path>`.

The format is [HOCON](https://github.com/lightbend/config/blob/master/HOCON.md). See [defaults.conf](node/src/main/resources/defaults.conf) for all options and default values.

Example configuration:
```hocon
standalone = false

protocol-server {
  network-id = "testnet"
  port = 40400
}

protocol-client {
  network-id = "testnet"
  bootstrap = "rnode://de6eed5d00cf080fc587eeb412cb31a75fd10358@52.119.8.109?protocol=40400&discovery=40404"
}

api-server {
  host = "my-rnode.domain.com"
  port-grpc-external = 40401
  port-grpc-internal = 40402
  port-http = 40403
  port-admin-http = 40405
}
```

## Troubleshooting

**General nix problems or unable to load `flake.nix`:**
```bash
nix-garbage-collect
```

**SBT build problems:**
```bash
rm -rf ~/.cache/coursier/
sbt clean
```

**StackOverflow error from node compile:**
```bash
sbt "node/compile"
```

**Rust problems:**
```bash
./scripts/clean_rust_libraries.sh
rustup default stable
```

## 🔐 Security

- Security policies and guidelines in `SECURITY.md`
- Vulnerability reporting procedures
- Security best practices for contributors

**Note**: This code has not yet completed a security review. We strongly recommend that you do not use it in production or to transfer items of material value. We take no responsibility for any loss you may incur through the use of this code.

## 📈 Performance

- Performance benchmarks and targets
- Optimization guidelines
- Monitoring and metrics

## 🤝 Contributing

Please read our [Contributing Guide](CONTRIBUTING.md) for details on:
- Code of conduct
- Development process
- Pull request process
- Coding standards
- Documentation requirements

## 📄 License

[![License: Apache 2.0](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](https://www.apache.org/licenses/LICENSE-2.0)

To get a summary of licenses being used by F1R3FLY's dependencies:
```bash
sbt node/dumpLicenseReport
```
The report will be available under `node/target/license-reports/rnode-licenses.html`

## 🙏 Acknowledgments

We use YourKit to profile RNode performance. YourKit supports open source projects with its full-featured Java Profiler. YourKit, LLC is the creator of [YourKit Java Profiler](https://www.yourkit.com/java/profiler/) and [YourKit .NET Profiler](https://www.yourkit.com/.net/profiler/), innovative and intelligent tools for profiling Java and .NET applications.

---

### 📦 Additional Resources

> **Note for F1R3FLY Projects**: This README follows our organization's commitment to documentation-first development and LLM-enhanced workflows. The core documentation structure supports systematic development and clear communication of project goals and progress.

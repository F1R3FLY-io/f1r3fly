# F1r3fly
[![Build Status](https://github.com/rchain/rchain/workflows/CI/badge.svg)](https://github.com/rchain/rchain/actions?query=workflow%3ACI+branch%3Astaging)
[![codecov](https://codecov.io/gh/rchain/rchain/branch/master/graph/badge.svg)](https://codecov.io/gh/rchain/rchain)

### [What is F1r3fly?]([#what-is-this-project?])
- A decentralized, economic, censorship-resistant, public compute infrastructure and blockchain.
- Hosts and executes programs popularly referred to as "smart contracts".
- Trustworthy, scalable, concurrent, with proof-of-stake consensus and content delivery.
### How to get F1r3fly
- Install locally using `nix` and `direnv` to setup a [development environment](#installation).
- Refer to the [F1r3fly Discord](https://discord.gg/NN59aFdAHM) for project-related tutorials, documentation, and information.
- Run the public testnet to explore F1r3fly's capabilities (Coming Soon).
<!-- ### [Installation instructions](#Installation)
- This version of F1r3fly can only be installed locally using `nix` and `direnv` to setup a development environment. -->
<!-- ### Running F1r3fly standalone or with multiple nodes
Connect multiple Docker or local RNodes to a user-defined network bridge for peer-to-peer communication and REPL capabilities.
### Using the REPL
- Invoke the REPL using the respective command and interact with F1r3fly using Rholang language.
- Validate F1r3fly's functionality by executing a command in the REPL and observing the output on the rnode0 (bootstrap) node.
### Peer node configuration
- Start a peer node using the respective command and provide the bootstrap address of rnode0.
- Observe the successful communication between the main node and the peer node in the output of both nodes.
### Obtaining command options
- Use the --help option to get a comprehensive list of command-line options for RNode. -->
<!-- ### Configuration file
- Specify RNode configuration parameters through a configuration file in HOCON format.
- View the [configuration file](node/src/main/resources/defaults.conf) for a detailed explanation of each parameter.
### Development environment
- Compile the F1r3fly project using the provided command.
- Run the compiled binary using the specified command.
### Developer resources
- Refer to the developer guide for more in-depth instructions and documentation. -->
### Known issues
- Be aware of the known issues listed in the GitHub issues.
### Reporting issues
- File any issues you encounter in the GitHub repository issue tracker.
### Acknowledgments
- Express gratitude to YourKit for their support of open-source projects.
### License information
<!-- - Generate a summary of licenses used by F1r3fly's dependencies using the provided command. -->


<!-- ## What is this project?


The open-source F1r3fly project is building a decentralized, economic,
censorship-resistant, public compute infrastructure and blockchain. It will
host and execute programs popularly referred to as “smart contracts”. It will
be trustworthy, scalable, concurrent, with proof-of-stake consensus and
content delivery.

[F1r3fly Discord](https://discord.gg/NN59aFdAHM) features project-related
tutorials and documentation, project planning information, events calendar,
and information for how to engage with this project. -->

## Note on the use of this software
This code has not yet completed a security review. We strongly recommend that you do not use it in production or to transfer items of material value. We take no responsibility for any loss you may incur through the use of this code.

## Installation

### Source

1. Install Nix: https://nixos.org/download/
   - For more information about Nix and how it works see: https://nixos.org/guides/how-nix-works/

2. Install direnv: https://direnv.net/#basic-installation
   - For more information about direnv and how it works see: https://direnv.net/

3. Clone this repository and after entering the repository, run `direnv allow`. There should be a message asking you to do this. 
   - You may run into the following error: `error: experimental Nix feature 'nix-command' is disabled; add '--extra-experimental-features nix-command' to enable it`. To fix this, first create the following file: `~/.config/nix/nix.conf`. Add the following line to the file you just created: `experimental-features = flakes nix-command`. Then run `direnv allow` again.
   - This will do a one-time compile of all our libraries which will take a couple of minutes. After completion, your environment will be setup.
   
### Docker

``docker pull f1r3flyindustries/f1r3fly-rust-node``

- Please see https://hub.docker.com/r/f1r3flyindustries/f1r3fly-rust-node for more information on how to run image from Docker Hub.

### Debian/Ubuntu

(Coming Soon)

### RedHat/Fedora

(Coming Soon)

### macOS

(Coming Soon)

## Building

Prerequisites: [Environment set up](#installation).

1. `docker context use default && sbt ";compile ;project node ;Docker/publishLocal ;project rchain"` will compile the project and create a docker image. 
2. `sbt ";compile ;project node ;assembly ;project rchain"` will compile the project and create a fat jar. You can use this to run locally without docker.
3. `sbt "clean"` will clean the project.

- It is recommended to have a terminal window open just for `sbt` to run various commands.

## Running

### Docker

Starting a shard after generating the docker image is as simple as:

```sh
docker compose -f docker/shard.yml up
```

- Running: `./scripts/delete_data.sh` will delete the data directory and ensure a fresh start (performs genesis ceremony).

### Local

To run a standalone node locally, you can use the following command after generating the fat jar:

```sh
java -Djna.library.path=./rust_libraries/release --add-opens java.base/sun.security.util=ALL-UNNAMED --add-opens java.base/java.nio=ALL-UNNAMED --add-opens java.base/sun.nio.ch=ALL-UNNAMED -jar node/target/scala-2.12/rnode-assembly-1.0.0-SNAPSHOT.jar run -s --no-upnp --allow-private-addresses --synchrony-constraint-threshold=0.0
```

- Running: `rm -rf ~/.rnode/` will delete the data directory and ensure a fresh start (performs genesis ceremony).

## Usage

### Node CLI

A command-line interface for interacting with F1r3fly nodes is available in the `node-cli` directory. This CLI provides functionality for:

- **Deploying** Rholang code to F1r3fly nodes
- **Proposing** blocks to create a new block containing deployed code
- **Full Deploy** operations (deploy + propose in one step)
- **Checking finalization** of blocks with automatic retries
- **Exploratory Deploy** to execute Rholang without committing to the blockchain (ideal for read-only nodes)
- **Generating Public Keys** from private keys for identity and signature verification
- **Generating Key Pairs** for creating new blockchain identities

For detailed usage instructions and examples, see the [Node CLI README](node-cli/README.md).

### Evaluating Rholang contracts

Prerequisites: [Running node](#running).

Build node into executable:

```sh
sbt ";compile ;stage"
```

Evaluate a contract:

```sh
./node/target/universal/stage/bin/rnode -Djna.library.path=./rust_libraries/release eval ./rholang/examples/tut-ai.rho
```

### F1r3flyFS

Check out the [F1r3flyFS](https://github.com/F1R3FLY-io/f1r3flyfs#f1r3flyfs) project for a simple, easy-to-use, and fast file system built on top of F1r3fly.

### Troubleshooting

General nix problems or unable to load `flake.nix` file:
```bash
nix-garbage-collect
```

SBT build problems:
```bash
$ rm -rf ~/.cache/coursier/

$ sbt clean
```

Rust problems: 
```bash
$ ./scripts/clean_rust_libraries.sh

$ rustup default stable
```



<!-- Docker will be used in the examples port portability reasons, but running the
node as a standalone process is very similar.

To fetch the latest version of RNode from the remote Docker hub and run it (exit with `C-c`):

```sh
$ docker run -it -p 40400:40400 rchain/rnode:latest

# With binding of RNode data directory to the host directory $HOME/rnode 
$ docker run -v $HOME/rnode:/var/lib/rnode -it -p 40400:40400 rchain/rnode:latest
```

In order to use both the peer-to-peer network and REPL capabilities of the
node, you need to run more than one Docker RNode on the same host, the
containers need to be connected to one user-defined network bridge:

```bash
$ docker network create rnode-net

$ docker run -dit --name rnode0 --network rnode-net rchain/rnode:latest run -s

$ docker ps
CONTAINER ID   IMAGE                 COMMAND                  CREATED          STATUS          PORTS     NAMES
ef770b4d4139   rchain/rnode:latest   "bin/rnode --profile…"   23 seconds ago   Up 22 seconds             rnode0
```

To attach terminal to RNode logstream execute

```bash
$ docker logs -f rnode0
[...]
08:38:11.460 [main] INFO  logger - Listening for traffic on rnode://137200d47b8bb0fff54a753aabddf9ee2bfea089@172.18.0.2?protocol=40400&discovery=40404
[...]
```

A repl instance can be invoked in a separate terminal using the following command:

```bash
$ docker run -it --rm --name rnode-repl --network rnode-net rchain/rnode:latest --grpc-host rnode0 --grpc-port 40402 repl

  ╦═╗┌─┐┬ ┬┌─┐┬┌┐┌  ╔╗╔┌─┐┌┬┐┌─┐  ╦═╗╔═╗╔═╗╦  
  ╠╦╝│  ├─┤├─┤││││  ║║║│ │ ││├┤   ╠╦╝║╣ ╠═╝║  
  ╩╚═└─┘┴ ┴┴ ┴┴┘└┘  ╝╚╝└─┘─┴┘└─┘  ╩╚═╚═╝╩  ╩═╝
    
rholang $
```

Type `@42!("Hello!")` in REPL console. This command should result in (`rnode0` output):
```bash
Evaluating:
@{42}!("Hello!")
```

A peer node can be started with the following command (note that `--bootstrap` takes the listening address of `rnode0`):

```bash
$ docker run -it --rm --name rnode1 --network rnode-net rchain/rnode:latest run --bootstrap 'rnode://8c775b2143b731a225f039838998ef0fac34ba25@rnode0?protocol=40400&discovery=40404' --allow-private-addresses --host rnode1
[...]
15:41:41.818 [INFO ] [node-runner-39      ] [coop.rchain.node.NodeRuntime ] - Starting node that will bootstrap from rnode://8c775b2143b731a225f039838998ef0fac34ba25@rnode0?protocol=40400&discovery=40404
15:57:37.021 [INFO ] [node-runner-32      ] [coop.rchain.comm.rp.Connect$ ] - Peers: 1
15:57:46.495 [INFO ] [node-runner-32      ] [c.r.c.util.comm.CommUtil$    ] - Successfully sent ApprovedBlockRequest to rnode://8c775b2143b731a225f039838998ef0fac34ba25@rnode0?protocol=40400&discovery=40404
15:57:50.463 [INFO ] [node-runner-40      ] [c.r.c.engine.Initializing    ] - Rholang state received and saved to store.
15:57:50.482 [INFO ] [node-runner-34      ] [c.r.casper.engine.Engine$    ] - Making a transition to Running state.
```

The above command should result in (`rnode0` output):
```bash
15:57:37.021 [INFO ] [node-runner-42      ] [c.r.comm.rp.HandleMessages$  ] - Responded to protocol handshake request from rnode://e80faf589973c2c1b9b8441790d34a9a0ffdd3ce@rnode1?protocol=40400&discovery=40404
15:57:37.023 [INFO ] [node-runner-42      ] [coop.rchain.comm.rp.Connect$ ] - Peers: 1
15:57:46.530 [INFO ] [node-runner-43      ] [c.r.casper.engine.Running$   ] - ApprovedBlock sent to rnode://e80faf589973c2c1b9b8441790d34a9a0ffdd3ce@rnode1?protocol=40400&discovery=40404
15:57:48.283 [INFO ] [node-runner-43      ] [c.r.casper.engine.Running$   ] - Store items sent to rnode://e80faf589973c2c1b9b8441790d34a9a0ffdd3ce@rnode1?protocol=40400&discovery=40404
``` -->

<!-- To get a full list of options rnode accepts, use the `--help` option:

```sh
$ docker run -it --rm rchain/rnode:latest --help
``` -->

### Configuration file

Most of the command line options can be specified in a configuration file.

The default location of the configuration file is the data directory. An
alternative location can be specified with the command line option `--config-file <path>`.

The format of the configuration file is [HOCON](https://github.com/lightbend/config/blob/master/HOCON.md).

The [defaults.conf](node/src/main/resources/defaults.conf) configuration file shows all options and default values.

Example configuration file:

```yml
standalone = false

protocol-server {
  network-id = "testnet"
  port = 40400
}

protocol-client {
  network-id = "testnet"
  bootstrap = "rnode://de6eed5d00cf080fc587eeb412cb31a75fd10358@52.119.8.109?protocol=40400&discovery=40404"
}

peers-discovery {
  port = 40404
}

api-server {
  host = "my-rnode.domain.com"
  port-grpc-external = 40401
  port-grpc-internal = 40402
  port-http = 40403
  port-admin-http = 40405
}

storage {
  data-dir = "/my-data-dir"
}

casper {
  fault-tolerance-threshold = 1
  shard-name = root
  finalization-rate = 1
}

metrics {
  prometheus = false
  influxdb = false
  influxdb-udp = false
  zipkin = false
  sigar = false
}

dev-mode = false
```

## Development

Compile the project with:

```bash
$ sbt clean compile

# With executable and Docker image
$ sbt clean compile stage docker:publishLocal
```

<!-- Run the resulting binary with:

```bash
$ ./node/target/universal/stage/bin/rnode
``` -->

### Testing 

Run all Rust tests: `./scripts/run_rust_tests.sh`

For more detailed instructions, see the [developer guide](DEVELOPER.md).

## Caveats and filing issues

### Caveats

During this pre-release phase of the F1r3fly software, there are some [known issues](https://github.com/rchain/rchain/issues?q=is%3Aopen+is%3Aissue+label%3Abug) and some [other issues](https://github.com/F1R3FLY-io/f1r3fly/issues).

### Filing Issues

File issues in GitHub repository issue tracker: [File a bug](https://github.com/F1R3FLY-io/f1r3fly/issues).

## Acknowledgements

We use YourKit to profile rchain performance.  YourKit supports open source
projects with its full-featured Java Profiler.  YourKit, LLC is the creator of
<a href="https://www.yourkit.com/java/profiler/">YourKit Java Profiler</a> and
<a href="https://www.yourkit.com/.net/profiler/">YourKit .NET Profiler</a>,
innovative and intelligent tools for profiling Java and .NET applications.

## Licence information

To get summary of licenses being used by the F1r3fly's dependencies, simply run
`sbt node/dumpLicenseReport`. The report will be available under
`node/target/license-reports/rnode-licenses.html`

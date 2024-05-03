# RSpace++

On branch `rhotypes` using `nix` and `direnv`

## Quickstart

- For setting up `nix` and `direnv`, see [project overview](../docs/paul_brain_dump.md)
- Make sure you have [protobuf](https://grpc.io/docs/protoc-installation/) installed

To get in and out of `direnv` you can use the following:
- `direnv allow` in root project directory
- `direnv revoke` then exit root project directory. Coming back into root project directory you will be out of nix shell

Starting standalone node using RSpace++
1. `sbt ";clean ;compile ;stage"`
2. `./node/target/universal/stage/bin/rnode -Djna.library.path=./rspace++/target/debug  run --standalone` in one terminal
3. In a another terminal, execute rholang: `./node/target/universal/stage/bin/rnode -Djna.library.path=./rspace++/target/debug eval rholang/examples/stdout.rho`

Standing up network using RSpace++
1. Follow these instructions on setting up `.rnode` directory [setting up rnode directory](../docs/paul_brain_dump.md#an-example-tying-the-above-together-hopefully) stopping just before you execute the java command that starts the node
2. `sbt ";clean ;compile ;project node ;assembly ;project rchain"`
3. `java -Djna.library.path=./rspace++/target/debug/ --add-opens java.base/sun.security.util=ALL-UNNAMED --add-opens java.base/java.nio=ALL-UNNAMED --add-opens java.base/sun.nio.ch=ALL-UNNAMED -jar node/target/scala-2.12/rnode-assembly-0.0.0-unknown.jar run -s --no-upnp --allow-private-addresses --synchrony-constraint-threshold=0.0 --validator-private-key <your_validator_key>`
4. (Optional) Run this command to ensure node performs genesis ceremony: `rm -rf ~/.rnode/casperbuffer/ ~/.rnode/dagstorage/ ~/.rnode/deploystorage/ ~/.rnode/blockstorage/ ~/.rnode/rnode.log ~/.rnode/rspace++/ ~/.rnode/node.certificate.pem ~/.rnode/node.key.pem`

Standing up network using RSpace++ (Under Docker) (Currently not working under Nix shell)
1. `sbt ";clean ;compile ;project node ;Docker/publishLocal ;project rchain"`
2. `docker compose -f docker/shard.yml up`

### Working Rholang Contracts using RSpace++ (under standalone node)

I have classified these as "working" if the output and deployment cost matches that of the old RSpace code.

- `block-data.rho`
- `dupe.rho`
- `hello_world_again.rho`
- `longfast.rho`
- `longslow.rho`
- `shortfast.rho`
- `shortslow.rho`
- `stderr.rho`
- `stderrAck.rho`
- `stdout.rho`
- `stdoutAck.rho`
- `tut-bytearray-methods.rho`
- `tut-hash-functions.rho`
- `tut-hello-again.rho`
- `tut-hello.rho`
- `tut-lists-methods.rho`
- `tut-maps-methods.rho`
- `tut-parens.rho`
- `tut-philosophers.rho`
- `tut-prime.rho`
- `tut-rcon-or.rho`
- `tut-rcon.rho`
- `tut-registry.rho` (Apparently this doesn't work with the old RSpace code)
- `tut-sets-methods.rho`
- `tut-strings-methods.rho`
- `tut-tuples-methods.rho`

### Testing Scala (using RSpace++)

- Run Rholang Reduce tests: `sbt "rholang/testOnly coop.rchain.rholang.interpreter.ReduceSpec"`
- Run basic RSpace-Bench Benchmark: `sbt "rspaceBench/jmh:run -i 10 -wi 10 -f1 -t1 .BasicBench."`
- Run Casper Genesis tests: `sbt "casper/testOnly coop.rchain.casper.genesis.GenesisTest"`

### Testing Rust (within rspace++ directory)

Run all tests: `cargo test`

- Run Spatial Matcher Tests: `cargo test matcher::match_test -- --test-threads=1`
- Run Storage Actions Tests: `cargo test --test storage_actions_test`
- Run History Action Tests: `cargo test history::history_action_tests`
- Run History Repository Tests: `cargo test history::history_repository_tests`

(`--test-threads=1` runs them sequentially)<br>
(`--nocapture` prints output during tests)

Run Scala tests for calling Rust functions (from root directory): `sbt "rspacePlusPlus/testOnly"`

## Scala and Rust Notes

- Using Scala: `jna`; Rust: `prost`, `heed`, `dashmap`, `blake3`, `serde`. See `Cargo.toml` for complete list of crates.

## Rust

- Within rspace++ directory, `cargo build --profile dev -p rspace_plus_plus_rhotypes` to build `rspace_plus_plus` library. Outputs to `rspace++/target/debug/`. Scala code pulls from here.


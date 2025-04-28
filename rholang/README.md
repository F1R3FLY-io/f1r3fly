# Rholang

Rholang is a concurrent programming language, with a focus on message-passing and formally modeled by the ρ-calculus, a reflective, higher-order extension of the π-calculus. It is designed to be used to implement protocols and "smart contracts" on a general-purpose blockchain, but could be used in other settings as well.

The language is still in the early stages of development. Currently we have a working interpreter for the language. It should be considered an early preview of the language.

This is a direct port of the `rholang` Scala library to Rust. The original Scala library code can be found [here](https://github.com/rchain/rchain/tree/dev/rholang).

## Development

To build the `rholang` Rust library, run `cargo build --release -p rholang`
  - `cargo build --profile dev -p rholang` will build the library in debug mode

### Testing

#### Rust

To run all tests: `cargo test`

Run all tests in release mode: `cargo test --release`

To run specific test file: `cargo test --test <test_file_name>`

To run specific test in specific folder: `cargo test --test <test_folder_name>::<test_file_name>`

#### Scala

The following tests all use `rholang-rust` and must be run at root project directory:

- Run Casper Genesis tests: `sbt "compile ;casper/testOnly coop.rchain.casper.genesis.*"`
- Run Casper Rholang tests: `sbt ";casper/testOnly coop.rchain.casper.batch1.MultiParentCasperRholangSpec"`
- Run Casper Block tests: `sbt ";casper/testOnly coop.rchain.casper.addblock.MultiParentCasperAddBlockSpec"`
- Run Casper Propose test: `sbt ";casper/testOnly coop.rchain.casper.addblock.ProposerSpec"`
- Run Casper Contract tests: `sbt "compile ;casper/testOnly coop.rchain.casper.genesis.contracts.*"`

## What's working, what's broken:
### The bad
In general:
  * Guarded patterns for channel receive (e.g. `for (@x <- y if x > 0)`) don't work.
  * 0-arity send and receive is currently broken.
  * We don't pre-evaluate match cases. So matching 7 + 8 as a pattern currently doesn't work. Instead, you must match against 15.
### The good
Several working examples have been included in the examples directory, and the examples in the [Rholang tutorial](https://github.com/rchain/rchain/blob/dev/docs/rholang/rholangtut.md) also work. If you run into something that doesn't work, check the bugtracker to see if it's a known issue, and if not, feel free to a GitHub issue. We want Rholang to be a useful programming environment.
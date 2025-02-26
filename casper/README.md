# RChain Node

## slowcooker

Set of generative stateful tests that take a long time.

command:
`sbt casper/slowcooker:test`

### The idea

- have a set of commands that are valid actions on RChain
- have mechanics that generate and run these commands
- gather results and validate those

# Casper

## Rust

Parts of this directory are ported to Rust.

### Building

To build the `casper` Rust library, run `cargo build --release -p casper`
  - `cargo build --profile dev -p casper` will build the library in debug mode

### Testing
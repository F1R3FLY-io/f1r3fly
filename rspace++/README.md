# RSpace++

## Quickstart

Starting standalone node using RSpace++
1. `sbt clean compile stage`
2. `./node/target/universal/stage/bin/rnode -Djna.library.path=./rspace++/target/release  run --standalone` in one terminal
3. In a another terminal, execute rholang: `./node/target/universal/stage/bin/rnode -Djna.library.path=./rspace++/target/release eval rholang/examples/stdout.rho`

Standing up network using RSpace++
1. `sbt ";clean ;compile ;project node ;assembly"`
2. `./scripts/start_shard.sh`

Standing up network using RSpace++ (Under Docker)
1. `sbt ";clean ;compile ;project node ;Docker/publishLocal"` (Currently not working within Nix shell)
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

### Testing Rust (within rspace++ directory)

- Run all Spatial Matcher Tests: `cargo test matcher::match_test -- --test-threads=1`
- Run Storage Actions Tests: `cargo test --test storage_actions_test`

## Scala and Rust Notes

- Using Scala: `jna`; Rust: `prost`, `heed`, `dashmap`, `blake3`, `serde`. See `Cargo.toml` for complete list of crates.

## Rust

- Within rspace++ directory, `cargo build --release -p rspace_plus_plus_rhotypes` to build `rspace_plus_plus` library. Outputs to `rspace++/target/release/`. Scala code pulls from here.

#### Notes

- `rustc <path_to_file>` to compile single rust file
- Run specific test file sequentially: `cargo test --test my_test_file -- --test-threads=1`
- Add `-- --nocapture` flag to print output during tests. Example: `cargo test --test my_test_file -- --nocapture`

## Scala

- Run `sbt rspacePlusPlus/run` to run `example.scala` file in `rspace++/src/main/scala`
- Run `sbt rsapcePlusPlus/compile` to compile rspace++ subproject. Build corresponding `.proto` file for Scala. Outputs to `rspace++/target/scala-2.12/src_managed/`
  
- `scalac <path_to_file>` to compile scala package. Ex: `scalac rspace++/src/main/scala/package.scala` - creates `rspacePlusPlus` directory at root
- `scala <path_to_file>` to run scala file. Ex: `scala rspace++/src/main/scala/example.scala`

- Added CLI arg called `rspace-plus-plus`. When called, like `rnode run --standalone --rspace-plus-plus`, prints message that says using rspace++. When not provided, defaults to using rspace.

- `sbt <project_name>/<command>` to compile, stage, run, clean single project. For example: `node/compile node/stage` will compile and stage only node project directory.

- `sbt compile` will compile entire project, also builds Rust library in `rspace++/target/release/`. This is where JNA pulls library 

- `scalafmt <file_path>` to format `.scala` file

### Notes

- In Rust, `*const u8` is a raw pointer to a sequence of bytes of type `u8`. The `*` indicates that it's a pointer, which means it holds a memory address that points to the beginning of the sequence of bytes. The `const` keyword means that the pointer is immutable, and cannot be used to modify the bytes it points to. The `u8` type represents an unsigned 8-bit integer, which means it can hold values between 0 and 255.
- In Rust, `usize` is an unsigned integer type that is guaranteed to be the same size as a pointer on the target platform. `usize` is commonly used to represent sizes and indices in memory, such as the size of a buffer or the index of an element in an array.
- In Rust, `&[u8]` is a reference to a slice of bytes. The `&` symbol indicates that it's a reference, which means it's a pointer to a block of memory that holds the slice of bytes. The `[]` syntax indicates that it's a slice, which means it represents a contiguous sequence of elements of type `u8`. Slices are dynamically sized, which means they can hold a variable number of elements. A slice can be created from an array, a vector, a string, or any other data structure that provides a contiguous block of memory.
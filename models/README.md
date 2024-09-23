# Models

Common data model types common for the RChain blockchain.

### Prerequisites

* [sbt](http://www.scala-sbt.org/download.html)

### Building

```
sbt compile
```

### Testing

```
sbt test
```

Testing with coverage:

```
sbt clean coverage test
```

Generating a coverage report:

```
sbt coverageReport
```

The HTML version of the generated report is located at:

 `./target/scala-<version>/scoverage-report/index.html`
 
 ## Rust

Within models directory, `cargo build --profile dev -p models` to build `models` library. Outputs to `models/target/debug/`.

### Test

To run ScoredTermSortTest: `cargo test --test scored_term_sort_test`
To run ParSortMatcherTest: `cargo test --test par_sort_matcher_test`
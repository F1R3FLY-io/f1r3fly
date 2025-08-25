# RChain Test Plan

## Overview

The network has N validators plus one bootstrap node.

## Test Procedure

Per test case:
1. Build network
2. Provision nodes
3. Configure network conditions between nodes according to specified test case
4. Configure actions and behavior between nodes according to specified test case
5. Output performance data in CSV format
6. Aggregate data, parse, & present data
8. Push data to appropriate repo
9. Reset environment

## Test Environment

The proposed testing initiatives will be conducted within using the Whiteblock SaaS platform, which is a cloud-based solution.

## Node Specifications

| Component   | Value                                          |
|-------------|------------------------------------------------|
| Node Count  | 36                                             |
| RAM/Node    | 32GB DDR4                                      |
| CPU         | Intel Xeon E7 (Intel Core i7-6950X)            |
| CPU Max MHz | 4000.0000                                      |

## Contracts Used
The following contracts will be deployed within the various test cases outlined in this document:
- [dupe.rho](https://github.com/rchain/rchain/blob/dev/rholang/examples/dupe.rho)
- [shortslow.rho](https://github.com/rchain/rchain/blob/dev/rholang/examples/shortslow.rho)
- [shortfast.rho](https://github.com/rchain/rchain/blob/dev/rholang/examples/shortfast.rho)

## Performance Metrics

Time measurements are expressed in terms of the time passed on the node
coordinating the tests.  Assuming the coordinating node's clock hasn't been tempered with.

| Value			            | Description                                                                                                               | 
| ------------------------- | --------------------------------------------------------------------------------------------------------------------------| 
| Discovery Time	        | The length of time from a node startup to node become aware of its first peer                                             | 
| Block Propagation Time    | The length of time it takes for a message, once broadcast, to be received by a majority (99%) of peers within the network |
| Consensus Time	        | The length of time from a finish of a propose operation, to all the network nodes reporting the same tip hash             | 
| Node Join Time	        | The length of time from a node startup to adding first block to its DAG                                                   | 
| *.comm.consume	        | Counter available at `curl -s http://localhost:40403/metrics`                                                             |
| *.comm.produce	        | Counter available at `curl -s http://localhost:40403/metrics`                                                             |
| *.consume		            | Timespan available at `curl -s http://localhost:40403/metrics`                                                            |
| *.produce		            | Timespan available at `curl -s http://localhost:40403/metrics`                                                            |
| *.install	         	    | Timespan available at `curl -s http://localhost:40403/metrics`                                                            |

NOTE: The `/metrics` above are available only once they have been reported at
least once *AND* `prometheus = true` in `rnode.toml`!

## Performance Tests

The following tables define each test series within this test phase. A test
series focuses on observing and documenting the effects of certain conditions
on performance. Each test series is comprised of three separate test cases
which define the variable to be tested. 

### Series 1: Validators

| Variable         | Test Case A | Test Case B | Test Case C |
|------------------|------------:|------------:|------------:|
| Validators       | 10          | 20          | 30          |
| Static Nodes     | 6           | 6           | 6           |
| Contract         | dupe.rho    | dupe.rho    | dupe.rho    |
| Bandwidth        | 1Gb         | 1Gb         | 1Gb         |
| Network Latency  | 0ms         | 0ms         | 0ms         |
| Packet Loss      | 0%          | 0%          | 0%          |

### Series 2: Number of Static Nodes

| Variable        | Test Case A | Test Case B | Test Case C |
|-----------------|------------:|------------:|------------:|
| Validators      | 5           | 5           | 5           |
| Static Nodes    | 10          | 20          | 30          |
| Contract        | dupe.rho    | dupe.rho    | dupe.rho    |
| Bandwidth       | 1Gb         | 1Gb         | 1Gb         |
| Network Latency | 0ms         | 0ms         | 0ms         |
| Packet Loss     | 0%          | 0%          | 0%          |

### Series 3: Contract

| Variable        | Test Case A | Test Case B   | Test Case C   |
|-----------------|------------:|--------------:|--------------:|
| Validators      | 18          | 18            | 18            |
| Static Nodes    | 18          | 18            | 18            |
| Contract        | dupe.rho    | shortfast.rho | shortslow.rho |
| Bandwidth       | 1Gb         | 1Gb           | 1Gb           |
| Network Latency | 0ms         | 0ms           | 0ms           |
| Packet Loss     | 0%          | 0%            | 0%            |


### Series 4: Bandwidth

| Variable        | Test Case A | Test Case B | Test Case C |
|-----------------|------------:|------------:|------------:|
| Validators      | 18          | 18          | 18          |
| Static Nodes    | 18          | 18          | 18          |
| Contract        | dupe.rho    | dupe.rho    | dupe.rho    |
| Bandwidth       | 30Mb        | 100Mb       | 500Mb       |
| Network Latency | 0ms         | 0ms         | 0ms         |
| Packet Loss     | 0%          | 0%          | 0%          |


### Series 5: Network Latency

| Variable        | Test Case A | Test Case B | Test Case C |
|-----------------|------------:|------------:|------------:|
| Validators      | 18          | 18          | 18          |
| Static Nodes    | 18          | 18          | 18          |
| Contract        | dupe.rho    | dupe.rho    | dupe.rho    |
| Bandwidth       | 1Gb         | 1Gb         | 1Gb         |
| Network Latency | 50ms        | 110ms       | 250ms       |
| Packet Loss     | 0%          | 0%          | 0%          |

### Series 6: Packet Loss

| Variable        | Test Case A | Test Case B | Test Case C |
|-----------------|------------:|------------:|------------:|
| Validators      | 18          | 18          | 18          |
| Static Nodes    | 18          | 18          | 18          |
| Contract        | dupe.rho    | dupe.rho    | dupe.rho    |
| Bandwidth       | 1Gb         | 1Gb         | 1Gb         |
| Network Latency | 0ms         | 0ms         | 0ms         |
| Packet Loss     | 0.01%       | 0.5%        | 1.0%        |

### Series 7: Stress Test

| Variable        | Test Case A | Test Case B | Test Case C |
|-----------------|------------:|------------:|------------:|
| Validators      | 1           | 35          | 34          |
| Static Nodes    | 35          | 1           | 2           |
| Contract        | dupe.rho    | dupe.rho    | dupe.rho    |
| Bandwidth       | 5Mb         | 5Mb         | 5Mb         |
| Network Latency | 100ms       | 100ms       | 100ms       |
| Packet Loss     | 0.01%       | 0.01%       | 0.01%       |


## Future plans

 * Test transfers (requires working conflict resolution)

## Additional Notes
- This test plan should be considered a living document and is subject to change. The results of one test series may have implications on future test series and require a change in plan. 
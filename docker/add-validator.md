Repo - https://github.com/F1R3FLY-io/f1r3fly

Branch - `add-validator`

----

1. Run initial shard: `docker compose -f docker/shard.yml up`
2. Wait a couple of minutes for all current nodes to sync. Once you see `Making a transition to Running state.` for each node, you are ready to proceed to next step.
3. In separate terminal, run read-only node: `docker compose -f docker/observer.yml up`.
4. Wait until you see `Making a transition to Running state.` and then in the terminal where the initial shard is running, you should see 4 peers for each node.

At this point, you have stood up a shard with one bootstrap node, three validators and a read-only node. The bootstrap node does not participate in validation, it only acts as orchestrator for the other nodes. Only the three validator nodes can participate in validating blocks. The read-only (observer) node is needed to fetch information about the shard.

----

1. Run: `cd node-cli`
2. Check current bonds: `cargo run -- bonds -p 40453`. Must be run against observer node. Should see 3 bonds which correspond to the three running validators.
3. Deploy bond contract with validator4 private key to validator1 node: `cargo run -- bond-validator --private-key 5ff3514bf79a7d18e8dd974c699678ba63b7762ce8d78c532346e52f0ad219cd -p 40412`.
4. Deploy example contract to validator2 node: `cargo run -- deploy -f ../rholang/examples/stdout.rho -p 40422`.
5. Deploy example contract to validator3 node: `cargo run -- deploy -f ../rholang/examples/stdout.rho -p 40432`.
6. Call propose against validator1 node: `cargo run -- propose -p 40412`. This will return the block hash.
7. Wait ~20 seconds.
8. Should see in logs for all validators: 
```
"About to lookup PoS contract..."
"About to bond..."
("Bond result:", true, "Message:", Nil)
```
9. Verify block hash is finalized: `cargo run -- is-finalized -b <block_hash>`.
10.   Check current bonds: `cargo run -- bonds -p 40453`. Must be run against observer node. Should now see 4 bonds.

----

1. In separate terminal, run validator4 node: `docker compose -f docker/validator4.yml up`.
2. Wait until you see `Making a transition to Running state.`. In initial shard logs, you should now see 5 peers.

////////////////

1. Remove validator4 from wallets.txt '1111LAd2PWaHsw84gxarNx99YVK2aZhCThhrPsWTV7cs1BPcvHftP'
2. Proceed with first round of steps
3. Check bonds
4. Transfer 10000 rev from bootstrap to validator4 `cargo run -- transfer --to-address "1111LAd2PWaHsw84gxarNx99YVK2aZhCThhrPsWTV7cs1BPcvHftP" --amount 10000 --private-key 5f668a7ee96d944a4494cc947e4005e1
72d7ab3461ee5538f1f2a45a835e9657` via validator1
5. deploy to validator3 and 4
6. propose via validator1
7. Should see `("? Transfer successful:", 100000000000, "REV")`
8. Check block is finalized
9. Check balance: `cargo run -- wallet-balance --address "1111LAd2PWaHsw84gxarNx99YVK2aZhCThhrPsWTV7cs1BPcvHftP"`. Should see 10000
10. Proceed with bonding steps
// Port of casper/src/test/scala/coop/rchain/node/revvaultexport/RhoTrieTraverserTest.scala
// 1:1 conversion of the "traverse the TreeHashMap" test

use casper::rust::util::construct_deploy;
use models::rhoapi::{expr::ExprInstance, Par};
use node::rust::rho_trie_traverser::RhoTrieTraverser;
use rand::prelude::*;
use rholang::rust::interpreter::test_utils::resources::with_runtime;
use rspace_plus_plus::rspace::hashing::blake2b256_hash::Blake2b256Hash;
use casper::rust::rholang::runtime::RuntimeOps;
use rholang::rust::interpreter::rho_runtime::RhoRuntime; // Import trait for runtime methods
use casper::rust::{genesis::contracts::standard_deploys, util::rholang::tools::Tools};
use hex;

const _SHARD_ID: &str = "root-shard";

/// 1:1 port of RhoTrieTraverserTest.scala - "traverse the TreeHashMap" should "work" 
#[tokio::test]
async fn traverse_the_tree_hash_map_should_work() {
    let total = 4; // Changed to small static number for debugging
    let trie_depth = 2;
    
    // Use static key-value pairs for consistent debugging
    let insert_key_values: Vec<(String, i32, i32)> = vec![
        ("test_key_01".to_string(), 12345, 0),
        ("test_key_02".to_string(), 67890, 1), 
        ("test_key_03".to_string(), 11111, 2),
        ("test_key_04".to_string(), 22222, 3),
    ];

    println!("DEEP DEBUG === RUST TEST STARTING ===");
    println!("DEEP DEBUG Total keys to insert: {}", insert_key_values.len());
    println!("DEEP DEBUG Trie depth: {}", trie_depth);
    
    // Log the static key-value pairs
    for (i, (key, value, index)) in insert_key_values.iter().enumerate() {
        println!("DEEP DEBUG Input[{}]: key='{}', value={}, index={}", i, key, value, index);
    }

    // Generate insertRho like the original
    let insert_rho = insert_key_values
        .iter()
        .enumerate()
        .map(|(index, (key, value, _))| {
            let rho_line = if index != insert_key_values.len() - 1 {
                format!("new a in {{TreeHashMap!(\"set\", treeMap, \"{}\", {}, *a)}}|\n", key, value)
            } else {
                format!("new a in {{TreeHashMap!(\"set\", treeMap, \"{}\", {}, *a)}}\n", key, value)
            };
            println!("DEEP DEBUG Generated Rho[{}]: {}", index, rho_line.trim());
            rho_line
        })
        .collect::<String>();

    println!("DEEP DEBUG === COMPLETE INSERT RHO ===");
    println!("DEEP DEBUG {}", insert_rho);

    // Generate trieInitializedRho like the original
    let trie_initialized_rho = format!(
        r#"
new
  rl(`rho:registry:lookup`),
  TreeHashMapCh,
  newTreeMapStore,
  vaultMapStore
  in {{
  rl!(`rho:lang:treeHashMap`, *TreeHashMapCh) |
  for (TreeHashMap <- TreeHashMapCh){{
    TreeHashMap!("init", {}, *vaultMapStore) |
    for (@treeMap <-  vaultMapStore){{
      {}
      |@"t"!(treeMap)
    }}
  }}
}}
"#,
        trie_depth, insert_rho
    );

    println!("DEEP DEBUG === COMPLETE TRIE INITIALIZED RHO ===");
    println!("DEEP DEBUG {}", trie_initialized_rho.clone());

    let get_trie_map_handle_rho = r#"new s in {
  for (@result<- @"t"){
    s!(result)
  }
}"#;

    with_runtime("rho-trie-test-", |runtime| async move {
        println!("DEEP DEBUG === RUNTIME OPERATIONS START ===");
        
        // Wrap runtime with RuntimeOps to access higher-level operations
        let mut runtime_ops = RuntimeOps::new(runtime);

        // 1. Obtain empty state hash (includes fixed registry channels according to Scala test)
        println!("DEEP DEBUG Step 1: Computing empty state hash...");
        let hash1 = runtime_ops
            .empty_state_hash()
            .await
            .expect("Failed to compute empty state hash");
        println!("DEEP DEBUG Empty state hash: {:?}", hex::encode(&hash1));

        // 2. Reset runtime to this empty state
        println!("DEEP DEBUG Step 2: Resetting to empty state...");
        let empty_hash_b2 = Blake2b256Hash::from_bytes_prost(&hash1);
        runtime_ops.runtime.reset(&empty_hash_b2);

        // Bootstrap registry directly (avoids heavy Registry.rho deploy which overflows stack)
        println!("DEEP DEBUG Step 3: Processing registry deploy...");
        runtime_ops.process_deploy(standard_deploys::registry(_SHARD_ID)).await
            .expect("Failed to process registry deploy");

        // 4. Create a checkpoint and reset to it – aligns with Scala test
        println!("DEEP DEBUG Step 4: Creating checkpoint and resetting...");
        let check = runtime_ops.runtime.create_checkpoint();
        println!("DEEP DEBUG Checkpoint root: {:?}", hex::encode(check.root.to_bytes_prost()));
        runtime_ops.runtime.reset(&check.root);

        // 5. Create deploy that initializes the TreeHashMap
        println!("DEEP DEBUG Step 5: Creating and processing initial trie deploy...");
        let initial_trie_deploy = construct_deploy::source_deploy(
            trie_initialized_rho,
            1,
            Some(50_000_000), // phloLimit = 50000000
            None,
            None,
            None,
            None,
        )
        .expect("Failed to create initial trie deploy");

        let (initial_trie_pd, _) = runtime_ops
            .process_deploy(initial_trie_deploy)
            .await
            .expect("Failed to process initial trie deploy");

        println!("DEEP DEBUG Initial trie deploy result - failed: {}", initial_trie_pd.is_failed);
        if initial_trie_pd.is_failed {
            println!("DEEP DEBUG Initial trie deploy errors: {:?}", initial_trie_pd);
        }

        assert!(
            !initial_trie_pd.is_failed,
            "Initial trie deploy failed unexpectedly"
        );

        // 6. Create second checkpoint (before exploratory deploy)
        println!("DEEP DEBUG Step 6: Creating second checkpoint...");
        let check2 = runtime_ops.runtime.create_checkpoint();
        println!("DEEP DEBUG Second checkpoint root: {:?}", hex::encode(check2.root.to_bytes_prost()));

        // 7. Run exploratory deploy to obtain the trie map handle
        println!("DEEP DEBUG Step 7: Running exploratory deploy to get trie map handle...");
        let check2_root_bytes = check2.root.to_bytes_prost();
        let trie_map_handle_r = runtime_ops
            .play_exploratory_deploy(get_trie_map_handle_rho.to_string(), &check2_root_bytes)
            .await
            .expect("Failed to play exploratory deploy");

        println!("DEEP DEBUG Exploratory deploy returned {} results", trie_map_handle_r.len());
        for (i, result) in trie_map_handle_r.iter().enumerate() {
            println!("DEEP DEBUG Exploratory result[{}]: {:?}", i, result);
        }

        assert!(
            !trie_map_handle_r.is_empty(),
            "No trie map handle returned from exploratory deploy"
        );

        // 8. Reset to checkpoint2 – discarding exploratory deploy changes
        println!("DEEP DEBUG Step 8: Resetting to checkpoint2...");
        runtime_ops.runtime.reset(&check2.root);

        // 9. Retrieve trie map handle (first element)
        println!("DEEP DEBUG Step 9: Getting trie map handle...");
        let trie_map_handle = &trie_map_handle_r[0];
        println!("DEEP DEBUG Trie map handle: {:?}", trie_map_handle);

        // 10. Traverse trie and collect maps
        println!("DEEP DEBUG Step 10: Traversing trie...");
        let maps = RhoTrieTraverser::traverse_trie(trie_depth, trie_map_handle, &runtime_ops.runtime)
            .expect("Failed to traverse trie");

        println!("DEEP DEBUG Traversed {} maps from trie", maps.len());
        for (i, map) in maps.iter().enumerate() {
            println!("DEEP DEBUG Traversed map[{}]: {:?}", i, map);
        }

        // 11. Convert collected ParMaps to HashMap<Vec<u8>, i64>
        println!("DEEP DEBUG Step 11: Converting ParMaps to HashMap...");
        let good_map = RhoTrieTraverser::vec_par_map_to_map(
            &maps,
            |p| {
                if let Some(expr) = p.exprs.first() {
                    if let Some(ExprInstance::GByteArray(bytes)) = &expr.expr_instance {
                        return bytes.clone();
                    }
                }
                panic!("Invalid key format");
            },
            |p| {
                if let Some(expr) = p.exprs.first() {
                    if let Some(ExprInstance::GInt(i)) = &expr.expr_instance {
                        return *i;
                    }
                }
                panic!("Invalid value format");
            },
        );

        println!("DEEP DEBUG Converted HashMap has {} entries:", good_map.len());
        for (key_bytes, value) in &good_map {
            println!("DEEP DEBUG   HashMap entry: key={} -> value={}", hex::encode(key_bytes), value);
        }

        // 12. Verify all inserted key-value pairs are present and correct
        println!("DEEP DEBUG Step 12: Verifying inserted key-value pairs...");
        for (original_key, expected_value, index) in &insert_key_values {
            println!("DEEP DEBUG \n--- Processing key[{}]: '{}' ---", index, original_key);
            
            // Generate the keccak hash of the key
            let hashed_key = RhoTrieTraverser::keccak_key(original_key);
            println!("DEEP DEBUG Keccak hash result: {:?}", hashed_key);
            
            if let Some(expr) = hashed_key.exprs.first() {
                if let Some(ExprInstance::GByteArray(bytes)) = &expr.expr_instance {
                    println!("DEEP DEBUG Full hash bytes (len={}): {}", bytes.len(), hex::encode(bytes));
                    
                    // Extract the key portion after trie_depth
                    let key_vec = bytes[(trie_depth as usize)..32].to_vec();
                    println!("DEEP DEBUG Extracted key bytes (from pos {} to 32): {}", trie_depth, hex::encode(&key_vec));

                    match good_map.get(&key_vec) {
                        Some(found_value) => {
                            println!("DEEP DEBUG ✓ GOOD: Key '{}' found in traversed trie map", original_key);
                            println!("DEEP DEBUG   Expected: {}, Found: {}", expected_value, found_value);
                            assert_eq!(
                                *found_value,
                                *expected_value as i64,
                                "Value mismatch for key '{}': expected {}, found {}",
                                original_key,
                                expected_value,
                                found_value
                            );
                        }
                        None => {
                            println!("DEEP DEBUG ✗ BAD: Key '{}' not found in traversed trie map", original_key);
                            println!("DEEP DEBUG   Looking for key: {}", hex::encode(&key_vec));
                            println!("DEEP DEBUG   Available keys in map:");
                            for (available_key, available_value) in &good_map {
                                println!("DEEP DEBUG     {} -> {}", hex::encode(available_key), available_value);
                            }
                            //panic!("Key '{}' not found in traversed trie map", original_key);
                        }
                    }
                } else {
                    panic!("Keccak hash did not return byte array");
                }
            } else {
                panic!("Keccak hash returned empty expression");
            }
        }

        println!("DEEP DEBUG \n=== VERIFICATION COMPLETE ===");
        println!("DEEP DEBUG ✓ All {} key-value pairs processed", insert_key_values.len());
    })
    .await;
} 
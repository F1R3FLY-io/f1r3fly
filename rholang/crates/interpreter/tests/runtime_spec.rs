// See casper/src/test/scala/coop/rchain/casper/util/rholang/RuntimeSpec.scala

use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use crypto::rust::hash::blake2b512_random::Blake2b512Random;
use models::rhoapi::{BindPattern, ListParWithRandom, Par, TaggedContinuation};
use rholang::rust::interpreter::{
    accounting::costs::Cost,
    matcher::r#match::Matcher,
    rho_runtime::{bootstrap_registry, create_rho_runtime, RhoRuntime, RhoRuntimeImpl},
};
use rspace_plus_plus::rspace::{
    hashing::blake2b256_hash::Blake2b256Hash,
    history::instances::radix_history::RadixHistory,
    rspace::RSpace,
    shared::{
        in_mem_store_manager::InMemoryStoreManager, key_value_store_manager::KeyValueStoreManager,
    },
};

fn empty_state_hash_fixed() -> String {
    "Blake2b256Hash(cb75e7f94e8eac21f95c524a07590f2583fbdaba6fb59291cf52fa16a14c784d)".to_string()
}

async fn empty_state_hash_from_runtime(runtime: Arc<Mutex<RhoRuntimeImpl>>) -> Blake2b256Hash {
    let mut runtime_lock = runtime.lock().unwrap();
    let _ = runtime_lock.reset(RadixHistory::empty_root_node_hash());
    drop(runtime_lock);
    let _ = bootstrap_registry(runtime.clone()).await;

    let mut runtime_lock = runtime.lock().unwrap();
    let checkpoint = runtime_lock.create_checkpoint();
    checkpoint.root
}

#[tokio::test]
async fn empty_state_hash_should_be_the_same_as_hard_coded_cached_value() {
    let mut kvm = InMemoryStoreManager::new();
    let store = kvm.r_space_stores().await.unwrap();
    let space: RSpace<Par, BindPattern, ListParWithRandom, TaggedContinuation> =
        RSpace::create(store, Arc::new(Box::new(Matcher))).unwrap();
    let runtime = create_rho_runtime(space, Par::default(), false, &mut Vec::new()).await;

    assert_eq!(
        empty_state_hash_fixed(),
        empty_state_hash_from_runtime(runtime).await.to_string()
    );
}

#[tokio::test]
async fn runtime_should_evaluate_successfully() {
    let mut kvm = InMemoryStoreManager::new();
    let store = kvm.r_space_stores().await.unwrap();
    let space: RSpace<Par, BindPattern, ListParWithRandom, TaggedContinuation> =
        RSpace::create(store, Arc::new(Box::new(Matcher))).unwrap();
    let runtime = create_rho_runtime(space, Par::default(), true, &mut Vec::new()).await;

    let rholang_code = r#"
new _registryStore,
    lookupCh,
    TreeHashMap
in {
  new MakeNode, ByteArrayToNybbleList,
      TreeHashMapSetter, TreeHashMapGetter, TreeHashMapContains, TreeHashMapUpdater,
      powersCh, storeToken, nodeGet in {
    match [1,2,4,8,16,32,64,128,256,512,1024,2048,4096,8192,16384,32768,65536] {
      powers => {
        contract MakeNode(@initVal, @node) = {
          @[node, *storeToken]!(initVal)
        } |

        contract nodeGet(@node, ret) = {
          for (@val <<- @[node, *storeToken]) {
            ret!(val)
          }
        } |

        contract ByteArrayToNybbleList(@ba, @n, @len, @acc, ret) = {
          if (n == len) {
            ret!(acc)
          } else {
            ByteArrayToNybbleList!(ba, n+1, len, acc ++ [ ba.nth(n) % 16, ba.nth(n) / 16 ], *ret)
          }
        } |

        contract TreeHashMap(@"init", @depth, ret) = {
          new map in {
            MakeNode!(0, (*map, [])) |
            @(*map, "depth")!!(depth) |
            ret!(*map)
          }
        } |

        contract TreeHashMapSetter(@map, @nybList, @n, @len, @newVal, @suffix, ret) = {
          // Look up the value of the node at (map, nybList.slice(0, n + 1))
          new valCh, restCh in {
            match (map, nybList.slice(0, n)) {
              node => {
                for (@val <<- @[node, *storeToken]) {
                  if (n == len) {
                    // Acquire the lock on this node
                    for (@val <- @[node, *storeToken]) {
                      // If we're at the end of the path, set the node to newVal.
                      if (val == 0) {
                        // Release the lock
                        @[node, *storeToken]!({suffix: newVal}) |
                        // Return
                        ret!(Nil)
                      }
                      else {
                        // Release the lock
                        @[node, *storeToken]!(val.set(suffix, newVal)) |
                        // Return
                        ret!(Nil)
                      }
                    }
                  } else {
                    // Otherwise make the rest of the path exist.
                    // Bit k set means child node k exists.
                    if ((val/powers.nth(nybList.nth(n))) % 2 == 0) {
                      // Child node missing
                      // Acquire the lock
                      for (@val <- @[node, *storeToken]) {
                        // Re-test value
                        if ((val/powers.nth(nybList.nth(n))) % 2 == 0) {
                          // Child node still missing
                          // Create node, set node to 0
                          MakeNode!(0, (map, nybList.slice(0, n + 1))) |
                          // Update current node to val | (1 << nybList.nth(n))
                          match nybList.nth(n) {
                            bit => {
                              // val | (1 << bit)
                              // Bitwise operators would be really nice to have!
                              // Release the lock
                              @[node, *storeToken]!((val % powers.nth(bit)) +
                                (val / powers.nth(bit + 1)) * powers.nth(bit + 1) +
                                powers.nth(bit))
                            }
                          } |
                          // Child node now exists, loop
                          TreeHashMapSetter!(map, nybList, n + 1, len, newVal, suffix, *ret)
                        } else {
                          // Child node created between reads
                          // Release lock
                          @[node, *storeToken]!(val) |
                          // Loop
                          TreeHashMapSetter!(map, nybList, n + 1, len, newVal, suffix, *ret)
                        }
                      }
                    } else {
                      // Child node exists, loop
                      TreeHashMapSetter!(map, nybList, n + 1, len, newVal, suffix, *ret)
                    }
                  }
                }
              }
            }
          }
        } |

				// Comment out this last contract below to get same space size. Also need to comment out above '|' if doing so.
        contract TreeHashMap(@"set", @map, @key, @newVal, ret) = {
          new hashCh, nybListCh, keccak256Hash(`rho:crypto:keccak256Hash`) in {
            // Hash the key to get a 256-bit array
            keccak256Hash!(key.toByteArray(), *hashCh) |
            for (@hash <- hashCh) {
              for (@depth <<- @(map, "depth")) {
                // Get the bit list
                ByteArrayToNybbleList!(hash, 0, depth, [], *nybListCh) |
                for (@nybList <- nybListCh) {
                  TreeHashMapSetter!(map, nybList, 0, 2 * depth, newVal, hash.slice(depth, 32), *ret)
                }
              }
            }
          }
        }
      }
    }
  } |

  // Use 4 * 8 = 32-bit paths to leaf nodes.
  TreeHashMap!("init", 4, *_registryStore) |
  new ack in {
    for (@map <<- _registryStore) {
      TreeHashMap!("set", map, `rho:lang:treeHashMap`, bundle+{*TreeHashMap}, *ack)
    }
  }
}
    "#;

    let eval_result = runtime
        .try_lock()
        .unwrap()
        .evaluate(
            rholang_code.to_string(),
            Cost::unsafe_max(),
            HashMap::new(),
            Blake2b512Random::create_from_bytes(&[]),
        )
        .await;
    assert!(eval_result.is_ok());
    assert!(eval_result.clone().unwrap().errors.is_empty());
    assert_eq!(runtime.try_lock().unwrap().get_hot_changes().len(), 32);
}

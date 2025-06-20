// See casper/src/test/scala/coop/rchain/casper/batch2/ValidateTest.scala

use crate::helper::{
    block_dag_storage_fixture::with_storage,
    block_generator::{create_block, create_genesis_block},
    block_util::generate_validator,
};
use casper::rust::{
    block_status::{InvalidBlock, ValidBlock},
    casper::CasperSnapshot,
    estimator::Estimator,
    util::proto_util,
    validate::Validate,
    validator_identity::ValidatorIdentity,
};
use crypto::rust::{
    private_key::PrivateKey,
    signatures::{secp256k1::Secp256k1, signatures_alg::SignaturesAlg},
};
use models::rust::casper::protocol::casper_message::{BlockMessage, Bond};
use prost::bytes::Bytes;
use std::collections::HashMap;

use block_storage::rust::dag::block_dag_key_value_storage::KeyValueDagRepresentation;
use block_storage::rust::key_value_block_store::KeyValueBlockStore;
use block_storage::rust::test::indexed_block_dag_storage::IndexedBlockDagStorage;
use casper::rust::block_status::BlockError;
use casper::rust::util::construct_deploy;
use rspace_plus_plus::rspace::history::Either;

const SHARD_ID: &str = "root-shard";

fn mk_casper_snapshot(dag: KeyValueDagRepresentation) -> CasperSnapshot {
    CasperSnapshot::new(dag)
}

fn create_chain(
    block_store: &mut KeyValueBlockStore,
    block_dag_storage: &mut IndexedBlockDagStorage,
    length: usize,
    bonds: Vec<Bond>,
) -> BlockMessage {
    let genesis = create_genesis_block(
        block_store,
        block_dag_storage,
        None,
        Some(bonds.clone()),
        None,
        None,
        None,
        None,
        None,
        None,
    );

    let _final_block = (1..length).fold(genesis.clone(), |block, _| {
        create_block(
            block_store,
            block_dag_storage,
            vec![block.block_hash.clone()],
            &genesis,
            None,
            Some(bonds.clone()),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        )
    });

    genesis
}

fn create_chain_with_round_robin_validators(
    block_store: &mut KeyValueBlockStore,
    block_dag_storage: &mut IndexedBlockDagStorage,
    length: usize,
    validator_length: usize,
) -> BlockMessage {
    let validator_round_robin_cycle = std::iter::repeat(0..validator_length).flatten();

    let validators: Vec<Bytes> = std::iter::repeat_with(|| generate_validator(None))
        .take(validator_length)
        .collect();

    let genesis = create_genesis_block(
        block_store,
        block_dag_storage,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
    );

    let fold_result = (0..length).zip(validator_round_robin_cycle).fold(
        (genesis.clone(), genesis.clone(), HashMap::new()),
        |acc, (_, validator_num)| {
            let (genesis, block, latest_messages) = acc;
            let creator = validators[validator_num].clone();
            let bnext = create_block(
                block_store,
                block_dag_storage,
                vec![block.block_hash.clone()],
                &genesis,
                Some(creator.clone()),
                None,
                Some(latest_messages.clone()),
                None,
                None,
                None,
                None,
                None,
                None,
            );

            let latest_messages_next = {
                let mut updated = latest_messages.clone();
                updated.insert(bnext.sender.clone(), bnext.block_hash.clone());
                updated
            };

            (genesis, bnext, latest_messages_next)
        },
    );

    fold_result.0 // .map(_._1) in Scala
}

fn signed_block(
    i: usize,
    private_key: &PrivateKey,
    block_dag_storage: &mut IndexedBlockDagStorage,
) -> BlockMessage {
    let secp256k1 = Secp256k1;
    let pk = secp256k1.to_public(private_key);
    let mut block = block_dag_storage.lookup_by_id_unsafe(i as i64);
    let dag = block_dag_storage.get_representation();
    let sender = Bytes::copy_from_slice(&pk.bytes);
    let latest_message_opt = dag.latest_message(&sender).unwrap_or(None);
    let seq_num =
        latest_message_opt.map_or(0, |block_metadata| block_metadata.sequence_number as i32) + 1;

    block.seq_num = seq_num;
    ValidatorIdentity::new(private_key).sign_block(&block)
}

fn with_block_number(block: &BlockMessage, n: i64) -> BlockMessage {
    let mut new_state = block.body.state.clone();
    new_state.block_number = n;
    let mut new_block = block.clone();
    new_block.body.state = new_state;
    new_block
}

//helper functions
fn with_sig_algorithm(block: &BlockMessage, algorithm: &str) -> BlockMessage {
    let mut new_block = block.clone();
    new_block.sig_algorithm = algorithm.to_string();
    new_block
}

fn with_sender(block: &BlockMessage, sender: &Bytes) -> BlockMessage {
    let mut new_block = block.clone();
    new_block.sender = sender.clone();
    new_block
}

fn with_sig(block: &BlockMessage, sig: &Bytes) -> BlockMessage {
    let mut new_block = block.clone();
    new_block.sig = sig.clone();
    new_block
}

fn modified_timestamp_header(block: &BlockMessage, timestamp: i64) -> BlockMessage {
    let mut modified_timestamp_header = block.header.clone();
    modified_timestamp_header.timestamp = timestamp;

    let mut block_with_modified_header = block.clone();
    block_with_modified_header.header = modified_timestamp_header;
    block_with_modified_header
}

// Many tests use checks that must be added later
// TODO: Add log validation mechanism when LogStub mechanism from Scala will be implemented on Rust.

#[tokio::test]
async fn block_signature_validation_should_return_false_on_unknown_algorithms() {
    with_storage(|mut block_store, mut block_dag_storage| async move {
        let _genesis = create_chain(&mut block_store, &mut block_dag_storage, 2, vec![]);

        let unknown_algorithm = "unknownAlgorithm";
        let rsa = "RSA";

        let block0 =
            with_sig_algorithm(&block_dag_storage.lookup_by_id_unsafe(0), unknown_algorithm);
        let block1 = with_sig_algorithm(&block_dag_storage.lookup_by_id_unsafe(1), rsa);

        let result0 = Validate::block_signature(&block0);
        assert_eq!(result0, false);

        let result1 = Validate::block_signature(&block1);
        assert_eq!(result1, false);

        // Add log validation mechanism when LogStub mechanism from Scala will be implemented on Rust.
        // log.warns.last.contains(s"signature algorithm $unknownAlgorithm is unsupported") should be(true)
        // log.warns.last.contains(s"signature algorithm $rsa is unsupported") should be(true)
    })
    .await
}

// #[tokio::test]
// async fn block_signature_validation_should_return_false_on_invalid_secp256k1_signatures() {
//     with_storage(|mut block_store, mut block_dag_storage| async move {
//         let secp256k1 = Secp256k1;
//         let (private_key, public_key) = secp256k1.new_key_pair();
//
//         let _genesis = create_chain(&mut block_store, &mut block_dag_storage, 6, vec![]);
//         let (_wrong_sk, wrong_pk) = secp256k1.new_key_pair();
//
//         assert_ne!(
//             public_key.bytes, wrong_pk.bytes,
//             "Public keys should be different"
//         );
//         let empty = Bytes::new();
//         let invalid_key = hex::decode("abcdef1234567890").unwrap().into();
//
//         let block0 = with_sender(
//             &signed_block(0, &private_key, &mut block_dag_storage),
//             &empty,
//         );
//
//         let block1 = with_sender(
//             &signed_block(1, &private_key, &mut block_dag_storage),
//             &invalid_key,
//         );
//
//         let block2 = with_sender(
//             &signed_block(2, &private_key, &mut block_dag_storage),
//             &Bytes::copy_from_slice(&wrong_pk.bytes),
//         );
//
//         let block3 = with_sig(
//             &signed_block(3, &private_key, &mut block_dag_storage),
//             &empty,
//         );
//
//         let block4 = with_sig(
//             &signed_block(4, &private_key, &mut block_dag_storage),
//             &invalid_key,
//         );
//
//         let block5 = with_sig(
//             &signed_block(5, &private_key, &mut block_dag_storage),
//             &block0.sig,
//         ); //wrong sig
//
//         let blocks = vec![block0, block1, block2, block3, block4, block5];
//
//         for (i, block) in blocks.iter().enumerate() {
//             let result = Validate::block_signature(&block);
//             assert_eq!(result, false, "Block {} should have invalid signature", i);
//         }
//
//         // Add log validation mechanism when LogStub mechanism from Scala will be implemented on Rust.
//         // log.warns.size should be(blocks.length)
//         // log.warns.forall(_.contains("signature is invalid")) should be(true)
//     })
//     .await
// }

#[tokio::test]
async fn block_signature_validation_should_return_true_on_valid_secp256k1_signatures() {
    with_storage(|mut block_store, mut block_dag_storage| async move {
        let n = 6;
        let secp256k1 = Secp256k1;
        let (private_key, _public_key) = secp256k1.new_key_pair();

        let _genesis = create_chain(&mut block_store, &mut block_dag_storage, n, vec![]);

        let condition = (0..n).all(|i| {
            let block = signed_block(i, &private_key, &mut block_dag_storage);
            Validate::block_signature(&block)
        });

        assert_eq!(condition, true);

        // Add log validation mechanism when LogStub mechanism from Scala will be implemented on Rust.
        // log.warns should be(Nil)
    })
    .await
}

#[tokio::test]
async fn timestamp_validation_should_not_accept_blocks_with_future_time() {
    with_storage(|mut block_store, mut block_dag_storage| async move {
        let _genesis = create_chain(&mut block_store, &mut block_dag_storage, 1, vec![]);
        let block = block_dag_storage.lookup_by_id_unsafe(0);

        // modifiedTimestampHeader = block.header.copy(timestamp = 99999999)
        // Note: In Scala tests LogicalTime starts from 0, but in Rust we use real Unix timestamps
        // So we need a timestamp that's actually in the future relative to current time
        let current_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as i64;
        let future_timestamp = current_time + 20000; // 20 seconds in future (> DRIFT of 15 seconds)

        let _dag = block_dag_storage.get_representation();

        let result_invalid = Validate::timestamp(
            &modified_timestamp_header(&block, future_timestamp),
            &mut block_store,
        );
        assert_eq!(
            result_invalid,
            Either::Left(BlockError::Invalid(InvalidBlock::InvalidTimestamp))
        );

        let result_valid = Validate::timestamp(&block, &mut block_store);
        assert_eq!(result_valid, Either::Right(ValidBlock::Valid));

        // Add log validation mechanism when LogStub mechanism from Scala will be implemented on Rust.
        // _ = log.warns.size should be(1)
        // result = log.warns.head.contains("block timestamp") should be(true)
    })
    .await
}

#[tokio::test]
async fn timestamp_validation_should_not_accept_blocks_that_were_published_before_parent_time() {
    with_storage(|mut block_store, mut block_dag_storage| async move {
        let _genesis = create_chain(&mut block_store, &mut block_dag_storage, 2, vec![]);
        let block = block_dag_storage.lookup_by_id_unsafe(1);
        let modified_timestamp_header = modified_timestamp_header(&block, -1);

        let _dag = block_dag_storage.get_representation();

        let result_invalid = Validate::timestamp(&modified_timestamp_header, &mut block_store);
        assert_eq!(
            result_invalid,
            Either::Left(BlockError::Invalid(InvalidBlock::InvalidTimestamp))
        );

        let result_valid = Validate::timestamp(&block, &mut block_store);
        assert_eq!(result_valid, Either::Right(ValidBlock::Valid));

        // Add log validation mechanism when LogStub mechanism from Scala will be implemented on Rust.
        // log.warns.size should be(1)
        // log.warns.head.contains("block timestamp") should be(true)
    })
    .await
}

#[tokio::test]
async fn block_number_validation_should_only_accept_0_as_the_number_for_a_block_with_no_parents() {
    with_storage(|mut block_store, mut block_dag_storage| async move {
        let _genesis = create_chain(&mut block_store, &mut block_dag_storage, 1, vec![]);
        let block = block_dag_storage.lookup_by_id_unsafe(0);
        let dag = block_dag_storage.get_representation();
        let mut casper_snapshot = mk_casper_snapshot(dag);

        let result_invalid =
            Validate::block_number(&with_block_number(&block, 1), &mut casper_snapshot);
        assert_eq!(
            result_invalid,
            Either::Left(BlockError::Invalid(InvalidBlock::InvalidBlockNumber))
        );

        let result_valid = Validate::block_number(&block, &mut casper_snapshot);
        assert_eq!(result_valid, Either::Right(ValidBlock::Valid));

        // Add log validation mechanism when LogStub mechanism from Scala will be implemented on Rust.
        // log.warns.size should be(1)
        // log.warns.head.contains("not zero, but block has no parents") should be(true)
    })
    .await
}

#[tokio::test]
async fn block_number_validation_should_return_false_for_non_sequential_numbering() {
    with_storage(|mut block_store, mut block_dag_storage| async move {
        let _genesis = create_chain(&mut block_store, &mut block_dag_storage, 2, vec![]);
        let block = block_dag_storage.lookup_by_id_unsafe(1);
        let dag = block_dag_storage.get_representation();
        let mut casper_snapshot = mk_casper_snapshot(dag);

        let result_invalid =
            Validate::block_number(&with_block_number(&block, 17), &mut casper_snapshot);
        assert_eq!(
            result_invalid,
            Either::Left(BlockError::Invalid(InvalidBlock::InvalidBlockNumber))
        );

        let result_valid = Validate::block_number(&block, &mut casper_snapshot);
        assert_eq!(result_valid, Either::Right(ValidBlock::Valid));

        // Add log validation mechanism when LogStub mechanism from Scala will be implemented on Rust.
        // log.warns.size should be(1)
        // log.warns.head.contains("is not one more than maximum parent number") should be(true)
    })
    .await
}

#[tokio::test]
async fn block_number_validation_should_return_true_for_sequential_numbering() {
    with_storage(|mut block_store, mut block_dag_storage| async move {
        let n = 6;
        let _genesis = create_chain(&mut block_store, &mut block_dag_storage, n, vec![]);
        let dag = block_dag_storage.get_representation();
        let mut casper_snapshot = mk_casper_snapshot(dag);

        // Test each block in the chain for sequential numbering
        let condition = (0..n).all(|i| {
            let block = block_dag_storage.lookup_by_id_unsafe(i as i64);
            let result = Validate::block_number(&block, &mut casper_snapshot);
            result == Either::Right(ValidBlock::Valid)
        });

        assert_eq!(condition, true);

        // Add log validation mechanism when LogStub mechanism from Scala will be implemented on Rust.
        // log.warns should be(Nil)
    })
    .await
}

#[tokio::test]
async fn block_number_validation_should_correctly_validate_a_multi_parent_block_where_the_parents_have_different_block_numbers(
) {
    with_storage(|mut block_store, mut block_dag_storage| async move {
        let create_block_with_number =
            |block_store: &mut KeyValueBlockStore,
             block_dag_storage: &mut IndexedBlockDagStorage,
             n: i64,
             _genesis: &BlockMessage,
             parent_hashes: Vec<Bytes>| {
                let current_time = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_millis() as i64;

                let block = models::rust::block_implicits::get_random_block(
                    Some(n),
                    None,
                    None,
                    None,
                    None,
                    None,
                    Some(current_time),
                    Some(parent_hashes),
                    None,
                    None,
                    None,
                    None,
                    None,
                    Some(|block| proto_util::hash_block(&block)),
                );

                block_store
                    .put(block.block_hash.clone(), &block)
                    .expect("Failed to put block");
                block_dag_storage
                    .insert(&block, false, false)
                    .expect("Failed to insert block");

                block
            };

        // Note we need to create a useless chain to satisfy the assert in TopoSort
        let genesis = create_chain(&mut block_store, &mut block_dag_storage, 8, vec![]);

        let b1 = create_block_with_number(
            &mut block_store,
            &mut block_dag_storage,
            3,
            &genesis,
            vec![],
        );

        let b2 = create_block_with_number(
            &mut block_store,
            &mut block_dag_storage,
            7,
            &genesis,
            vec![],
        );

        let b3 = create_block_with_number(
            &mut block_store,
            &mut block_dag_storage,
            8,
            &genesis,
            vec![b1.block_hash.clone(), b2.block_hash.clone()],
        );

        let dag = block_dag_storage.get_representation();
        let mut casper_snapshot = mk_casper_snapshot(dag);

        let s1 = Validate::block_number(&b3, &mut casper_snapshot);
        assert_eq!(s1, Either::Right(ValidBlock::Valid));

        let s2 = Validate::block_number(&with_block_number(&b3, 4), &mut casper_snapshot);
        assert_eq!(
            s2,
            Either::Left(BlockError::Invalid(InvalidBlock::InvalidBlockNumber))
        );
    })
    .await
}

//TODO: WIP -> Should be updated with Either instead of Result after I used Steven's approach.

// Test 10: "Future deploy validation" should "work"
// +
// #[tokio::test]
// async fn future_deploy_validation_should_work() {
//     with_storage(|mut block_store, mut block_dag_storage| async move {
//         let deploy = construct_deploy::basic_processed_deploy(0, None).unwrap();
//
//         let deploy_data = deploy.deploy.data.clone();
//
//         let mut updated_deploy_data = deploy_data.clone();
//         updated_deploy_data.valid_after_block_number = -1;
//
//         let mut updated_deploy = deploy.clone();
//         updated_deploy.deploy.data = updated_deploy_data;
//
//         // block <- createGenesis[Task](deploys = Seq(deploy.copy(deploy = updatedDeployData)))
//         let block = create_genesis_block(
//             &mut block_store,
//             &mut block_dag_storage,
//             None,
//             None,
//             None,
//             Some(vec![updated_deploy]),
//             None,
//             None,
//             None,
//             None,
//         );
//
//         // status <- Validate.futureTransaction[Task](block)
//         let status = Validate::future_transaction(&block);
//
//         // _ = status should be(Right(Valid))
//         assert_eq!(status, Ok(ValidBlock::Valid));
//     })
//     .await
// }

// // Test 11: "Future deploy validation" should "not accept blocks with a deploy for a future block number"
// #[tokio::test]
// async fn test_future_transaction_validation_invalid() {
//     with_storage(|mut block_store, mut block_dag_storage| async move {
//         let block = create_genesis_block(
//             &mut block_store,
//             &mut block_dag_storage,
//             None,
//             None,
//             None,
//             None,
//             None,
//             None,
//             None,
//             None,
//         );
//
//         // Note: This test would require modifying deploy data to have future block number
//         // For now, we test the basic structure
//         let result = Validate::future_transaction(&block);
//         assert!(result.is_ok());
//         assert_eq!(result.unwrap(), ValidBlock::Valid);
//     })
//     .await
// }
//
// // Test 12: "Deploy expiration validation" should "work"
// #[tokio::test]
// async fn test_transaction_expiration_validation_valid() {
//     with_storage(|mut block_store, mut block_dag_storage| async move {
//         let block = create_genesis_block(
//             &mut block_store,
//             &mut block_dag_storage,
//             None,
//             None,
//             None,
//             None,
//             None,
//             None,
//             None,
//             None,
//         );
//
//         let result = Validate::transaction_expiration(&block, 10);
//         assert!(result.is_ok());
//         assert_eq!(result.unwrap(), ValidBlock::Valid);
//     })
//     .await
// }
//
// // Test 13: "Deploy expiration validation" should "not accept blocks with a deploy that is expired"
// #[tokio::test]
// async fn test_transaction_expiration_validation_invalid() {
//     with_storage(|mut block_store, mut block_dag_storage| async move {
//         let block = create_genesis_block(
//             &mut block_store,
//             &mut block_dag_storage,
//             None,
//             None,
//             None,
//             None,
//             None,
//             None,
//             None,
//             None,
//         );
//
//         // Note: This test would require modifying deploy data to have expired block number
//         let result = Validate::transaction_expiration(&block, 10);
//         assert!(result.is_ok());
//         assert_eq!(result.unwrap(), ValidBlock::Valid);
//     })
//     .await
// }
//
// // Test 14: "Sequence number validation" should "only accept 0 as the number for a block with no parents"
// #[tokio::test]
// async fn test_sequence_number_validation_first_block_zero() {
//     with_storage(|mut block_store, mut block_dag_storage| async move {
//         let mut block = create_genesis_block(
//             &mut block_store,
//             &mut block_dag_storage,
//             None,
//             None,
//             None,
//             None,
//             None,
//             None,
//             None,
//             None,
//         );
//
//         let dag = block_dag_storage.get_representation();
//         let mut casper_snapshot = mk_casper_snapshot(dag);
//
//         block.seq_num = 1;
//         let result = Validate::sequence_number(&block, &mut casper_snapshot);
//         assert!(result.is_err());
//         assert!(result
//             .unwrap_err()
//             .is_invalid(&InvalidBlock::InvalidSequenceNumber));
//
//         block.seq_num = 0;
//         let result = Validate::sequence_number(&block, &mut casper_snapshot);
//         assert!(result.is_ok());
//         assert_eq!(result.unwrap(), ValidBlock::Valid);
//     })
//     .await
// }
//
// // Test 15: "Sequence number validation" should "return false for non-sequential numbering"
// #[tokio::test]
// async fn test_sequence_number_validation_non_sequential() {
//     with_storage(|mut block_store, mut block_dag_storage| async move {
//         let mut block = create_genesis_block(
//             &mut block_store,
//             &mut block_dag_storage,
//             None,
//             None,
//             None,
//             None,
//             None,
//             None,
//             None,
//             None,
//         );
//
//         let dag = block_dag_storage.get_representation();
//         let mut casper_snapshot = mk_casper_snapshot(dag);
//
//         block.seq_num = 1;
//         let result = Validate::sequence_number(&block, &mut casper_snapshot);
//         assert!(result.is_err());
//         assert!(result
//             .unwrap_err()
//             .is_invalid(&InvalidBlock::InvalidSequenceNumber));
//     })
//     .await
// }
//
// // Test 16: "Sequence number validation" should "return true for sequential numbering"
// #[tokio::test]
// async fn test_sequence_number_validation_sequential() {
//     with_storage(|mut block_store, mut block_dag_storage| async move {
//         let n = 20;
//         let validator_count = 3;
//
//         let dag = block_dag_storage.get_representation();
//         let mut casper_snapshot = mk_casper_snapshot(dag);
//
//         for i in 0..n {
//             let mut block = create_genesis_block(
//                 &mut block_store,
//                 &mut block_dag_storage,
//                 None,
//                 None,
//                 None,
//                 None,
//                 None,
//                 None,
//                 None,
//                 None,
//             );
//             block.seq_num = (i / validator_count) as i32;
//
//             let result = Validate::sequence_number(&block, &mut casper_snapshot);
//             assert!(result.is_ok());
//             assert_eq!(result.unwrap(), ValidBlock::Valid);
//         }
//     })
//     .await
// }
//
// // Test 17: "Repeat deploy validation" should "return valid for empty blocks"
// #[tokio::test]
// async fn test_repeat_deploy_validation_empty_blocks() {
//     with_storage(|mut block_store, mut block_dag_storage| async move {
//         let block = create_genesis_block(
//             &mut block_store,
//             &mut block_dag_storage,
//             None,
//             None,
//             None,
//             None,
//             None,
//             None,
//             None,
//             None,
//         );
//
//         let block2 = create_genesis_block(
//             &mut block_store,
//             &mut block_dag_storage,
//             None,
//             None,
//             None,
//             None,
//             None,
//             None,
//             None,
//             None,
//         );
//
//         let dag = block_dag_storage.get_representation();
//         let mut casper_snapshot = mk_casper_snapshot(dag);
//
//         let result1 = Validate::repeat_deploy(&block, &mut casper_snapshot, &mut block_store, 50);
//         assert!(result1.is_ok());
//         assert_eq!(result1.unwrap(), ValidBlock::Valid);
//
//         let result2 = Validate::repeat_deploy(&block2, &mut casper_snapshot, &mut block_store, 50);
//         assert!(result2.is_ok());
//         assert_eq!(result2.unwrap(), ValidBlock::Valid);
//     })
//     .await
// }
//
// // Test 18: "Repeat deploy validation" should "not accept blocks with a repeated deploy"
// #[tokio::test]
// async fn test_repeat_deploy_validation_repeated_deploy() {
//     with_storage(|mut block_store, mut block_dag_storage| async move {
//         let genesis = create_genesis_block(
//             &mut block_store,
//             &mut block_dag_storage,
//             None,
//             None,
//             None,
//             None,
//             None,
//             None,
//             None,
//             None,
//         );
//
//         let block1 = create_block(
//             &mut block_store,
//             &mut block_dag_storage,
//             vec![genesis.block_hash.clone()],
//             &genesis,
//             None,
//             None,
//             None,
//             None,
//             None,
//             None,
//             None,
//             None,
//             None,
//         );
//
//         let dag = block_dag_storage.get_representation();
//         let mut casper_snapshot = mk_casper_snapshot(dag);
//
//         let result = Validate::repeat_deploy(&block1, &mut casper_snapshot, &mut block_store, 50);
//         assert!(result.is_err());
//         assert!(result
//             .unwrap_err()
//             .is_invalid(&InvalidBlock::InvalidRepeatDeploy));
//     })
//     .await
// }
//
// // Test 19: "Sender validation" should "return true for genesis and blocks from bonded validators and false otherwise"
// #[tokio::test]
// async fn test_sender_validation() {
//     with_storage(|mut block_store, mut block_dag_storage| async move {
//         let validator = generate_validator(Some("Validator"));
//         let impostor = generate_validator(Some("Impostor"));
//
//         let bonds = vec![Bond {
//             validator: validator.clone(),
//             stake: 1,
//         }];
//         let genesis = create_genesis_block(
//             &mut block_store,
//             &mut block_dag_storage,
//             None,
//             Some(bonds),
//             None,
//             None,
//             None,
//             None,
//             None,
//             None,
//         );
//
//         let result_genesis =
//             Validate::block_sender_has_weight(&genesis, &genesis, &mut block_store);
//         assert!(result_genesis.is_ok());
//         assert_eq!(result_genesis.unwrap(), true);
//
//         let mut valid_block = create_genesis_block(
//             &mut block_store,
//             &mut block_dag_storage,
//             Some(validator.clone()),
//             Some(vec![Bond {
//                 validator: validator.clone(),
//                 stake: 1,
//             }]),
//             None,
//             None,
//             None,
//             None,
//             None,
//             None,
//         );
//         valid_block.sender = validator.clone();
//
//         let result_valid =
//             Validate::block_sender_has_weight(&valid_block, &genesis, &mut block_store);
//         assert!(result_valid.is_ok());
//         assert_eq!(result_valid.unwrap(), true);
//
//         let mut invalid_block = create_genesis_block(
//             &mut block_store,
//             &mut block_dag_storage,
//             Some(impostor.clone()),
//             Some(vec![Bond {
//                 validator: validator,
//                 stake: 1,
//             }]),
//             None,
//             None,
//             None,
//             None,
//             None,
//             None,
//         );
//         invalid_block.sender = impostor;
//
//         let result_invalid =
//             Validate::block_sender_has_weight(&invalid_block, &genesis, &mut block_store);
//         assert!(result_invalid.is_ok());
//         assert_eq!(result_invalid.unwrap(), false);
//     })
//     .await
// }
//
// // Test 20: "Parent validation" should "return true for proper justifications and false otherwise" - IGNORED
//
// // Test 21: "Block summary validation" should "short circuit after first invalidity"
// #[tokio::test]
// async fn test_block_summary_validation_short_circuit() {
//     with_storage(|mut block_store, mut block_dag_storage| async move {
//         let mut block = create_genesis_block(
//             &mut block_store,
//             &mut block_dag_storage,
//             None,
//             None,
//             None,
//             None,
//             None,
//             Some(SHARD_ID.to_string()),
//             None,
//             None,
//         );
//
//         let mut block_modified = with_block_number(&block, 17);
//         block_modified.seq_num = 1;
//
//         let dag = block_dag_storage.get_representation();
//         let mut casper_snapshot = mk_casper_snapshot(dag);
//         let estimator = Estimator::apply(10, Some(10));
//
//         let result = Validate::block_summary(
//             &block_modified,
//             &block,
//             &mut casper_snapshot,
//             SHARD_ID,
//             i32::MAX,
//             &estimator,
//             &mut block_store,
//         )
//         .await;
//
//         assert!(result.is_err());
//         assert!(result
//             .unwrap_err()
//             .is_invalid(&InvalidBlock::InvalidBlockNumber));
//     })
//     .await
// }
//
// // Test 22: "Justification follow validation" should "return valid for proper justifications and failed otherwise"
// #[tokio::test]
// async fn test_justification_follow_validation() {
//     with_storage(|mut block_store, mut block_dag_storage| async move {
//         let v1 = generate_validator(Some("Validator One"));
//         let v2 = generate_validator(Some("Validator Two"));
//         let v1_bond = Bond {
//             validator: v1.clone(),
//             stake: 2,
//         };
//         let v2_bond = Bond {
//             validator: v2.clone(),
//             stake: 3,
//         };
//         let bonds = vec![v1_bond, v2_bond];
//
//         let genesis = create_genesis_block(
//             &mut block_store,
//             &mut block_dag_storage,
//             None,
//             Some(bonds.clone()),
//             None,
//             None,
//             None,
//             None,
//             None,
//             None,
//         );
//
//         let b2 = create_block(
//             &mut block_store,
//             &mut block_dag_storage,
//             vec![genesis.block_hash.clone()],
//             &genesis,
//             Some(v2.clone()),
//             Some(bonds.clone()),
//             None,
//             None,
//             None,
//             None,
//             None,
//             None,
//             None,
//         );
//
//         let result = Validate::justification_follows(&b2, &mut block_store);
//         assert!(result.is_ok());
//         assert_eq!(result.unwrap(), ValidBlock::Valid);
//
//         let b7 = create_block(
//             &mut block_store,
//             &mut block_dag_storage,
//             vec![genesis.block_hash.clone()],
//             &genesis,
//             Some(v1.clone()),
//             Some(vec![]),
//             None,
//             None,
//             None,
//             None,
//             None,
//             None,
//             None,
//         );
//
//         let result = Validate::justification_follows(&b7, &mut block_store);
//         assert!(result.is_err());
//         assert!(result
//             .unwrap_err()
//             .is_invalid(&InvalidBlock::InvalidFollows));
//     })
//     .await
// }
//
// // Test 23: "Justification regression validation" should "return valid for proper justifications and justification regression detected otherwise"
// #[tokio::test]
// async fn test_justification_regression_validation() {
//     with_storage(|mut block_store, mut block_dag_storage| async move {
//         let v0 = generate_validator(Some("Validator 1"));
//         let v1 = generate_validator(Some("Validator 2"));
//         let bonds = vec![
//             Bond {
//                 validator: v0.clone(),
//                 stake: 1,
//             },
//             Bond {
//                 validator: v1.clone(),
//                 stake: 3,
//             },
//         ];
//
//         let b0 = create_genesis_block(
//             &mut block_store,
//             &mut block_dag_storage,
//             None,
//             Some(bonds.clone()),
//             None,
//             None,
//             None,
//             None,
//             None,
//             None,
//         );
//
//         let b1 = create_block(
//             &mut block_store,
//             &mut block_dag_storage,
//             vec![b0.block_hash.clone()],
//             &b0,
//             Some(v0.clone()),
//             Some(bonds.clone()),
//             None,
//             None,
//             None,
//             None,
//             None,
//             None,
//             None,
//         );
//
//         let dag = block_dag_storage.get_representation();
//         let mut casper_snapshot = mk_casper_snapshot(dag);
//
//         let result = Validate::justification_regressions(&b1, &mut casper_snapshot);
//         assert!(result.is_ok());
//         assert_eq!(result.unwrap(), ValidBlock::Valid);
//     })
//     .await
// }
//
// // Test 24: "Justification regression validation" should "return valid for regressive invalid blocks"
// #[tokio::test]
// async fn test_justification_regression_validation_invalid_blocks() {
//     with_storage(|mut block_store, mut block_dag_storage| async move {
//         let v0 = generate_validator(Some("Validator 1"));
//         let v1 = generate_validator(Some("Validator 2"));
//         let bonds = vec![
//             Bond {
//                 validator: v0.clone(),
//                 stake: 1,
//             },
//             Bond {
//                 validator: v1.clone(),
//                 stake: 3,
//             },
//         ];
//
//         let b0 = create_genesis_block(
//             &mut block_store,
//             &mut block_dag_storage,
//             None,
//             Some(bonds.clone()),
//             None,
//             None,
//             None,
//             None,
//             None,
//             None,
//         );
//
//         let dag = block_dag_storage.get_representation();
//         let mut casper_snapshot = mk_casper_snapshot(dag);
//
//         let result = Validate::justification_regressions(&b0, &mut casper_snapshot);
//         assert!(result.is_ok());
//         assert_eq!(result.unwrap(), ValidBlock::Valid);
//     })
//     .await
// }
//
// // Test 25: "Bonds cache validation" should "succeed on a valid block and fail on modified bonds"
// #[tokio::test]
// async fn test_bonds_cache_validation() {
//     with_storage(|mut block_store, mut block_dag_storage| async move {
//         let genesis = create_genesis_block(
//             &mut block_store,
//             &mut block_dag_storage,
//             None,
//             Some(vec![Bond {
//                 validator: generate_validator(Some("Test")),
//                 stake: 1,
//             }]),
//             None,
//             None,
//             None,
//             None,
//             None,
//             None,
//         );
//
//         assert!(!genesis.body.state.bonds.is_empty());
//     })
//     .await
// }
//
// // Test 26: "Field format validation" should "succeed on a valid block and fail on empty fields"
// #[tokio::test]
// async fn test_format_validation() {
//     with_storage(|mut block_store, mut block_dag_storage| async move {
//         let genesis = create_genesis_block(
//             &mut block_store,
//             &mut block_dag_storage,
//             None,
//             None,
//             None,
//             None,
//             None,
//             None,
//             None,
//             None,
//         );
//
//         let result = Validate::format_of_fields(&genesis);
//         assert_eq!(result, true);
//
//         let mut invalid_block = genesis.clone();
//         invalid_block.block_hash = Bytes::new();
//         let result = Validate::format_of_fields(&invalid_block);
//         assert_eq!(result, false);
//
//         let mut invalid_block = genesis.clone();
//         invalid_block.sig = Bytes::new();
//         let result = Validate::format_of_fields(&invalid_block);
//         assert_eq!(result, false);
//
//         let mut invalid_block = genesis.clone();
//         invalid_block.sig_algorithm = String::new();
//         let result = Validate::format_of_fields(&invalid_block);
//         assert_eq!(result, false);
//
//         let mut invalid_block = genesis.clone();
//         invalid_block.shard_id = String::new();
//         let result = Validate::format_of_fields(&invalid_block);
//         assert_eq!(result, false);
//
//         let mut invalid_block = genesis.clone();
//         invalid_block.body.state.post_state_hash = Bytes::new();
//         let result = Validate::format_of_fields(&invalid_block);
//         assert_eq!(result, false);
//     })
//     .await
// }
//
// // Test 27: "Block hash format validation" should "fail on invalid hash"
// #[tokio::test]
// async fn test_block_hash_validation() {
//     with_storage(|mut block_store, mut block_dag_storage| async move {
//         let genesis = create_genesis_block(
//             &mut block_store,
//             &mut block_dag_storage,
//             None,
//             None,
//             None,
//             None,
//             None,
//             None,
//             None,
//             None,
//         );
//
//         let result = Validate::block_hash(&genesis);
//         assert!(result.is_ok());
//         assert_eq!(result.unwrap(), ValidBlock::Valid);
//
//         let mut invalid_block = genesis.clone();
//         invalid_block.block_hash = "123".as_bytes().into();
//         let result = Validate::block_hash(&invalid_block);
//         assert!(result.is_err());
//         assert!(result
//             .unwrap_err()
//             .is_invalid(&InvalidBlock::InvalidBlockHash));
//     })
//     .await
// }
//
// // Test 28: "Block version validation" should "work"
// #[tokio::test]
// async fn test_version_validation() {
//     with_storage(|mut block_store, mut block_dag_storage| async move {
//         let genesis = create_genesis_block(
//             &mut block_store,
//             &mut block_dag_storage,
//             None,
//             None,
//             None,
//             None,
//             None,
//             None,
//             None,
//             None,
//         );
//
//         let result = Validate::version(&genesis, -1);
//         assert_eq!(result, false);
//
//         let result = Validate::version(&genesis, 1);
//         assert_eq!(result, true);
//     })
//     .await
// }

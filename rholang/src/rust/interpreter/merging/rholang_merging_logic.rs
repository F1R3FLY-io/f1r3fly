// See rholang/src/main/scala/coop/rchain/rholang/interpreter/merging/RholangMergingLogic.scala

use indexmap::IndexSet;
use prost::Message;
use rayon::prelude::*;
use rspace_plus_plus::rspace::errors::HistoryError;
use std::collections::{HashMap, HashSet};

use crypto::rust::hash::blake2b512_random::Blake2b512Random;
use models::rhoapi::ListParWithRandom;
use rspace_plus_plus::rspace::hot_store_trie_action::TrieInsertAction;
use rspace_plus_plus::rspace::hot_store_trie_action::TrieInsertBinaryProduce;
use rspace_plus_plus::rspace::{
    hashing::{blake2b256_hash::Blake2b256Hash, stable_hash_provider},
    hot_store_trie_action::HotStoreTrieAction,
    internal::Datum,
    merger::channel_change::ChannelChange,
    serializers::serializers,
    trace::event::Produce,
};

use crate::rust::interpreter::rho_type::RhoNumber;

pub struct RholangMergingLogic;

impl RholangMergingLogic {
    /**
     * Transforms absolute values with the difference from initial values.
     *
     * Example for 3 state changes (A, B, C are channels, PSH is initial value/pre-state hash):
     *
     * Initial state (PSH):
     *   A = 10, B = 2, C = 20
     *
     * Final values:      Calculated diffs:
     * Change 0: A = 20   A = +10
     * Change 1: B = 5    B = +3
     * Change 2: A = 15   A = -5
     *           C = 10   C = -10
     *
     * @param channelValues Final values
     * @param getInitialValue Accessor to initial value
     */
    pub fn calculate_num_channel_diff(
        channel_values: Vec<HashMap<Blake2b256Hash, i64>>,
        get_initial_value: impl Fn(&Blake2b256Hash) -> Option<i64> + Send + Sync,
    ) -> Vec<HashMap<Blake2b256Hash, i64>> {
        // First collect unique keys while preserving order
        let unique_keys: Vec<_> = channel_values
            .iter()
            .flat_map(|channel| channel.keys().cloned())
            .collect::<IndexSet<_>>()
            .into_iter()
            .collect();

        // Get initial values for all unique keys in parallel
        let mut state = unique_keys
            .par_iter()
            .map(|key| (key.clone(), get_initial_value(key).unwrap_or(0)))
            .collect::<HashMap<_, _>>();

        // Process each channel value map
        channel_values
            .into_iter()
            .map(|end_val_map| {
                let mut diffs = HashMap::with_capacity(end_val_map.len());

                for (ch, end_val) in end_val_map {
                    if let Some(prev_val) = state.get(&ch) {
                        let diff = end_val - prev_val;
                        diffs.insert(ch.clone(), diff);
                        state.insert(ch, end_val);
                    }
                }
                diffs
            })
            .collect()
    }

    /**
     * Merge number channel value from multiple changes and base state.
     *
     * @param channelHash Channel hash
     * @param diff Difference from base state
     * @param changes Channel changes to calculate new random generator
     * @param getBaseData Base state value reader
     */
    pub fn calculate_number_channel_merge(
        channel_hash: Blake2b256Hash,
        diff: i64,
        changes: ChannelChange<prost::bytes::Bytes>,
        get_base_data: impl Fn(&Blake2b256Hash) -> Result<Vec<Datum<ListParWithRandom>>, HistoryError>
            + Send
            + Sync,
    ) -> HotStoreTrieAction<(), (), (), ()> {
        // Read initial value of number channel from base state
        let init_val_opt = Self::convert_to_read_number(get_base_data)(&channel_hash);

        // Calculate number channel new value
        let init_num = init_val_opt.unwrap_or(0);
        let new_val = init_num + diff;

        // Calculate merged random generator (use only unique changes as input)
        let new_rnd = if changes.added.iter().collect::<HashSet<_>>().len() == 1 {
            // Single branch, just use available random generator
            Self::decode_rnd(changes.added.first().unwrap().as_ref().to_vec())
        } else {
            // Multiple branches, merge random generators
            let rnd_added_sorted = changes
                .added
                .iter()
                .map(|bytes| Self::decode_rnd(bytes.as_ref().to_vec()))
                .collect::<HashSet<_>>()
                .into_iter()
                .map(|rnd| (rnd.clone(), rnd.to_bytes()))
                .collect::<Vec<_>>();

            // Sort by bytes
            let mut sorted = rnd_added_sorted;
            sorted.sort_by(|a, b| a.1.cmp(&b.1));

            // Extract sorted random generators
            let sorted_rnds = sorted.into_iter().map(|(rnd, _)| rnd).collect::<Vec<_>>();

            // Merge the random generators
            Blake2b512Random::merge(sorted_rnds)
        };

        // Create final merged value
        let datum_encoded = Self::create_datum_encoded(&channel_hash, new_val, new_rnd);

        // Create update store action
        HotStoreTrieAction::TrieInsertAction(TrieInsertAction::TrieInsertBinaryProduce(
            TrieInsertBinaryProduce {
                hash: channel_hash,
                data: vec![datum_encoded],
            },
        ))
    }

    fn decode_rnd(par_with_rnd_encoded: Vec<u8>) -> Blake2b512Random {
        let pars_with_rnd = ListParWithRandom::decode(par_with_rnd_encoded.as_ref()).unwrap();
        let rnd = Blake2b512Random::from_bytes(&pars_with_rnd.random_state);
        rnd
    }

    fn get_number_with_rnd(par_with_rnd: &ListParWithRandom) -> (i64, Blake2b512Random) {
        assert!(
            par_with_rnd.pars.len() == 1,
            "Number channel should contain single Int term, found {:?}",
            par_with_rnd.pars
        );
        let num = RhoNumber::unapply(&par_with_rnd.pars[0]).unwrap();
        (
            num,
            Blake2b512Random::from_bytes(&par_with_rnd.random_state),
        )
    }

    fn create_datum_encoded(
        channel_hash: &Blake2b256Hash,
        num: i64,
        rnd: Blake2b512Random,
    ) -> Vec<u8> {
        // Create value with random generator
        let num_par = RhoNumber::create_par(num);
        let par_with_rnd = ListParWithRandom {
            pars: vec![num_par],
            random_state: rnd.to_bytes(),
        };

        // Create hash of the data
        let data_hash =
            stable_hash_provider::hash_produce(channel_hash.bytes(), &par_with_rnd, false);

        // Create produce
        let produce = Produce {
            channel_hash: channel_hash.clone(),
            hash: data_hash,
            persistent: false,
            is_deterministic: true,
            output_value: vec![],
        };

        // Create datum
        let datum = Datum {
            a: par_with_rnd,
            persist: false,
            source: produce,
        };

        // Encode datum
        serializers::encode_datum(&datum)
    }

    /**
     * Converts function to get all data on a channel to function to get single number value.
     */
    pub fn convert_to_read_number<F>(get_data_func: F) -> impl Fn(&Blake2b256Hash) -> Option<i64>
    where
        F: Fn(&Blake2b256Hash) -> Result<Vec<Datum<ListParWithRandom>>, HistoryError>,
    {
        move |hash: &Blake2b256Hash| {
            let data = get_data_func(hash)
                .unwrap_or_else(|error| panic!("Error getting data: {:?}", error));
            assert!(
                data.len() <= 1,
                "To calculate difference on a number channel, single value is expected, found {:?}",
                data
            );
            data.first()
                .map(|datum| Self::get_number_with_rnd(&datum.a))
                .map(|(num, _)| num)
        }
    }
}

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct DeployMergeableData {
    pub channels: Vec<NumberChannel>,
}

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct NumberChannel {
    pub hash: Blake2b256Hash,
    pub diff: i64,
}

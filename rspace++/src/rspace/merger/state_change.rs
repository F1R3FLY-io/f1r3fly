// See rspace/src/main/scala/coop/rchain/rspace/merger/StateChange.scala

use dashmap::DashMap;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};

use crate::rspace::{
    errors::HistoryError,
    hashing::{blake2b256_hash::Blake2b256Hash, stable_hash_provider},
    history::{
        history_reader::HistoryReader,
        instances::rspace_history_reader_impl::RSpaceHistoryReaderImpl,
    },
};

use super::{
    channel_change::ChannelChange,
    event_log_index::EventLogIndex,
    merging_logic::{consumes_affected, produces_affected},
};

/**
 * Datum changes are referenced by channel, continuation changes are references by consume.
 * In addition, map from consume channels to binary representation of a join in trie have to be maintained.
 * This is because only hashes of channels are available in log event, and computing a join binary to be
 * inserted or removed on merge requires channels before hashing.
 */
pub struct StateChange {
    pub datums_changes: DashMap<Blake2b256Hash, ChannelChange<Vec<u8>>>,
    pub cont_changes: DashMap<Vec<Blake2b256Hash>, ChannelChange<Vec<u8>>>,
    pub consume_channels_to_join_serialized_map: DashMap<Vec<Blake2b256Hash>, Vec<u8>>,
}

impl StateChange {
    pub fn new<C, P, A, K>(
        pre_state_reader: RSpaceHistoryReaderImpl<C, P, A, K>,
        post_state_reader: RSpaceHistoryReaderImpl<C, P, A, K>,
        event_log_index: EventLogIndex,
    ) -> Result<Self, HistoryError>
    where
        C: Clone + for<'a> Deserialize<'a> + Serialize + 'static + Sync + Send,
        P: Clone + for<'a> Deserialize<'a> + 'static + Sync + Send,
        A: Clone + for<'a> Deserialize<'a> + 'static + Sync + Send,
        K: Clone + for<'a> Deserialize<'a> + 'static + Sync + Send,
    {
        let datums_diff = DashMap::new();
        let cont_diff = DashMap::new();
        // Since event log only contains hashes of channels, so to know which join stored corresponds to channels,
        // this index have to be maintained
        let joins_map = DashMap::new();

        let produces_affected = produces_affected(&event_log_index);
        let channels_of_consumes_affected = consumes_affected(&event_log_index)
            .0
            .into_iter()
            .map(|consume| consume.channel_hashes)
            .collect::<Vec<_>>();

        // Process produces in parallel
        produces_affected.0.par_iter().try_for_each(|produce| {
            let history_pointer = produce.channel_hash.clone();

            let change = Self::compute_value_change(
                &history_pointer,
                |h| pre_state_reader.get_data_proj_binary(h),
                |h| post_state_reader.get_data_proj_binary(h),
            )?;

            let mut curr_val = datums_diff
                .entry(history_pointer)
                .or_insert(ChannelChange::empty());
            curr_val.added.extend(change.added);
            curr_val.removed.extend(change.removed);

            Ok::<(), HistoryError>(())
        })?;

        // Process consumes in parallel
        channels_of_consumes_affected
            .par_iter()
            .try_for_each(|consume_channels| {
                let consume_channels = consume_channels.clone();
                let history_pointer = stable_hash_provider::hash_from_hashes(&consume_channels);

                let change = Self::compute_value_change(
                    &history_pointer,
                    |h| pre_state_reader.get_continuations_proj_binary(h),
                    |h| post_state_reader.get_continuations_proj_binary(h),
                )?;

                let mut curr_val = cont_diff
                    .entry(consume_channels)
                    .or_insert(ChannelChange::empty());
                curr_val.added.extend(change.added);
                curr_val.removed.extend(change.removed);

                Ok::<(), HistoryError>(())
            })?;

        // Process joins in parallel
        channels_of_consumes_affected.par_iter().try_for_each(|consume_channels| {
            let mut consume_channels = consume_channels.clone();
            let history_pointer = consume_channels[0].clone();
            let pre = pre_state_reader.get_joins(&history_pointer)?;
            let post = post_state_reader.get_joins(&history_pointer)?;

            // find join which match channels
            let join = pre
                .into_iter()
                .chain(post)
                .find(|join| {
                    let mut join_channels = join
                        .iter()
                        .map(|item| stable_hash_provider::hash(item))
                        .collect::<Vec<_>>();
                    // sorting is required because channels of a consume in event log and channels of a join in
                    // history might not be ordered the same way
                    consume_channels.sort();
                    join_channels.sort();
                    *consume_channels == join_channels
                })
                .expect("Tuple space inconsistency found: channel of consume does not contain join record corresponding to the consume channels.");

            let raw_join = bincode::serialize(&join)
                .expect("Unable to serialize join");
            joins_map.insert(consume_channels, raw_join);
            Ok::<(), HistoryError>(())
        })?;

        // Check for errors - empty changes
        let has_empty_produce_changes = datums_diff.iter().any(|entry| {
            let change = entry.value();
            change.added.is_empty() && change.removed.is_empty()
        });

        if has_empty_produce_changes {
            return Err(HistoryError::MergeError(
                "State change compute logic error: empty channel change for produce.".to_string(),
            ));
        }

        let has_empty_consume_changes = cont_diff.iter().any(|entry| {
            let change = entry.value();
            change.added.is_empty() && change.removed.is_empty()
        });

        if has_empty_consume_changes {
            return Err(HistoryError::MergeError(
                "State change compute logic error: empty channel change for consume.".to_string(),
            ));
        }

        Ok(Self {
            datums_changes: datums_diff,
            cont_changes: cont_diff,
            consume_channels_to_join_serialized_map: joins_map,
        })
    }

    fn compute_value_change(
        history_pointer: &Blake2b256Hash,
        start_value: impl Fn(&Blake2b256Hash) -> Result<Vec<Vec<u8>>, HistoryError>,
        end_value: impl Fn(&Blake2b256Hash) -> Result<Vec<Vec<u8>>, HistoryError>,
    ) -> Result<ChannelChange<Vec<u8>>, HistoryError> {
        let start = start_value(history_pointer)?;
        let end = end_value(history_pointer)?;

        let added = end
            .iter()
            .filter(|item| !start.contains(item))
            .cloned()
            .collect::<Vec<_>>();

        let deleted = start
            .iter()
            .filter(|item| !end.contains(item))
            .cloned()
            .collect::<Vec<_>>();

        Ok(ChannelChange {
            added,
            removed: deleted,
        })
    }

    pub fn empty() -> Self {
        Self {
            datums_changes: DashMap::new(),
            cont_changes: DashMap::new(),
            consume_channels_to_join_serialized_map: DashMap::new(),
        }
    }

    pub fn combine(self, other: Self) -> Self {
        let datums_changes = self.datums_changes;
        let cont_changes = self.cont_changes;
        let consume_channels_to_join_serialized_map = self.consume_channels_to_join_serialized_map;

        // Combine datum changes
        for (key, value) in other.datums_changes {
            match datums_changes.entry(key) {
                dashmap::mapref::entry::Entry::Occupied(mut entry) => {
                    let current = entry.get_mut();
                    current.added.extend(value.added);
                    current.removed.extend(value.removed);
                }
                dashmap::mapref::entry::Entry::Vacant(entry) => {
                    entry.insert(value);
                }
            }
        }

        // Combine continuation changes
        for (key, value) in other.cont_changes {
            match cont_changes.entry(key) {
                dashmap::mapref::entry::Entry::Occupied(mut entry) => {
                    let current = entry.get_mut();
                    current.added.extend(value.added);
                    current.removed.extend(value.removed);
                }
                dashmap::mapref::entry::Entry::Vacant(entry) => {
                    entry.insert(value);
                }
            }
        }

        // Combine join maps (newer values take precedence)
        for (key, value) in other.consume_channels_to_join_serialized_map {
            consume_channels_to_join_serialized_map.insert(key, value);
        }

        Self {
            datums_changes,
            cont_changes,
            consume_channels_to_join_serialized_map,
        }
    }
}

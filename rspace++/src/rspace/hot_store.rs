// use crate::rspace::history_reader_base::{HistoryReaderBase, HistoryReaderBaseImpl};
use crate::rspace::internal::{Datum, Row, WaitingContinuation};
use dashmap::DashMap;
use std::collections::HashMap;
use std::fmt::Debug;
// use futures::channel::oneshot;
use std::hash::Hash;
use std::sync::{Arc, Mutex};
// use tokio::sync::Mutex;
use crate::rspace::hot_store_action::{
    DeleteAction, DeleteContinuations, DeleteData, DeleteJoins, HotStoreAction, InsertAction,
    InsertContinuations, InsertData, InsertJoins,
};

// See rspace/src/main/scala/coop/rchain/rspace/HotStore.scala
pub trait HotStore<C, P, A, K> {
    fn get_continuations(&self, channels: Vec<C>) -> Vec<WaitingContinuation<P, K>>;
    fn put_continuation(&self, channels: Vec<C>, wc: WaitingContinuation<P, K>) -> Option<()>;
    fn install_continuation(&self, channels: Vec<C>, wc: WaitingContinuation<P, K>) -> Option<()>;
    fn remove_continuation(&self, channels: Vec<C>, index: i32) -> Option<()>;

    fn get_data(&self, channel: &C) -> Vec<Datum<A>>;
    fn put_datum(&self, channel: C, d: Datum<A>) -> ();
    fn remove_datum(&self, channel: C, index: i32) -> Option<()>;

    fn get_joins(&self, channel: C) -> Vec<Vec<C>>;
    fn put_join(&self, channel: C, join: Vec<C>) -> Option<()>;
    fn install_join(&self, channel: C, join: Vec<C>) -> Option<()>;
    fn remove_join(&self, channel: C, join: Vec<C>) -> Option<()>;

    fn changes(&self) -> Vec<HotStoreAction<C, P, A, K>>;
    fn to_map(&self) -> HashMap<Vec<C>, Row<P, A, K>>;
}

struct HotStoreState<C, P, A, K> {
    continuations: DashMap<Vec<C>, Vec<WaitingContinuation<P, K>>>,
    installed_continuations: DashMap<Vec<C>, WaitingContinuation<P, K>>,
    data: DashMap<C, Vec<Datum<A>>>,
    joins: DashMap<C, Vec<Vec<C>>>,
    installed_joins: DashMap<C, Vec<Vec<C>>>,
}

// struct HistoryStoreCache<C, P, A, K> {
//     continuations: DashMap<Vec<C>, oneshot::Sender<Vec<WaitingContinuation<P, K>>>>,
//     datums: DashMap<C, oneshot::Sender<Vec<Datum<A>>>>,
//     joins: DashMap<C, oneshot::Sender<Vec<Vec<C>>>>,
// }

pub struct InMemConcHotStore<C, P, A, K> {
    hot_store_state: Arc<Mutex<HotStoreState<C, P, A, K>>>,
    // history_store_cache: Arc<Mutex<HistoryStoreCache<C, P, A, K>>>,
    // history_reader_base: Arc<Mutex<HistoryReaderBaseImpl<C, P, A, K>>>,
}

// See rspace/src/main/scala/coop/rchain/rspace/HotStore.scala
impl<C: Hash + Eq + Clone + Debug, P: Clone + Debug, A: Clone + Debug, K: Clone + Debug>
    HotStore<C, P, A, K> for InMemConcHotStore<C, P, A, K>
{
    fn get_continuations(&self, channels: Vec<C>) -> Vec<WaitingContinuation<P, K>> {
        // This is STUBBED out. Ideally this comes from history store
        let from_history_store: Vec<WaitingContinuation<P, K>> = Vec::new();

        let maybe_continuations = {
            let state = self.hot_store_state.lock().unwrap();
            state
                .continuations
                .get(&channels)
                .map(|continuations| continuations.clone())
        };

        let maybe_installed_continuation = {
            let state = self.hot_store_state.lock().unwrap();
            state
                .installed_continuations
                .get(&channels)
                .map(|continuations| continuations.clone())
        };

        match maybe_continuations {
            Some(continuations) => match maybe_installed_continuation {
                Some(installed_continuation) => {
                    let mut result = vec![installed_continuation];
                    result.extend(continuations);
                    result
                }
                None => continuations,
            },
            None => {
                self.hot_store_state
                    .lock()
                    .unwrap()
                    .continuations
                    .insert(channels, from_history_store.clone());
                match maybe_installed_continuation {
                    Some(installed_continuation) => {
                        let mut result = vec![installed_continuation];
                        result.extend(from_history_store);
                        result
                    }
                    None => from_history_store,
                }
            }
        }
    }

    fn put_continuation(&self, channels: Vec<C>, wc: WaitingContinuation<P, K>) -> Option<()> {
        // println!("\nHit put_continuation");

        // This is STUBBED out. Ideally this comes from history store
        let from_history_store: Vec<WaitingContinuation<P, K>> = Vec::new();
        // println!("\nfrom_history_store: {:?}", from_history_store);

        let state = self.hot_store_state.lock().unwrap();
        let current_continuations = state
            .continuations
            .get(&channels)
            .map(|c| c.clone())
            .unwrap_or(from_history_store);
        let new_continuations = vec![wc]
            .into_iter()
            .chain(current_continuations.into_iter())
            .collect();
        state.continuations.insert(channels, new_continuations);
        Some(())
    }

    fn install_continuation(&self, channels: Vec<C>, wc: WaitingContinuation<P, K>) -> Option<()> {
        // println!("hit install_continuation");
        let state = self.hot_store_state.lock().unwrap();
        let result = state.installed_continuations.insert(channels, wc);

        // println!("installed_continuation result: {:?}", result);
        // println!("to_map: {:?}\n", self.print());

        match result {
            Some(_) => Some(()),
            None => None,
        }
    }

    fn remove_continuation(&self, channels: Vec<C>, index: i32) -> Option<()> {
        // This is STUBBED out. Ideally this comes from history store
        let from_history_store: Vec<WaitingContinuation<P, K>> = Vec::new();

        let state = self.hot_store_state.lock().unwrap();
        let current_continuations = state
            .continuations
            .get(&channels)
            .map(|c| c.clone())
            .unwrap_or(from_history_store);
        let installed_continuation = state.installed_continuations.get(&channels);
        let is_installed = installed_continuation.is_some();

        let removing_installed = is_installed && index == 0;
        let removed_index = if is_installed { index - 1 } else { index };
        let out_of_bounds =
            removed_index < 0 || removed_index as usize >= current_continuations.len();

        if removing_installed {
            println!("ERROR: Attempted to remove an installed continuation");
            state.continuations.insert(channels, current_continuations);
            None
        } else if out_of_bounds {
            println!("ERROR: Index {index} out of bounds when removing continuation");
            state.continuations.insert(channels, current_continuations);
            None
        } else {
            let mut new_continuations = current_continuations;
            new_continuations.remove(removed_index as usize);
            state.continuations.insert(channels, new_continuations);
            Some(())
        }
    }

    fn get_data(&self, channel: &C) -> Vec<Datum<A>> {
        // This is STUBBED out. Ideally this comes from history store
        let from_history_store: Vec<Datum<A>> = Vec::new();

        let maybe_data = {
            let state = self.hot_store_state.lock().unwrap();
            state.data.get(channel).map(|data| data.clone())
        };

        match maybe_data {
            Some(data) => data,
            None => {
                self.hot_store_state
                    .lock()
                    .unwrap()
                    .data
                    .insert(channel.clone(), from_history_store.clone());
                from_history_store
            }
        }
    }

    fn put_datum(&self, channel: C, d: Datum<A>) -> () {
        // println!("\nHit put_datum");

        // This is STUBBED out. Ideally this comes from cold store
        let from_history_store: Vec<Datum<A>> = Vec::new();
        // println!(
        //     "\nfrom_history_store in put_datum: {:?}",
        //     from_history_store
        // );

        let state = self.hot_store_state.lock().unwrap();
        let mut update_data = state
            .data
            .get(&channel)
            .map(|d| d.clone())
            .unwrap_or(from_history_store);
        update_data.insert(0, d);
        let _ = state.data.insert(channel, update_data);
    }

    fn remove_datum(&self, channel: C, index: i32) -> Option<()> {
        // This is STUBBED out. Ideally this comes from history store
        let from_history_store: Vec<Datum<A>> = Vec::new();

        let state = self.hot_store_state.lock().unwrap();
        let current_datums = state
            .data
            .get(&channel)
            .map(|c| c.clone())
            .unwrap_or(from_history_store);
        let out_of_bounds = index as usize >= current_datums.len();

        if out_of_bounds {
            println!("ERROR: Index {index} out of bounds when removing datum");
            state.data.insert(channel, current_datums);
            None
        } else {
            let mut new_datums = current_datums;
            new_datums.remove(index as usize);
            state.data.insert(channel, new_datums);
            Some(())
        }
    }

    fn get_joins(&self, channel: C) -> Vec<Vec<C>> {
        // println!("\nHit get_joins");

        // This is STUBBED out. Ideally this comes from history store
        let from_history_store: Vec<Vec<C>> = Vec::new();
        // println!(
        //     "\nfrom_history_store in get_joins: {:?}",
        //     from_history_store
        // );

        let maybe_joins = {
            let state = self.hot_store_state.lock().unwrap();
            state.joins.get(&channel).map(|joins| joins.clone())
        };

        match maybe_joins {
            Some(joins) => {
                // println!("Found joins in store");
                let installed_joins = {
                    let state = self.hot_store_state.lock().unwrap();
                    state
                        .installed_joins
                        .get(&channel)
                        .map(|joins| joins.clone())
                        .unwrap_or_default()
                };

                let mut result = installed_joins;
                result.extend(joins);
                result
            }
            None => {
                // println!("No joins found in store");
                {
                    let state = self.hot_store_state.lock().unwrap();
                    state
                        .joins
                        .insert(channel.clone(), from_history_store.clone());
                }
                // println!("Inserted into store. Returning from history");

                let installed_joins = {
                    let state = self.hot_store_state.lock().unwrap();
                    state
                        .installed_joins
                        .get(&channel)
                        .map(|joins| joins.clone())
                        .unwrap_or_default()
                };

                let mut result = installed_joins;
                result.extend(from_history_store);
                result
            }
        }
    }

    fn put_join(&self, channel: C, join: Vec<C>) -> Option<()> {
        // This is STUBBED out. Ideally this comes from history store
        let from_history_store: Vec<Vec<C>> = Vec::new();

        let state = self.hot_store_state.lock().unwrap();
        let current_joins = state
            .joins
            .get(&channel)
            .map(|j| j.clone())
            .unwrap_or(from_history_store);
        if current_joins.contains(&join) {
            Some(())
        } else {
            let new_joins = vec![join]
                .into_iter()
                .chain(current_joins.into_iter())
                .collect();
            state.joins.insert(channel, new_joins);
            Some(())
        }
    }

    fn install_join(&self, channel: C, join: Vec<C>) -> Option<()> {
        // println!("hit install_join");
        let state = self.hot_store_state.lock().unwrap();
        let current_installed_joins = state
            .installed_joins
            .get(&channel)
            .map(|c| c.clone())
            .unwrap_or(Vec::new())
            .clone();
        if !current_installed_joins.contains(&join) {
            let mut new_installed_joins = current_installed_joins;
            new_installed_joins.push(join);
            let _ = state.installed_joins.insert(channel, new_installed_joins);
        }
        Some(())
    }

    fn remove_join(&self, channel: C, join: Vec<C>) -> Option<()> {
        // This is STUBBED out. Ideally this comes from history store
        let joins_in_history_store: Vec<Vec<C>> = Vec::new();
        let continuations_in_history_store: Vec<WaitingContinuation<P, K>> = Vec::new();

        let state = self.hot_store_state.lock().unwrap();
        let current_joins = state
            .joins
            .get(&channel)
            .map(|j| j.clone())
            .unwrap_or(joins_in_history_store);

        let current_continuations = {
            let mut conts = state
                .installed_continuations
                .get(&join)
                .map(|c| vec![c.clone()])
                .unwrap_or_else(Vec::new);
            conts.extend(
                state
                    .continuations
                    .get(&join)
                    .map(|continuations| continuations.clone())
                    .unwrap_or(continuations_in_history_store),
            );
            conts
        };

        let index = current_joins.iter().position(|x| *x == join);
        let out_of_bounds = index.is_none();
        let do_remove = current_continuations.is_empty();

        if do_remove {
            if out_of_bounds {
                println!("ERROR: Join not found when removing join");
                state.joins.insert(channel, current_joins);
                None
            } else {
                let mut new_joins = current_joins;
                new_joins.remove(index.unwrap());
                state.joins.insert(channel, new_joins);
                Some(())
            }
        } else {
            state.joins.insert(channel, current_joins);
            None
        }
    }

    fn changes(&self) -> Vec<HotStoreAction<C, P, A, K>> {
        let cache = self.hot_store_state.lock().unwrap();
        let continuations: Vec<HotStoreAction<C, P, A, K>> = cache
            .continuations
            .clone()
            .into_iter()
            .map(|(k, v)| {
                if v.is_empty() {
                    HotStoreAction::Delete(DeleteAction::DeleteContinuations(DeleteContinuations {
                        channels: k,
                    }))
                } else {
                    HotStoreAction::Insert(InsertAction::InsertContinuations(InsertContinuations {
                        channels: k,
                        continuations: v,
                    }))
                }
            })
            .collect();

        let data: Vec<HotStoreAction<C, P, A, K>> = cache
            .data
            .clone()
            .into_iter()
            .map(|(k, v)| {
                if v.is_empty() {
                    HotStoreAction::Delete(DeleteAction::DeleteData(DeleteData { channel: k }))
                } else {
                    HotStoreAction::Insert(InsertAction::InsertData(InsertData {
                        channel: k,
                        data: v,
                    }))
                }
            })
            .collect();

        let joins: Vec<HotStoreAction<C, P, A, K>> = cache
            .joins
            .clone()
            .into_iter()
            .map(|(k, v)| {
                if v.is_empty() {
                    HotStoreAction::Delete(DeleteAction::DeleteJoins(DeleteJoins { channel: k }))
                } else {
                    HotStoreAction::Insert(InsertAction::InsertJoins(InsertJoins {
                        channel: k,
                        joins: v,
                    }))
                }
            })
            .collect();

        [continuations, data, joins].concat()
    }

    fn to_map(&self) -> HashMap<Vec<C>, Row<P, A, K>> {
        let state = self.hot_store_state.lock().unwrap();
        let data = state
            .data
            .iter()
            .map(|entry| {
                let (k, v) = entry.pair();
                (vec![k.clone()], v.clone())
            })
            .collect::<HashMap<_, _>>();

        let all_continuations = {
            let mut all = state
                .continuations
                .iter()
                .map(|entry| {
                    let (k, v) = entry.pair();
                    (k.clone(), v.clone())
                })
                .collect::<HashMap<_, _>>();
            for (k, v) in state.installed_continuations.iter().map(|entry| {
                let (k, v) = entry.pair();
                (k.clone(), v.clone())
            }) {
                all.entry(k).or_insert_with(Vec::new).push(v);
            }
            all
        };

        let mut map = HashMap::new();

        for (k, v) in data.into_iter() {
            let row = Row {
                data: v,
                wks: all_continuations.get(&k).cloned().unwrap_or_else(Vec::new),
            };
            if !(row.data.is_empty() && row.wks.is_empty()) {
                map.insert(k, row);
            }
        }
        map
    }
}

// See rspace/src/main/scala/coop/rchain/rspace/HotStore.scala
impl<C: Hash + Eq + Debug, P: Debug, A: Debug, K: Debug> InMemConcHotStore<C, P, A, K> {
    pub fn create() -> InMemConcHotStore<C, P, A, K> {
        InMemConcHotStore {
            hot_store_state: Arc::new(Mutex::new(HotStoreState {
                continuations: DashMap::new(),
                installed_continuations: DashMap::new(),
                data: DashMap::new(),
                joins: DashMap::new(),
                installed_joins: DashMap::new(),
            })),
        }
    }

    pub fn print(&self) {
        let state = self.hot_store_state.lock().unwrap();
        println!("\nCurrent Store:");

        println!("Continuations:");
        for entry in state.continuations.iter() {
            let (key, value) = entry.pair();
            println!("Key: {:?}, Value: {:?}", key, value);
        }

        println!("\nInstalled Continuations:");
        for entry in state.installed_continuations.iter() {
            let (key, value) = entry.pair();
            println!("Key: {:?}, Value: {:?}", key, value);
        }

        println!("\nData:");
        for entry in state.data.iter() {
            let (key, value) = entry.pair();
            println!("Key: {:?}, Value: {:?}", key, value);
        }

        println!("\nJoins:");
        for entry in state.joins.iter() {
            let (key, value) = entry.pair();
            println!("Key: {:?}, Value: {:?}", key, value);
        }

        println!("\nInstalled Joins:");
        for entry in state.installed_joins.iter() {
            let (key, value) = entry.pair();
            println!("Key: {:?}, Value: {:?}", key, value);
        }
        println!();
    }

    pub fn clear(&self) {
        let mut state = self.hot_store_state.lock().unwrap();
        state.continuations = DashMap::new();
        state.installed_continuations = DashMap::new();
        state.data = DashMap::new();
        state.joins = DashMap::new();
        state.installed_joins = DashMap::new();
    }
}

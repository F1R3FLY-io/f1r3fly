#![allow(dead_code)]

use crate::rtypes::rtypes;
use dashmap::DashMap;
use prost::Message;
use std::collections::hash_map::DefaultHasher;
use std::error::Error;
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;

pub struct MemConcDB<D: Message, K: Message> {
    db: DashMap<String, Vec<u8>>,
    phantom: PhantomData<(D, K)>,
}

impl<
        D: Clone + std::hash::Hash + std::fmt::Debug + std::default::Default + prost::Message,
        K: Clone + std::hash::Hash + std::fmt::Debug + std::default::Default + prost::Message,
    > MemConcDB<D, K>
{
    pub fn create() -> Result<MemConcDB<D, K>, Box<dyn Error>> {
        let db = DashMap::new();

        Ok(MemConcDB {
            db,
            phantom: PhantomData,
        })
    }

    pub fn consume(
        &self,
        commit: rtypes::Commit,
        persistent: bool,
    ) -> Option<Vec<rtypes::OptionResult>> {
        if commit.channels.len() == commit.patterns.len() {
            let mut results: Vec<rtypes::OptionResult> = vec![];
            let mut should_break = false;

            for i in 0..commit.channels.len() {
                let data_prefix = format!("channel-{}-data", commit.channels[i]);
                let mut key_to_delete: String = String::from("");

                for ele in self.db.iter() {
                    // println!("memconc consume Key: {:?}", ele.key());
                    if ele.key().starts_with(&data_prefix) {
                        // println!("memconc consume has prefix: {:?}", ele.key());
                        let rcdata =
                            rtypes::RetrieveContinuation::decode(ele.value().as_slice()).unwrap();
                        if commit.patterns[i] == rcdata.match_case {
                            // println!("memconc pattern match: {:?}", rcdata.match_case);

                            let mut option_result = rtypes::OptionResult::default();
                            option_result.continuation = commit.continuation.clone();
                            option_result.data = rcdata.data.clone();

                            results.push(option_result);

                            should_break = true;
                            if !rcdata.persistent {
                                key_to_delete = ele.key().to_owned();
                            }
                        }
                    }
                    if should_break {
                        break;
                    }
                }
                if key_to_delete != "" {
                    // println!("key_to_delete: {:?}", key_to_delete);
                    self.db.remove(&key_to_delete);
                }
            }

            if results.len() > 0 {
                return Some(results);
            } else {
                for i in 0..commit.channels.len() {
                    let mut commitcont_data = rtypes::CommitContinuation::default();
                    commitcont_data.pattern = commit.patterns[i].clone();
                    commitcont_data.continuation = commit.continuation.clone();
                    commitcont_data.persistent = persistent;

                    // println!("\nNo matching data for {:?}", commit);

                    let data_hash = self.calculate_hash(&commitcont_data);
                    let key = format!(
                        "channel-{}-continuation-{}",
                        &commit.channels[i], &data_hash
                    );

                    let mut commitcont_data_buf = Vec::new();
                    commitcont_data_buf.reserve(commitcont_data.encoded_len());
                    commitcont_data.encode(&mut commitcont_data_buf).unwrap();

                    // returns old key if one was found
                    let _old_key = self.db.insert(key, commitcont_data_buf);
                }

                None
            }
        } else {
            println!("channel and pattern vectors are not equal length!");
            None
        }
    }

    pub fn produce(
        &self,
        retrieve: rtypes::Retrieve,
        persistent: bool,
    ) -> Option<rtypes::OptionResult> {
        let continuation_prefix = format!("channel-{}-continuation", retrieve.chan);
        let mut result = None;
        let mut key_to_delete: String = String::from("");
        let mut should_break = false;
        for ele in self.db.iter() {
            // println!("Key: {:?}", ele.key());
            if ele.key().starts_with(&continuation_prefix) {
                let ccdata = rtypes::CommitContinuation::decode(ele.value().as_slice()).unwrap();
                // println!("has prefix: {:?}", ele.key());

                if ccdata.pattern == retrieve.match_case {
                    // println!("memconc has match {:?}", ccdata.pattern);

                    let mut option_result = rtypes::OptionResult::default();
                    option_result.continuation = ccdata.continuation.clone();
                    option_result.data = retrieve.data.clone();

                    result = Some(option_result);
                    should_break = true;
                    if !ccdata.persistent {
                        key_to_delete = ele.key().to_owned();
                    }
                }
            }
            if should_break {
                break;
            }
        }

        if key_to_delete != "" {
            // println!("key_to_delete: {:?}", key_to_delete);
            self.db.remove(&key_to_delete);
        }

        //printout check
        // self.db.retain(|key, value| {
        //     println!("retain keyval {:?}", key);
        //     return true;
        // });

        if result.is_some() {
            return result;
        } else {
            let mut retrievecont_data = rtypes::RetrieveContinuation::default();
            retrievecont_data.data = retrieve.data.clone();
            retrievecont_data.match_case = retrieve.match_case.clone();
            retrievecont_data.persistent = persistent;

            // println!("\nNo matching continuation for {:?}", retrieve);

            let data_hash = self.calculate_hash(&retrievecont_data);
            let key = format!("channel-{}-data-{}", &retrieve.chan, &data_hash);

            let mut retrievecont_data_buf = Vec::new();
            retrievecont_data_buf.reserve(retrievecont_data.encoded_len());
            retrievecont_data
                .encode(&mut retrievecont_data_buf)
                .unwrap();

            // returns old key if one was found
            let _old_key = self.db.insert(key, retrievecont_data_buf);

            None
        }
    }

    pub fn print_channel(&self, channel: &str) -> Result<(), Box<dyn Error>> {
        if !self.db.is_empty() {
            println!("\nCurrent store state:");

            let continuation_prefix = format!("channel-{}-continuation", channel);
            let data_prefix = format!("channel-{}-data", channel);

            for entry in self.db.iter() {
                let data_buf = entry.value();
                let key = entry.key();

                if key.starts_with(&continuation_prefix) {
                    let ccdata = rtypes::CommitContinuation::decode(data_buf.as_slice()).unwrap();
                    println!("KEY: {:?} VALUE: {:?}", key, ccdata);
                } else if key.starts_with(&data_prefix) {
                    let rcdata = rtypes::RetrieveContinuation::decode(data_buf.as_slice()).unwrap();
                    println!("KEY: {:?} VALUE: {:?}", key, rcdata);
                } else {
                    println!("KEY: {:?} VALUE: {:?}", key, data_buf);
                }
            }
        } else {
            println!("\nDatabase is empty")
        }

        Ok(())
    }

    pub fn is_empty(&self) -> bool {
        return self.db.is_empty();
    }

    pub fn clear(&self) -> Result<(), Box<dyn Error>> {
        let _ = self.db.clear();
        Ok(())
    }

    fn calculate_hash<T: Hash>(&self, t: &T) -> u64 {
        let mut s = DefaultHasher::new();
        t.hash(&mut s);
        s.finish()
    }
}

#![allow(dead_code)]

use crate::rtypes::rtypes;
use heed::types::*;
use heed::{Database, Env, EnvOpenOptions};
use prost::Message;
use std::collections::hash_map::DefaultHasher;
use std::error::Error;
use std::fs;
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;
use std::path::Path;

/*
See RSpace.scala and Tuplespace.scala in rspace/
*/
pub struct DiskConcDB<D: Message, K: Message> {
    env: Env,
    db: Database<Str, SerdeBincode<Vec<u8>>>,
    phantom: PhantomData<(D, K)>,
}

impl<
        D: Clone + std::hash::Hash + std::fmt::Debug + std::default::Default + prost::Message,
        K: Clone + std::hash::Hash + std::fmt::Debug + std::default::Default + prost::Message,
    > DiskConcDB<D, K>
{
    pub fn create() -> Result<DiskConcDB<D, K>, Box<dyn Error>> {
        fs::create_dir_all(Path::new("target").join("DiskConcDB"))?;
        let env = EnvOpenOptions::new().open(Path::new("target").join("DiskConcDB"))?;

        // open the default unamed database
        let db = env.create_database(None)?;

        Ok(DiskConcDB {
            env,
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
            let rtxn = self.env.read_txn().unwrap();

            for i in 0..commit.channels.len() {
                let data_prefix = format!("channel-{}-data", commit.channels[i]);
                let mut iter_data = self.db.prefix_iter(&rtxn, &data_prefix).unwrap();
                let mut iter_data_option = iter_data.next().transpose().unwrap();

                while iter_data_option.is_some() {
                    let iter_data_unwrap = iter_data_option.unwrap();
                    let rcdata_buf = iter_data_unwrap.1;
                    let rcdata =
                        rtypes::RetrieveContinuation::decode(rcdata_buf.as_slice()).unwrap();

                    if commit.patterns[i] == rcdata.match_case {
                        if !rcdata.persistent {
                            let mut wtxn = self.env.write_txn().unwrap();
                            let _ = self.db.delete(&mut wtxn, iter_data_unwrap.0);
                            wtxn.commit().unwrap();
                        }

                        let mut option_result = rtypes::OptionResult::default();
                        option_result.continuation = commit.continuation.clone();
                        option_result.data = rcdata.data.clone();

                        results.push(option_result);
                        break;
                    }
                    iter_data_option = iter_data.next().transpose().unwrap();
                }
                drop(iter_data);
            }
            rtxn.commit().unwrap();

            if results.len() > 0 {
                return Some(results);
            } else {
                for i in 0..commit.channels.len() {
                    let mut commitcont_data = rtypes::CommitContinuation::default();
                    commitcont_data.pattern = commit.patterns[i].clone();
                    commitcont_data.continuation = commit.continuation.clone();
                    commitcont_data.persistent = persistent;

                    // println!("\nNo matching data for {:?}", commit);

                    // opening a write transaction
                    let mut wtxn = self.env.write_txn().unwrap();

                    let data_hash = self.calculate_hash(&commitcont_data);
                    let key = format!(
                        "channel-{}-continuation-{}",
                        &commit.channels[i], &data_hash
                    );

                    let mut commitcont_data_buf = Vec::new();
                    commitcont_data_buf.reserve(commitcont_data.encoded_len());
                    commitcont_data.encode(&mut commitcont_data_buf).unwrap();

                    let _ = self.db.put(&mut wtxn, &key, &commitcont_data_buf);
                    wtxn.commit().unwrap();
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
        let rtxn = self.env.read_txn().unwrap();

        let continuation_prefix = format!("channel-{}-continuation", retrieve.chan);
        let mut iter_continuation = self.db.prefix_iter(&rtxn, &continuation_prefix).unwrap();
        let mut iter_continuation_option = iter_continuation.next().transpose().unwrap();

        while iter_continuation_option.is_some() {
            let iter_data = iter_continuation_option.unwrap();
            let ccdata_buf = iter_data.1;
            let ccdata = rtypes::CommitContinuation::decode(ccdata_buf.as_slice()).unwrap();

            if ccdata.pattern == retrieve.match_case {
                if !ccdata.persistent {
                    let mut wtxn = self.env.write_txn().unwrap();
                    let _ = self.db.delete(&mut wtxn, iter_data.0);
                    wtxn.commit().unwrap();
                }

                let mut option_result = rtypes::OptionResult::default();
                option_result.continuation = ccdata.continuation.clone();
                option_result.data = retrieve.data.clone();

                return Some(option_result);
            }
            iter_continuation_option = iter_continuation.next().transpose().unwrap();
        }
        drop(iter_continuation);
        rtxn.commit().unwrap();

        let mut retrievecont_data = rtypes::RetrieveContinuation::default();
        retrievecont_data.data = retrieve.data.clone();
        retrievecont_data.match_case = retrieve.match_case.clone();
        retrievecont_data.persistent = persistent;

        // println!("\nNo matching continuation for {:?}", send);

        let mut wtxn = self.env.write_txn().unwrap();

        let data_hash = self.calculate_hash(&retrievecont_data);
        let key = format!("channel-{}-data-{}", &retrieve.chan, &data_hash);

        let mut retrievecont_data_buf = Vec::new();
        retrievecont_data_buf.reserve(retrievecont_data.encoded_len());
        retrievecont_data
            .encode(&mut retrievecont_data_buf)
            .unwrap();

        let _ = self.db.put(&mut wtxn, &key, &retrievecont_data_buf);
        wtxn.commit().unwrap();

        None
    }

    pub fn print_channel(&self, channel: &str) -> Result<(), Box<dyn Error>> {
        let rtxn = self.env.read_txn()?;

        let continuation_prefix = format!("channel-{}-continuation", channel);
        let mut iter_continuation = self.db.prefix_iter(&rtxn, &continuation_prefix)?;

        let data_prefix = format!("channel-{}-data", channel);
        let mut iter_data = self.db.prefix_iter(&rtxn, &data_prefix)?;

        if !self.db.is_empty(&rtxn)? {
            println!("\nCurrent channel state for \"{}\":", channel);

            let mut iter_continuation_option = iter_continuation.next().transpose()?;
            while iter_continuation_option.is_some() {
                let key = iter_continuation_option.as_ref().unwrap().0;
                let ccdata_buf = &iter_continuation_option.as_ref().unwrap().1;
                let ccdata = rtypes::CommitContinuation::decode(ccdata_buf.as_slice()).unwrap();
                println!("KEY: {:?} VALUE: {:?}", key, ccdata);
                iter_continuation_option = iter_continuation.next().transpose()?;
            }

            let mut iter_data_option = iter_data.next().transpose()?;
            while iter_data_option.is_some() {
                let key = iter_data_option.as_ref().unwrap().0;
                let rcdata_buf = &iter_data_option.as_ref().unwrap().1;
                let rcdata = rtypes::RetrieveContinuation::decode(rcdata_buf.as_slice()).unwrap();
                println!("KEY: {:?} VALUE: {:?}", key, rcdata);
                iter_data_option = iter_data.next().transpose()?;
            }
        } else {
            println!("\nDatabase is empty")
        }

        drop(iter_continuation);
        drop(iter_data);
        rtxn.commit()?;

        Ok(())
    }

    pub fn is_empty(&self) -> bool {
        let rtxn = self.env.read_txn().unwrap();
        return self.db.is_empty(&rtxn).unwrap();
    }

    pub fn clear(&self) -> Result<(), Box<dyn Error>> {
        let mut wtxn = self.env.write_txn()?;
        let _ = self.db.clear(&mut wtxn)?;
        wtxn.commit()?;

        Ok(())
    }

    fn calculate_hash<T: Hash>(&self, t: &T) -> u64 {
        let mut s = DefaultHasher::new();
        t.hash(&mut s);
        s.finish()
    }
}

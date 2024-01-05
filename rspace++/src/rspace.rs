use crate::diskconc::DiskConcDB;
use crate::diskseq::DiskSeqDB;
use crate::memconc::MemConcDB;
use crate::memseq::MemSeqDB;
use crate::rtypes::rtypes;
use std::error::Error;

// See https://docs.google.com/document/d/1yWdvJwsq4Ft7elzKBM0dehh4RFoQ-vXt-1TAUTLLxMY/edit
#[repr(C)]
pub struct RSpace<D: prost::Message, K: prost::Message> {
    diskseq: DiskSeqDB<D, K>,
    diskconc: DiskConcDB<D, K>,
    memseq: MemSeqDB<D, K>,
    memconc: MemConcDB<D, K>,
}

impl<
        D: Clone + std::hash::Hash + std::fmt::Debug + std::default::Default + prost::Message,
        K: Clone + std::hash::Hash + std::fmt::Debug + std::default::Default + prost::Message,
    > RSpace<D, K>
{
    pub fn create() -> Result<RSpace<D, K>, Box<dyn Error>> {
        let ds = DiskSeqDB::create().unwrap();
        let dc = DiskConcDB::create().unwrap();
        let ms = MemSeqDB::create().unwrap();
        let mc = MemConcDB::create().unwrap();

        Ok(RSpace {
            diskseq: ds,
            diskconc: dc,
            memseq: ms,
            memconc: mc,
        })
    }

    // Verb Set 1
    pub fn get_once_durable_concurrent(&self, rdata: rtypes::Retrieve) -> Option<rtypes::OptionResult> {
        return self.diskconc.produce(rdata, false);
    }

    pub fn get_once_non_durable_concurrent(
        &self,
        rdata: rtypes::Retrieve,
    ) -> Option<rtypes::OptionResult> {
        return self.memconc.produce(rdata, false);
    }

    pub fn get_once_durable_sequential(&self, rdata: rtypes::Retrieve) -> Option<rtypes::OptionResult> {
        return self.diskseq.produce(rdata, false);
    }

    pub fn get_once_non_durable_sequential(
        &self,
        rdata: rtypes::Retrieve,
    ) -> Option<rtypes::OptionResult> {
        return self.memseq.produce(rdata, false);
    }

    // Verb Set 2
    pub fn get_always_durable_concurrent(
        &self,
        rdata: rtypes::Retrieve,
    ) -> Option<rtypes::OptionResult> {
        return self.diskconc.produce(rdata, true);
    }

    pub fn get_always_non_durable_concurrent(
        &self,
        rdata: rtypes::Retrieve,
    ) -> Option<rtypes::OptionResult> {
        return self.memconc.produce(rdata, true);
    }

    pub fn get_always_durable_sequential(
        &self,
        rdata: rtypes::Retrieve,
    ) -> Option<rtypes::OptionResult> {
        return self.diskseq.produce(rdata, true);
    }

    pub fn get_always_non_durable_sequential(
        &self,
        rdata: rtypes::Retrieve,
    ) -> Option<rtypes::OptionResult> {
        return self.memseq.produce(rdata, true);
    }

    // Verb Set 3
    pub fn put_once_durable_concurrent(
        &self,
        cdata: rtypes::Commit,
    ) -> Option<Vec<rtypes::OptionResult>> {
        return self.diskconc.consume(cdata, false);
    }

    pub fn put_once_non_durable_concurrent(
        &self,
        cdata: rtypes::Commit,
    ) -> Option<Vec<rtypes::OptionResult>> {
        return self.memconc.consume(cdata, false);
    }

    pub fn put_once_durable_sequential(
        &self,
        cdata: rtypes::Commit,
    ) -> Option<Vec<rtypes::OptionResult>> {
        return self.diskseq.consume(cdata, false);
    }

    pub fn put_once_non_durable_sequential(
        &self,
        cdata: rtypes::Commit,
    ) -> Option<Vec<rtypes::OptionResult>> {
        return self.memseq.consume(cdata, false);
    }

    // Verb Set 4
    pub fn put_always_durable_concurrent(
        &self,
        cdata: rtypes::Commit,
    ) -> Option<Vec<rtypes::OptionResult>> {
        return self.diskconc.consume(cdata, true);
    }

    pub fn put_always_non_durable_concurrent(
        &self,
        cdata: rtypes::Commit,
    ) -> Option<Vec<rtypes::OptionResult>> {
        return self.memconc.consume(cdata, true);
    }

    pub fn put_always_durable_sequential(
        &self,
        cdata: rtypes::Commit,
    ) -> Option<Vec<rtypes::OptionResult>> {
        return self.diskseq.consume(cdata, true);
    }

    pub fn put_always_non_durable_sequential(
        &self,
        cdata: rtypes::Commit,
    ) -> Option<Vec<rtypes::OptionResult>> {
        return self.memseq.consume(cdata, true);
    }

    pub fn print_data(&self, channel: &str) -> () {
        let _ = self.memseq.print_channel(channel);
        //let _ = self.memconc.print_channel(channel);
        // let _ = self.diskseq.print_channel(channel);
        // let _ = self.diskconc.print_channel(channel);
    }

    // TODO: Remove the need to pass in channel. Should be able to print entire store
    pub fn print_store(&self, channel: &str) -> () {
        println!("\n*** IN-MEMORY SEQUENTIAL ***");
        let _ = self.memseq.print_channel(channel);

        println!("\n*** IN-MEMORY CONCURRENT ***");
        let _ = self.memconc.print_channel(channel);

        println!("\n*** ON-DISK SEQUENTIAL ***");
        let _ = self.diskseq.print_channel(channel);

        println!("\n*** ON-DISK CONCURRENT ***");
        let _ = self.diskconc.print_channel(channel);
    }

    pub fn is_empty(&self) -> bool {
        self.memseq.is_empty()
            && self.memconc.is_empty()
            && self.diskseq.is_empty()
            && self.diskconc.is_empty()
    }

    pub fn is_memseq_empty(&self) -> bool {
        let memseq_is_empty = self.memseq.is_empty();
        return memseq_is_empty;
    }
    pub fn is_memconc_empty(&self) -> bool {
        let memconc_is_empty = self.memconc.is_empty();
        return memconc_is_empty;
    }
    pub fn is_diskseq_empty(&self) -> bool {
        let diskseq_is_empty = self.diskseq.is_empty();
        return diskseq_is_empty;
    }
    pub fn is_diskconc_empty(&self) -> bool {
        let diskconc_is_empty = self.diskconc.is_empty();
        return diskconc_is_empty;
    }

    pub fn clear_store(&self) -> () {
        let _ = self.memseq.clear();
        let _ = self.memconc.clear();
        let _ = self.diskseq.clear();
        let _ = self.diskconc.clear();
    }
}

// See models/src/main/scala/coop/rchain/models/BundleOps.scala

use crate::rhoapi::Bundle;

pub struct BundleOps;

impl BundleOps {
    pub fn show(bundle: &Bundle) -> String {
        let sign = if bundle.read_flag && bundle.write_flag {
            String::from("")
        } else if bundle.read_flag && !bundle.write_flag {
            String::from("-")
        } else if !bundle.read_flag && bundle.write_flag {
            String::from("+")
        } else if !bundle.read_flag && !bundle.write_flag {
            String::from("0")
        } else {
            String::from("")
        };

        format!("{:<8}", format!("bundle{}", sign))
    }
}

// See models/src/main/scala/coop/rchain/models/BundleOps.scala

use crate::rhoapi::Bundle;

pub struct BundleOps;

impl BundleOps {
    pub fn merge(b: &Bundle, other: &Bundle) -> Bundle {
        let mut other_mut = other.clone();
        other_mut.read_flag = b.read_flag && other.read_flag;
        other_mut.write_flag = b.write_flag && other.write_flag;
        other_mut
    }

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

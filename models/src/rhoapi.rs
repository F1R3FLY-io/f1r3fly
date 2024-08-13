#![allow(non_snake_case)]

pub mod rhoapi {
    include!(concat!(env!("OUT_DIR"), "/rhoapi.rs"));
}

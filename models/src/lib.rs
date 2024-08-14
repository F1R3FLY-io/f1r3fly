pub mod rust;

pub mod rhoapi {
    include!(concat!(env!("OUT_DIR"), "/rhoapi.rs"));
}

pub mod rspace_plus_plus_types {
    include!(concat!(env!("OUT_DIR"), "/rspace_plus_plus_types.rs"));
}

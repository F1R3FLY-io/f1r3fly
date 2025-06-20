use std::sync::Once;

mod engine;
mod estimator;
mod finality;
mod helper;
mod merging;
mod safety;
mod sync;
mod util;
mod add_block;

static INIT: Once = Once::new();

pub fn init_logger() {
    INIT.call_once(|| {
        env_logger::builder()
            .is_test(true) // ensures logs show up in test output
            .filter_level(log::LevelFilter::Info)
            .try_init()
            .unwrap();
    });
}

use std::sync::Once;

mod helper;
mod util;

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

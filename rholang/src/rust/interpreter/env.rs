use std::collections::HashMap;

pub struct Env<A> {
    env_map: HashMap<i32, A>,
    level: i32,
    shift: i32,
}

use std::collections::HashMap;

// See rholang/src/main/scala/coop/rchain/rholang/interpreter/Env.scala
pub struct Env<A> {
    env_map: HashMap<i32, A>,
    level: i32,
    shift: i32,
}

impl<A> Env<A> {
    pub fn new() -> Env<A> {
        Env {
            env_map: HashMap::new(),
            level: 0,
            shift: 0,
        }
    }
}

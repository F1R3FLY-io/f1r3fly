use models::rhoapi::Par;

// See rholang/src/main/scala/coop/rchain/rholang/interpreter/PrettyPrinter.scala
pub struct PrettyPrinter {
    pub free_shift: i32,
    pub bound_shift: i32,
    pub news_shift_indices: Vec<i32>,
    pub free_id: String,
    pub base_id: String,
    pub rotation: i32,
    pub max_var_count: i32,
    pub is_building_channel: bool,
}

impl PrettyPrinter {
    pub fn build_string_from_par(m: &Par) -> String {
        todo!()
    }
}

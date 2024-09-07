// See rholang/src/main/scala/coop/rchain/rholang/interpreter/PrettyPrinter.scala

use models::rhoapi::{Expr, GUnforgeable, MatchCase, Par, Var};
use shared::rust::printer::Printer;

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
    pub fn new() -> Self {
        PrettyPrinter::create(0, 0)
    }

    fn create(free_shift: i32, bound_shift: i32) -> Self {
        PrettyPrinter {
            free_shift,
            bound_shift,
            news_shift_indices: Vec::new(),
            free_id: String::from("free"),
            base_id: String::from("a"),
            rotation: 23,
            max_var_count: 128,
            is_building_channel: false,
        }
    }

    pub fn cap(&self, str: &str) -> String {
        match Printer::output_capped() {
            Some(n) => format!("{}...", &str[..n as usize]),

            None => str.to_string(),
        }
    }

    fn indent_string(&self) -> String {
        String::from("  ")
    }

    fn bound_id(&self) -> String {
        self.rotate(self.base_id.clone())
    }

    fn set_base_id(&self) -> String {
        self.increment(self.base_id.clone())
    }

    pub fn build_string_from_expr(&self, e: &Expr) -> String {
        self.cap(&self._build_string_from_expr(e))
    }

    pub fn build_string_from_var(&self, v: &Var) -> String {
        self.cap(&self._build_string_from_var(v))
    }

    pub fn build_string_from_par(&self, m: &Par) -> String {
        self.cap(&self._build_string_from_par(m, 0))
    }

    pub fn build_channel_string(&self, m: &Par) -> String {
        self.cap(&self._build_channel_string(m, 0))
    }

    fn build_string_from_unforgeable(&self, u: &GUnforgeable) -> String {
        todo!()
    }

    fn _build_string_from_expr(&self, e: &Expr) -> String {
        todo!()
    }

    fn build_remainder_string(&self, remainder: &Option<Var>) -> String {
        todo!()
    }

    fn _build_string_from_var(&self, v: &Var) -> String {
        todo!()
    }

    fn _build_channel_string(&self, p: &Par, indent: i32) -> String {
        todo!()
    }

    fn _build_string_from_par(&self, m: &Par, indent: i32) -> String {
        todo!()
    }

    fn increment(&self, id: String) -> String {
        fn inc_char(char_id: char) -> char {
            let new_char = ((char_id as u8 + 1 - b'a') % 26 + b'a') as char;

            new_char
        }

        let new_id = inc_char(id.chars().last().unwrap());

        if new_id == 'a' {
            if id.len() > 1 {
                self.increment(id[..id.len() - 1].to_string()) + new_id.to_string().as_str()
            } else {
                "aa".to_string()
            }
        } else {
            id[..id.len() - 1].to_string() + new_id.to_string().as_str()
        }
    }

    fn rotate(&self, id: String) -> String {
        id.chars()
            .map(|char| {
                let new_char = ((char as u8 + self.rotation as u8 - b'a') % 26 + b'a') as char;

                new_char
            })
            .collect()
    }

    fn build_variables(&self, bind_count: i32) -> String {
        todo!()
    }

    fn build_vec<T: prost::Message>(&self, s: Vec<T>) -> String {
        todo!()
    }

    fn build_patterns(&self, patterns: Vec<Par>) -> String {
        todo!()
    }

    fn build_match_case(&self, match_case: MatchCase, indent: i32) -> String {
        todo!()
    }
}

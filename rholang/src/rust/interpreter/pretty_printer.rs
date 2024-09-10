// See rholang/src/main/scala/coop/rchain/rholang/interpreter/PrettyPrinter.scala

use models::{
    rhoapi::{
        expr::ExprInstance, g_unforgeable::UnfInstance, EAnd, EDiv, EEq, EGt, EGte, EList, ELt,
        ELte, EMatches, EMinus, EMinusMinus, EMod, EMult, ENeg, ENeq, ENot, EOr, EPercentPercent,
        EPlus, EPlusPlus, ESet, ETuple, EVar, Expr, GUnforgeable, MatchCase, Par, Var,
    },
    rust::{par_map_type_mapper::ParMapTypeMapper, par_set_type_mapper::ParSetTypeMapper},
};
use shared::rust::{printer::Printer, string_ops::wrap_with_braces};

use super::errors::InterpreterError;

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
        self.cap(
            &self
                ._build_string_from_expr(e)
                .map_err(|err| panic!("{}", err))
                .unwrap(),
        )
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

    fn build_string_from_unforgeable(&self, u: &GUnforgeable) -> Result<String, InterpreterError> {
        match &u.unf_instance {
            Some(instance) => match instance {
                UnfInstance::GPrivateBody(p) => {
                    Ok(format!("Unforgeable(0x{})", hex::encode(p.id.clone())))
                }
                UnfInstance::GDeployIdBody(id) => {
                    Ok(format!("DeployId(0x{})", hex::encode(id.sig.clone())))
                }
                UnfInstance::GDeployerIdBody(id) => Ok(format!(
                    "DeployerId(0x{})",
                    hex::encode(id.public_key.clone())
                )),
                UnfInstance::GSysAuthTokenBody(value) => {
                    Ok(format!("GSysAuthTokenBody({:?})", value))
                }
            },
            // TODO: Figure out if we can prevent prost from generating
            None => Ok(String::from("Nil")),
        }
    }

    fn _build_string_from_expr(&self, e: &Expr) -> Result<String, InterpreterError> {
        match &e.expr_instance {
            Some(instance) => match instance {
                ExprInstance::ENegBody(ENeg { p }) => Ok(format!(
                    "-{}",
                    wrap_with_braces(self.build_string_from_par(
                        &p.as_ref().expect("ENeg par field was None, should be Some")
                    ))
                )),

                ExprInstance::ENotBody(ENot { p }) => Ok(format!(
                    "~{}",
                    wrap_with_braces(self.build_string_from_par(
                        &p.as_ref().expect("ENot par field was None, should be Some")
                    ))
                )),

                ExprInstance::EMultBody(EMult { p1, p2 }) => Ok(format!(
                    "{} * {}",
                    self.build_string_from_par(
                        &p1.as_ref()
                            .expect("EMult p1 field was None, should be Some")
                    ),
                    wrap_with_braces(
                        self.build_string_from_par(
                            &p2.as_ref()
                                .expect("EMult p2 field was None, should be Some")
                        )
                    )
                )),

                ExprInstance::EDivBody(EDiv { p1, p2 }) => Ok(format!(
                    "{} / {}",
                    self.build_string_from_par(
                        &p1.as_ref().expect("EDiv p1 field was None, should be Some")
                    ),
                    wrap_with_braces(self.build_string_from_par(
                        &p2.as_ref().expect("EDiv p2 field was None, should be Some")
                    ))
                )),

                ExprInstance::EModBody(EMod { p1, p2 }) => Ok(format!(
                    "{} % {}",
                    self.build_string_from_par(
                        &p1.as_ref().expect("EMod p1 field was None, should be Some")
                    ),
                    wrap_with_braces(self.build_string_from_par(
                        &p2.as_ref().expect("EMod p2 field was None, should be Some")
                    ))
                )),

                ExprInstance::EPercentPercentBody(EPercentPercent { p1, p2 }) => Ok(format!(
                    "{} %% {}",
                    self.build_string_from_par(
                        &p1.as_ref()
                            .expect("EPercentPercent p1 field was None, should be Some")
                    ),
                    wrap_with_braces(
                        self.build_string_from_par(
                            &p2.as_ref()
                                .expect("EPercentPercent p2 field was None, should be Some")
                        )
                    )
                )),

                ExprInstance::EPlusBody(EPlus { p1, p2 }) => Ok(format!(
                    "{} + {}",
                    self.build_string_from_par(
                        &p1.as_ref()
                            .expect("EPlus p1 field was None, should be Some")
                    ),
                    wrap_with_braces(
                        self.build_string_from_par(
                            &p2.as_ref()
                                .expect("EPlus p2 field was None, should be Some")
                        )
                    )
                )),

                ExprInstance::EPlusPlusBody(EPlusPlus { p1, p2 }) => Ok(format!(
                    "{} ++ {}",
                    self.build_string_from_par(
                        &p1.as_ref()
                            .expect("EPlusPlus p1 field was None, should be Some")
                    ),
                    wrap_with_braces(
                        self.build_string_from_par(
                            &p2.as_ref()
                                .expect("EPlusPlus p2 field was None, should be Some")
                        )
                    )
                )),

                ExprInstance::EMinusBody(EMinus { p1, p2 }) => Ok(format!(
                    "{} - {}",
                    self.build_string_from_par(
                        &p1.as_ref()
                            .expect("EMinus p1 field was None, should be Some")
                    ),
                    wrap_with_braces(
                        self.build_string_from_par(
                            &p2.as_ref()
                                .expect("EMinus p2 field was None, should be Some")
                        )
                    )
                )),

                ExprInstance::EMinusMinusBody(EMinusMinus { p1, p2 }) => Ok(format!(
                    "{} - {}",
                    self.build_string_from_par(
                        &p1.as_ref()
                            .expect("EMinusMinus p1 field was None, should be Some")
                    ),
                    wrap_with_braces(
                        self.build_string_from_par(
                            &p2.as_ref()
                                .expect("EMinusMinus p2 field was None, should be Some")
                        )
                    )
                )),

                ExprInstance::EAndBody(EAnd { p1, p2 }) => Ok(format!(
                    "{} && {}",
                    self.build_string_from_par(
                        &p1.as_ref().expect("EAnd p1 field was None, should be Some")
                    ),
                    wrap_with_braces(self.build_string_from_par(
                        &p2.as_ref().expect("EAnd p2 field was None, should be Some")
                    ))
                )),

                ExprInstance::EOrBody(EOr { p1, p2 }) => Ok(format!(
                    "{} || {}",
                    self.build_string_from_par(
                        &p1.as_ref().expect("EOr p1 field was None, should be Some")
                    ),
                    wrap_with_braces(self.build_string_from_par(
                        &p2.as_ref().expect("EOr p2 field was None, should be Some")
                    ))
                )),

                ExprInstance::EEqBody(EEq { p1, p2 }) => Ok(format!(
                    "{} == {}",
                    self.build_string_from_par(
                        &p1.as_ref().expect("EEq p1 field was None, should be Some")
                    ),
                    wrap_with_braces(self.build_string_from_par(
                        &p2.as_ref().expect("EEq p2 field was None, should be Some")
                    ))
                )),

                ExprInstance::ENeqBody(ENeq { p1, p2 }) => Ok(format!(
                    "{} != {}",
                    self.build_string_from_par(
                        &p1.as_ref().expect("ENeq p1 field was None, should be Some")
                    ),
                    wrap_with_braces(self.build_string_from_par(
                        &p2.as_ref().expect("ENeq p2 field was None, should be Some")
                    ))
                )),

                ExprInstance::EGtBody(EGt { p1, p2 }) => Ok(format!(
                    "{} > {}",
                    self.build_string_from_par(
                        &p1.as_ref().expect("EGt p1 field was None, should be Some")
                    ),
                    wrap_with_braces(self.build_string_from_par(
                        &p2.as_ref().expect("EGt p2 field was None, should be Some")
                    ))
                )),

                ExprInstance::EGteBody(EGte { p1, p2 }) => Ok(format!(
                    "{} >= {}",
                    self.build_string_from_par(
                        &p1.as_ref().expect("EGte p1 field was None, should be Some")
                    ),
                    wrap_with_braces(self.build_string_from_par(
                        &p2.as_ref().expect("EGte p2 field was None, should be Some")
                    ))
                )),

                ExprInstance::ELtBody(ELt { p1, p2 }) => Ok(format!(
                    "{} < {}",
                    self.build_string_from_par(
                        &p1.as_ref().expect("ELt p1 field was None, should be Some")
                    ),
                    wrap_with_braces(self.build_string_from_par(
                        &p2.as_ref().expect("ELt p2 field was None, should be Some")
                    ))
                )),

                ExprInstance::ELteBody(ELte { p1, p2 }) => Ok(format!(
                    "{} <= {}",
                    self.build_string_from_par(
                        &p1.as_ref().expect("ELte p1 field was None, should be Some")
                    ),
                    wrap_with_braces(self.build_string_from_par(
                        &p2.as_ref().expect("ELte p2 field was None, should be Some")
                    ))
                )),

                ExprInstance::EMatchesBody(EMatches { target, pattern }) => {
                    Ok(wrap_with_braces(format!(
                        "{} matches {}",
                        self.build_string_from_par(
                            &target
                                .as_ref()
                                .expect("EMatches target field was None, should be Some")
                        ),
                        self.build_string_from_par(
                            &pattern
                                .as_ref()
                                .expect("EMatches pattern field was None, should be Some")
                        )
                    )))
                }

                ExprInstance::EListBody(EList { ps, remainder, .. }) => Ok(format!(
                    "[{},{}]",
                    self.build_vec(ps),
                    self.build_remainder_string(remainder)
                )),

                ExprInstance::ETupleBody(ETuple { ps, .. }) => {
                    Ok(format!("({})", self.build_vec(ps),))
                }

                ExprInstance::ESetBody(eset) => {
                    let par_set = ParSetTypeMapper::eset_to_par_set(eset.clone());
                    let pars = par_set.ps;
                    let remainder = &par_set.remainder;
                    Ok(format!(
                        "Set({},{})",
                        self.build_vec(&pars.sorted_pars),
                        self.build_remainder_string(remainder)
                    ))
                }

                ExprInstance::EMapBody(emap) => {
                    let par_map = ParMapTypeMapper::emap_to_par_map(emap.clone());
                    let sorted_list = par_map.ps.sorted_list;
                    let remainder = &par_map.remainder;
                    let mut result = String::from("{");

                    for (i, (key, value)) in sorted_list.iter().enumerate() {
                        result.push_str(&self.build_string_from_par(key));
                        result.push_str(" : ");
                        result.push_str(&self.build_string_from_par(value));

                        if i != sorted_list.len() - 1 {
                            result.push_str(", ");
                        }
                    }

                    result.push_str(&self.build_remainder_string(remainder));
                    result.push_str("}");

                    Ok(result)
                }

                ExprInstance::EVarBody(EVar { v }) => Ok(self.build_string_from_var(
                    v.as_ref()
                        .expect("var field on EVar was None, should be Some"),
                )),

                ExprInstance::GBool(_) => todo!(),
                ExprInstance::GInt(_) => todo!(),
                ExprInstance::GString(_) => todo!(),
                ExprInstance::GUri(_) => todo!(),
                ExprInstance::GByteArray(_) => todo!(),
                ExprInstance::EMethodBody(_) => todo!(),
            },
            // TODO: Figure out if we can prevent prost from generating
            None => Ok(String::from("Nil")),
        }
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

    fn build_vec<T: prost::Message>(&self, s: &Vec<T>) -> String {
        todo!()
    }

    fn build_patterns(&self, patterns: Vec<Par>) -> String {
        todo!()
    }

    fn build_match_case(&self, match_case: MatchCase, indent: i32) -> String {
        todo!()
    }
}

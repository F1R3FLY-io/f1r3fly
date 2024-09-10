// See rholang/src/main/scala/coop/rchain/rholang/interpreter/PrettyPrinter.scala

use models::{
    rhoapi::{
        connective::ConnectiveInstance, expr::ExprInstance, g_unforgeable::UnfInstance,
        var::VarInstance, Bundle, Connective, EAnd, EDiv, EEq, EGt, EGte, EList, ELt, ELte,
        EMatches, EMinus, EMinusMinus, EMod, EMult, ENeg, ENeq, ENot, EOr, EPercentPercent, EPlus,
        EPlusPlus, ETuple, EVar, Expr, GUnforgeable, Match, MatchCase, New, Par, Receive, Var,
    },
    rust::{
        bundle_ops::BundleOps, par_map_type_mapper::ParMapTypeMapper,
        par_set_type_mapper::ParSetTypeMapper,
    },
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

    pub fn build_string_from_expr(&mut self, e: &Expr) -> String {
        let str = &self
            ._build_string_from_expr(e)
            .map_err(|err| panic!("{}", err))
            .unwrap();
        self.cap(str)
    }

    pub fn build_string_from_var(&self, v: &Var) -> String {
        self.cap(&self._build_string_from_var(v))
    }

    pub fn build_string_from_message(&mut self, m: &dyn std::any::Any) -> String {
        let str = &self
            ._build_string_from_message(m, 0)
            .map_err(|err| panic!("{}", err))
            .unwrap();
        self.cap(&str)
    }

    pub fn build_channel_string(&mut self, m: &Par) -> String {
        let str = &self
            ._build_channel_string(m, 0)
            .map_err(|err| panic!("{}", err))
            .unwrap();
        self.cap(str)
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
            // TODO: Figure out if we can prevent prost from generating - OLD
            None => Ok(String::from("Nil")),
        }
    }

    fn _build_string_from_expr(&mut self, e: &Expr) -> Result<String, InterpreterError> {
        match &e.expr_instance {
            Some(instance) => match instance {
                ExprInstance::ENegBody(ENeg { p }) => Ok(format!(
                    "-{}",
                    wrap_with_braces(self.build_string_from_message(
                        p.as_ref().expect("ENeg par field was None, should be Some")
                    ))
                )),

                ExprInstance::ENotBody(ENot { p }) => Ok(format!(
                    "~{}",
                    wrap_with_braces(self.build_string_from_message(
                        p.as_ref().expect("ENot par field was None, should be Some")
                    ))
                )),

                ExprInstance::EMultBody(EMult { p1, p2 }) => Ok(format!(
                    "{} * {}",
                    self.build_string_from_message(
                        p1.as_ref()
                            .expect("EMult p1 field was None, should be Some")
                    ),
                    wrap_with_braces(
                        self.build_string_from_message(
                            p2.as_ref()
                                .expect("EMult p2 field was None, should be Some")
                        )
                    )
                )),

                ExprInstance::EDivBody(EDiv { p1, p2 }) => Ok(format!(
                    "{} / {}",
                    self.build_string_from_message(
                        p1.as_ref().expect("EDiv p1 field was None, should be Some")
                    ),
                    wrap_with_braces(self.build_string_from_message(
                        p2.as_ref().expect("EDiv p2 field was None, should be Some")
                    ))
                )),

                ExprInstance::EModBody(EMod { p1, p2 }) => Ok(format!(
                    "{} % {}",
                    self.build_string_from_message(
                        p1.as_ref().expect("EMod p1 field was None, should be Some")
                    ),
                    wrap_with_braces(self.build_string_from_message(
                        p2.as_ref().expect("EMod p2 field was None, should be Some")
                    ))
                )),

                ExprInstance::EPercentPercentBody(EPercentPercent { p1, p2 }) => Ok(format!(
                    "{} %% {}",
                    self.build_string_from_message(
                        p1.as_ref()
                            .expect("EPercentPercent p1 field was None, should be Some")
                    ),
                    wrap_with_braces(
                        self.build_string_from_message(
                            p2.as_ref()
                                .expect("EPercentPercent p2 field was None, should be Some")
                        )
                    )
                )),

                ExprInstance::EPlusBody(EPlus { p1, p2 }) => Ok(format!(
                    "{} + {}",
                    self.build_string_from_message(
                        p1.as_ref()
                            .expect("EPlus p1 field was None, should be Some")
                    ),
                    wrap_with_braces(
                        self.build_string_from_message(
                            p2.as_ref()
                                .expect("EPlus p2 field was None, should be Some")
                        )
                    )
                )),

                ExprInstance::EPlusPlusBody(EPlusPlus { p1, p2 }) => Ok(format!(
                    "{} ++ {}",
                    self.build_string_from_message(
                        p1.as_ref()
                            .expect("EPlusPlus p1 field was None, should be Some")
                    ),
                    wrap_with_braces(
                        self.build_string_from_message(
                            p2.as_ref()
                                .expect("EPlusPlus p2 field was None, should be Some")
                        )
                    )
                )),

                ExprInstance::EMinusBody(EMinus { p1, p2 }) => Ok(format!(
                    "{} - {}",
                    self.build_string_from_message(
                        p1.as_ref()
                            .expect("EMinus p1 field was None, should be Some")
                    ),
                    wrap_with_braces(
                        self.build_string_from_message(
                            p2.as_ref()
                                .expect("EMinus p2 field was None, should be Some")
                        )
                    )
                )),

                ExprInstance::EMinusMinusBody(EMinusMinus { p1, p2 }) => Ok(format!(
                    "{} - {}",
                    self.build_string_from_message(
                        p1.as_ref()
                            .expect("EMinusMinus p1 field was None, should be Some")
                    ),
                    wrap_with_braces(
                        self.build_string_from_message(
                            p2.as_ref()
                                .expect("EMinusMinus p2 field was None, should be Some")
                        )
                    )
                )),

                ExprInstance::EAndBody(EAnd { p1, p2 }) => Ok(format!(
                    "{} && {}",
                    self.build_string_from_message(
                        p1.as_ref().expect("EAnd p1 field was None, should be Some")
                    ),
                    wrap_with_braces(self.build_string_from_message(
                        p2.as_ref().expect("EAnd p2 field was None, should be Some")
                    ))
                )),

                ExprInstance::EOrBody(EOr { p1, p2 }) => Ok(format!(
                    "{} || {}",
                    self.build_string_from_message(
                        p1.as_ref().expect("EOr p1 field was None, should be Some")
                    ),
                    wrap_with_braces(self.build_string_from_message(
                        p2.as_ref().expect("EOr p2 field was None, should be Some")
                    ))
                )),

                ExprInstance::EEqBody(EEq { p1, p2 }) => Ok(format!(
                    "{} == {}",
                    self.build_string_from_message(
                        p1.as_ref().expect("EEq p1 field was None, should be Some")
                    ),
                    wrap_with_braces(self.build_string_from_message(
                        p2.as_ref().expect("EEq p2 field was None, should be Some")
                    ))
                )),

                ExprInstance::ENeqBody(ENeq { p1, p2 }) => Ok(format!(
                    "{} != {}",
                    self.build_string_from_message(
                        p1.as_ref().expect("ENeq p1 field was None, should be Some")
                    ),
                    wrap_with_braces(self.build_string_from_message(
                        p2.as_ref().expect("ENeq p2 field was None, should be Some")
                    ))
                )),

                ExprInstance::EGtBody(EGt { p1, p2 }) => Ok(format!(
                    "{} > {}",
                    self.build_string_from_message(
                        p1.as_ref().expect("EGt p1 field was None, should be Some")
                    ),
                    wrap_with_braces(self.build_string_from_message(
                        p2.as_ref().expect("EGt p2 field was None, should be Some")
                    ))
                )),

                ExprInstance::EGteBody(EGte { p1, p2 }) => Ok(format!(
                    "{} >= {}",
                    self.build_string_from_message(
                        p1.as_ref().expect("EGte p1 field was None, should be Some")
                    ),
                    wrap_with_braces(self.build_string_from_message(
                        p2.as_ref().expect("EGte p2 field was None, should be Some")
                    ))
                )),

                ExprInstance::ELtBody(ELt { p1, p2 }) => Ok(format!(
                    "{} < {}",
                    self.build_string_from_message(
                        p1.as_ref().expect("ELt p1 field was None, should be Some")
                    ),
                    wrap_with_braces(self.build_string_from_message(
                        p2.as_ref().expect("ELt p2 field was None, should be Some")
                    ))
                )),

                ExprInstance::ELteBody(ELte { p1, p2 }) => Ok(format!(
                    "{} <= {}",
                    self.build_string_from_message(
                        p1.as_ref().expect("ELte p1 field was None, should be Some")
                    ),
                    wrap_with_braces(self.build_string_from_message(
                        p2.as_ref().expect("ELte p2 field was None, should be Some")
                    ))
                )),

                ExprInstance::EMatchesBody(EMatches { target, pattern }) => {
                    Ok(wrap_with_braces(format!(
                        "{} matches {}",
                        self.build_string_from_message(
                            target
                                .as_ref()
                                .expect("EMatches target field was None, should be Some")
                        ),
                        self.build_string_from_message(
                            pattern
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
                        result.push_str(&self.build_string_from_message(key));
                        result.push_str(" : ");
                        result.push_str(&self.build_string_from_message(value));

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

                ExprInstance::GBool(b) => Ok(b.to_string()),
                ExprInstance::GInt(i) => Ok(i.to_string()),
                ExprInstance::GString(s) => Ok(format!("\"{}\"", s)),
                ExprInstance::GUri(u) => Ok(format!("`{}`", u)),
                ExprInstance::EMethodBody(method) => {
                    let args: Vec<String> = method
                        .arguments
                        .iter()
                        .map(|arg| self.build_string_from_message(arg))
                        .collect();

                    let args_string = args.join(", ");

                    Ok(format!(
                        "({}).{}({})",
                        self.build_string_from_message(
                            method
                                .target
                                .as_ref()
                                .expect("target field on Method was None, should be Some")
                        ),
                        method.method_name,
                        args_string
                    ))
                }
                ExprInstance::GByteArray(bs) => Ok(hex::encode(bs)),
            },
            // TODO: Figure out if we can prevent prost from generating - OLD
            None => Ok(String::from("Nil")),
        }
    }

    fn build_remainder_string(&self, remainder: &Option<Var>) -> String {
        match remainder {
            Some(v) => {
                format!("...{:?}", v)
            }
            None => format!(""),
        }
    }

    fn _build_string_from_var(&self, v: &Var) -> String {
        match &v.var_instance {
            Some(instance) => match instance {
                VarInstance::FreeVar(level) => {
                    format!("{}{}", self.free_id, self.free_shift + level)
                }
                VarInstance::BoundVar(level) => {
                    let prefix = if PrettyPrinter::is_new_var(
                        level,
                        self.news_shift_indices.clone(),
                        self.bound_shift,
                    ) && !self.is_building_channel
                    {
                        "*".to_string()
                    } else {
                        "".to_string()
                    };

                    format!(
                        "{}{}",
                        prefix,
                        self.bound_id() + &(self.bound_shift - level - 1).to_string()
                    )
                }
                VarInstance::Wildcard(_) => String::from("_"),
            },
            None => String::from("@Nil"),
        }
    }

    fn _build_channel_string(
        &mut self,
        p: &Par,
        indent: usize,
    ) -> Result<String, InterpreterError> {
        let quote_if_not_new = |s: String, news_shift_indices: Vec<i32>, bound_shift: i32| {
            let is_bound_new = match p.exprs.as_slice() {
                [x] => match &x.expr_instance {
                    Some(instance) => match instance {
                        ExprInstance::EVarBody(EVar { v }) => match v {
                            Some(v) => match &v.var_instance {
                                Some(instance) => match instance {
                                    VarInstance::BoundVar(level) => PrettyPrinter::is_new_var(
                                        &level,
                                        news_shift_indices,
                                        bound_shift,
                                    ),
                                    _ => false,
                                },
                                None => false,
                            },
                            None => false,
                        },

                        _ => false,
                    },
                    None => false,
                },
                _ => false,
            };

            if is_bound_new {
                s
            } else {
                format!("@{{{}}}", s)
            }
        };

        self.is_building_channel = true;
        let str = self._build_string_from_message(p, indent)?;
        if str.len() > 60 {
            Ok(quote_if_not_new(
                str,
                self.news_shift_indices.clone(),
                self.bound_shift,
            ))
        } else {
            let whitespace = "\n(\\s\\s)*";
            let replaced = regex::Regex::new(whitespace)
                .unwrap()
                .replace_all(&str, " ");
            Ok(quote_if_not_new(
                replaced.to_string(),
                self.news_shift_indices.clone(),
                self.bound_shift,
            ))
        }
    }

    fn _build_string_from_message(
        &mut self,
        m: &dyn std::any::Any,
        indent: usize,
    ) -> Result<String, InterpreterError> {
        if let Some(v) = m.downcast_ref::<Var>() {
            Ok(self.build_string_from_var(v))
        } else if let Some(s) = m.downcast_ref::<models::rhoapi::Send>() {
            let str = if s.persistent {
                String::from("!!(")
            } else {
                String::from("!(")
            };

            Ok(format!(
                "{}{})",
                self.build_string_from_message(
                    s.chan
                        .as_ref()
                        .expect("channel field on Send was None, should be Some")
                ),
                str
            ))
        } else if let Some(r) = m.downcast_ref::<Receive>() {
            let (totally_free, binds_string) = r.binds.iter().enumerate().try_fold(
                (0, String::from("")),
                |(previous_free, mut string), (i, bind)| {
                    self.free_shift = self.bound_shift + previous_free;
                    self.bound_shift = 0;
                    self.free_id = self.bound_id();
                    self.base_id = self.set_base_id();

                    let bind_string = self.build_pattern(&bind.patterns);
                    string.push_str(&bind_string);

                    if r.persistent {
                        string.push_str(" <= ");
                    } else if r.peek {
                        string.push_str(" <<- ");
                    } else {
                        string.push_str(" <- ");
                    }

                    string.push_str(
                        &self._build_channel_string(
                            &bind
                                .source
                                .as_ref()
                                .expect("source field on bind was None, should be Some"),
                            indent,
                        )?,
                    );

                    if i != r.binds.len() - 1 {
                        string.push_str("  & ");
                    }

                    Ok((bind.free_count + previous_free, string))
                },
            )?;

            self.bound_shift = self.bound_shift + totally_free;
            let body_str = self.build_string_from_message(
                r.body
                    .as_ref()
                    .expect("body field on receive was None, should be Soem"),
            );

            if !body_str.is_empty() {
                Ok(format!(
                    "for( {} ) {{\n{}{}{}\n{}}}",
                    binds_string,
                    self.indent_string().repeat(indent + 1),
                    body_str,
                    self.indent_string().repeat(indent),
                    ""
                ))
            } else {
                Ok(format!("for( {} ) {{}}", binds_string))
            }
        } else if let Some(b) = m.downcast_ref::<Bundle>() {
            Ok(format!(
                "{}{{\n{}{}\n}}",
                BundleOps::show(b),
                self.indent_string().repeat(indent + 1),
                self._build_string_from_message(
                    b.body
                        .as_ref()
                        .expect("body field on bundle was None, should be Some"),
                    indent + 1
                )?
            ))
        } else if let Some(n) = m.downcast_ref::<New>() {
            let introduced_news_shift_idx: Vec<i32> =
                (0..n.bind_count).map(|i| i + self.bound_shift).collect();

            let result = format!(
                "new {} in {{\n{}{}",
                self.build_variables(n.bind_count),
                self.indent_string().repeat(indent + 1),
                {
                    self.bound_shift = self.bound_shift + n.bind_count;
                    self.news_shift_indices = self
                        .news_shift_indices
                        .clone()
                        .into_iter()
                        .chain(introduced_news_shift_idx)
                        .collect();
                    self._build_string_from_message(
                        n.p.as_ref()
                            .expect("p field on New was None, should be Some"),
                        indent + 1,
                    )?
                }
            );

            Ok(format!(
                "{}\n{}{}",
                result,
                self.indent_string().repeat(indent),
                "}"
            ))
        } else if let Some(e) = m.downcast_ref::<Expr>() {
            Ok(self.build_string_from_expr(e))
        } else if let Some(m) = m.downcast_ref::<Match>() {
            let result = format!(
                "match {} {{\n{}{}",
                self.build_string_from_message(&m.target),
                self.indent_string().repeat(indent + 1),
                m.cases
                    .iter()
                    .enumerate()
                    .fold(Ok(String::new()), |acc, (i, match_case)| {
                        let string = acc?;

                        let case_string = format!(
                            "{}{}{}",
                            self.indent_string().repeat(indent + 1),
                            self.build_match_case(match_case, indent + 1)?,
                            if i != m.cases.len() - 1 { "\n" } else { "" }
                        );

                        Ok(string + &case_string)
                    })?
            );

            Ok(format!(
                "{}\n{}{}",
                result,
                self.indent_string().repeat(indent),
                "}"
            ))
        } else if let Some(u) = m.downcast_ref::<GUnforgeable>() {
            self.build_string_from_unforgeable(u)
        } else if let Some(c) = m.downcast_ref::<Connective>() {
            match &c.connective_instance {
                Some(conn_instance) => match conn_instance {
                    ConnectiveInstance::ConnAndBody(value) => Ok(format!(
                        "{{ {} }}",
                        value
                            .ps
                            .iter()
                            .map(|p| self.build_string_from_message(p))
                            .collect::<Vec<String>>()
                            .join(" /\\ ")
                    )),
                    ConnectiveInstance::ConnOrBody(value) => Ok(format!(
                        "{{ {} }}",
                        value
                            .ps
                            .iter()
                            .map(|p| self.build_string_from_message(p))
                            .collect::<Vec<String>>()
                            .join(" \\/ ")
                    )),
                    ConnectiveInstance::ConnNotBody(value) => {
                        Ok(format!("~{{{}}}", self.build_string_from_message(value)))
                    }
                    ConnectiveInstance::VarRefBody(value) => Ok(format!(
                        "={}{}",
                        self.free_id,
                        self.free_shift - value.index - 1
                    )),
                    ConnectiveInstance::ConnBool(_) => Ok(String::from("Bool")),
                    ConnectiveInstance::ConnInt(_) => Ok(String::from("Int")),
                    ConnectiveInstance::ConnString(_) => Ok(String::from("String")),
                    ConnectiveInstance::ConnUri(_) => Ok(String::from("Uri")),
                    ConnectiveInstance::ConnByteArray(_) => Ok(String::from("ByteArray")),
                },
                None => Ok(String::new()),
            }
        } else if let Some(p) = m.downcast_ref::<Par>() {
            if self.is_empty_par(p) {
                Ok(String::from("Nil"))
            } else {
                let p_cloned = p.clone();
                let vec: Vec<Box<dyn std::any::Any>> = vec![
                    Box::new(p_cloned.bundles),
                    Box::new(p_cloned.sends),
                    Box::new(p_cloned.receives),
                    Box::new(p_cloned.news),
                    Box::new(p_cloned.exprs),
                    Box::new(p_cloned.matches),
                    Box::new(p_cloned.unforgeables),
                    Box::new(p_cloned.connectives),
                ];

                let mut prev_non_empty = false;
                let mut result = String::new();

                for items in vec {
                    if let Some(items_vec) = items.downcast_ref::<Vec<Bundle>>() {
                        if !items_vec.is_empty() {
                            if prev_non_empty {
                                result.push_str(&format!(
                                    " |\n{}",
                                    self.indent_string().repeat(indent)
                                ));
                            }

                            for (index, _par) in items_vec.iter().enumerate() {
                                result.push_str(&self._build_string_from_message(_par, indent)?);

                                if index != items_vec.len() - 1 {
                                    result.push_str(&format!(
                                        " |\n{}",
                                        self.indent_string().repeat(indent)
                                    ));
                                }
                            }

                            prev_non_empty = true;
                        }
                    } else if let Some(items_vec) =
                        items.downcast_ref::<Vec<models::rhoapi::Send>>()
                    {
                        if !items_vec.is_empty() {
                            if prev_non_empty {
                                result.push_str(&format!(
                                    " |\n{}",
                                    self.indent_string().repeat(indent)
                                ));
                            }

                            for (index, _par) in items_vec.iter().enumerate() {
                                result.push_str(&self._build_string_from_message(_par, indent)?);

                                if index != items_vec.len() - 1 {
                                    result.push_str(&format!(
                                        " |\n{}",
                                        self.indent_string().repeat(indent)
                                    ));
                                }
                            }

                            prev_non_empty = true;
                        }
                    } else if let Some(items_vec) = items.downcast_ref::<Vec<Receive>>() {
                        if !items_vec.is_empty() {
                            if prev_non_empty {
                                result.push_str(&format!(
                                    " |\n{}",
                                    self.indent_string().repeat(indent)
                                ));
                            }

                            for (index, _par) in items_vec.iter().enumerate() {
                                result.push_str(&self._build_string_from_message(_par, indent)?);

                                if index != items_vec.len() - 1 {
                                    result.push_str(&format!(
                                        " |\n{}",
                                        self.indent_string().repeat(indent)
                                    ));
                                }
                            }

                            prev_non_empty = true;
                        }
                    } else if let Some(items_vec) = items.downcast_ref::<Vec<New>>() {
                        if !items_vec.is_empty() {
                            if prev_non_empty {
                                result.push_str(&format!(
                                    " |\n{}",
                                    self.indent_string().repeat(indent)
                                ));
                            }

                            for (index, _par) in items_vec.iter().enumerate() {
                                result.push_str(&self._build_string_from_message(_par, indent)?);

                                if index != items_vec.len() - 1 {
                                    result.push_str(&format!(
                                        " |\n{}",
                                        self.indent_string().repeat(indent)
                                    ));
                                }
                            }

                            prev_non_empty = true;
                        }
                    } else if let Some(items_vec) = items.downcast_ref::<Vec<Expr>>() {
                        if !items_vec.is_empty() {
                            if prev_non_empty {
                                result.push_str(&format!(
                                    " |\n{}",
                                    self.indent_string().repeat(indent)
                                ));
                            }

                            for (index, _par) in items_vec.iter().enumerate() {
                                result.push_str(&self._build_string_from_message(_par, indent)?);

                                if index != items_vec.len() - 1 {
                                    result.push_str(&format!(
                                        " |\n{}",
                                        self.indent_string().repeat(indent)
                                    ));
                                }
                            }

                            prev_non_empty = true;
                        }
                    } else if let Some(items_vec) = items.downcast_ref::<Vec<Match>>() {
                        if !items_vec.is_empty() {
                            if prev_non_empty {
                                result.push_str(&format!(
                                    " |\n{}",
                                    self.indent_string().repeat(indent)
                                ));
                            }

                            for (index, _par) in items_vec.iter().enumerate() {
                                result.push_str(&self._build_string_from_message(_par, indent)?);

                                if index != items_vec.len() - 1 {
                                    result.push_str(&format!(
                                        " |\n{}",
                                        self.indent_string().repeat(indent)
                                    ));
                                }
                            }

                            prev_non_empty = true;
                        }
                    } else if let Some(items_vec) = items.downcast_ref::<Vec<GUnforgeable>>() {
                        if !items_vec.is_empty() {
                            if prev_non_empty {
                                result.push_str(&format!(
                                    " |\n{}",
                                    self.indent_string().repeat(indent)
                                ));
                            }

                            for (index, _par) in items_vec.iter().enumerate() {
                                result.push_str(&self._build_string_from_message(_par, indent)?);

                                if index != items_vec.len() - 1 {
                                    result.push_str(&format!(
                                        " |\n{}",
                                        self.indent_string().repeat(indent)
                                    ));
                                }
                            }

                            prev_non_empty = true;
                        }
                    } else if let Some(items_vec) = items.downcast_ref::<Vec<Connective>>() {
                        if !items_vec.is_empty() {
                            if prev_non_empty {
                                result.push_str(&format!(
                                    " |\n{}",
                                    self.indent_string().repeat(indent)
                                ));
                            }

                            for (index, _par) in items_vec.iter().enumerate() {
                                result.push_str(&self._build_string_from_message(_par, indent)?);

                                if index != items_vec.len() - 1 {
                                    result.push_str(&format!(
                                        " |\n{}",
                                        self.indent_string().repeat(indent)
                                    ));
                                }
                            }

                            prev_non_empty = true;
                        }
                    } else {
                        return Err(InterpreterError::BugFoundError(format!(
                            "Attempt to print unknown prost::Message type(s): {:?}",
                            items
                        )));
                    }
                }

                Ok(result)
            }
        } else {
            Err(InterpreterError::BugFoundError(format!(
                "Attempt to print unknown prost::Message type: {:?}",
                m
            )))
        }
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
        (0..std::cmp::min(self.max_var_count, bind_count))
            .map(|i| format!("{}{}", self.bound_id(), self.bound_shift + i))
            .collect::<Vec<String>>()
            .join(", ")
    }

    fn build_vec(&mut self, s: &Vec<Par>) -> String {
        s.iter().enumerate().fold(String::new(), |string, (i, p)| {
            let mut result = string;

            result.push_str(&self.build_string_from_message(p));

            if i != s.len() - 1 {
                result.push_str(", ");
            }

            result
        })
    }

    fn build_pattern(&mut self, patterns: &Vec<Par>) -> String {
        patterns
            .iter()
            .enumerate()
            .fold(String::new(), |string, (i, pattern)| {
                let mut result = string;

                result.push_str(&self.build_channel_string(pattern));

                if i != patterns.len() - 1 {
                    result.push_str(", ");
                }

                result
            })
    }

    fn build_match_case(
        &mut self,
        match_case: &MatchCase,
        indent: usize,
    ) -> Result<String, InterpreterError> {
        let pattern_free = match_case.free_count;
        let open_brace = format!("{{\n{}", self.indent_string().repeat(indent + 1));
        let close_brace = format!("\n{}}}", self.indent_string().repeat(indent));

        self.free_shift = self.bound_shift;
        self.bound_shift = 0;
        self.free_id = self.bound_id();
        self.base_id = self.set_base_id();

        Ok(format!(
            "{} => {}{}{}",
            self._build_string_from_message(
                match_case
                    .pattern
                    .as_ref()
                    .expect("pattern field on MatchCase was None, should be Some"),
                indent
            )?,
            open_brace,
            {
                self.bound_shift = self.bound_shift + pattern_free;
                self._build_string_from_message(
                    match_case
                        .source
                        .as_ref()
                        .expect("source field on MatchCase was None, should be Some"),
                    indent + 1,
                )?
            },
            close_brace
        ))
    }

    fn is_empty_par(&self, p: &Par) -> bool {
        p.sends.is_empty()
            && p.receives.is_empty()
            && p.news.is_empty()
            && p.exprs.is_empty()
            && p.matches.is_empty()
            && p.unforgeables.is_empty()
            && p.bundles.is_empty()
            && p.connectives.is_empty()
    }

    fn is_new_var(level: &i32, news_shift_indices: Vec<i32>, bound_shift: i32) -> bool {
        news_shift_indices.contains(&(bound_shift - level - 1))
    }
}

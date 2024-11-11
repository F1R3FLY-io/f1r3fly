#[derive(Debug, PartialEq, Clone)]
pub enum Proc {
    Par {
        left: Box<Proc>,
        right: Box<Proc>,
        line_num: usize,
        col_num: usize,
    },

    SendSync {
        name: Name,
        messages: ProcList,
        cont: SyncSendCont,
        line_num: usize,
        col_num: usize,
    },

    New {
        decls: Decls,
        proc: Box<Proc>,
        line_num: usize,
        col_num: usize,
    },

    IfElse {
        condition: Box<Proc>,
        if_true: Box<Proc>,
        alternative: Option<Box<Proc>>,
        line_num: usize,
        col_num: usize,
    },

    Let {
        decls: DeclsChoice,
        body: Box<Block>,
        line_num: usize,
        col_num: usize,
    },

    Bundle {
        bundle_type: BundleType,
        proc: Box<Block>,
        line_num: usize,
        col_num: usize,
    },

    Match {
        expression: Box<Proc>,
        cases: Vec<Case>,
        line_num: usize,
        col_num: usize,
    },

    Choice {
        branches: Vec<Branch>,
        line_num: usize,
        col_num: usize,
    },

    Contract {
        name: Name,
        formals: Names,
        proc: Box<Block>,
        line_num: usize,
        col_num: usize,
    },

    Input {
        formals: Receipts,
        proc: Box<Block>,
        line_num: usize,
        col_num: usize,
    },

    Send {
        name: Name,
        send_type: SendType,
        inputs: ProcList,
        line_num: usize,
        col_num: usize,
    },

    // ProcExpression
    Or {
        left: Box<Proc>,
        right: Box<Proc>,
        line_num: usize,
        col_num: usize,
    },

    And {
        left: Box<Proc>,
        right: Box<Proc>,
        line_num: usize,
        col_num: usize,
    },

    Matches {
        left: Box<Proc>,
        right: Box<Proc>,
        line_num: usize,
        col_num: usize,
    },

    Eq {
        left: Box<Proc>,
        right: Box<Proc>,
        line_num: usize,
        col_num: usize,
    },

    Neq {
        left: Box<Proc>,
        right: Box<Proc>,
        line_num: usize,
        col_num: usize,
    },

    Lt {
        left: Box<Proc>,
        right: Box<Proc>,
        line_num: usize,
        col_num: usize,
    },

    Lte {
        left: Box<Proc>,
        right: Box<Proc>,
        line_num: usize,
        col_num: usize,
    },

    Gt {
        left: Box<Proc>,
        right: Box<Proc>,
        line_num: usize,
        col_num: usize,
    },

    Gte {
        left: Box<Proc>,
        right: Box<Proc>,
        line_num: usize,
        col_num: usize,
    },

    Concat {
        left: Box<Proc>,
        right: Box<Proc>,
        line_num: usize,
        col_num: usize,
    },

    MinusMinus {
        left: Box<Proc>,
        right: Box<Proc>,
        line_num: usize,
        col_num: usize,
    },

    Minus {
        left: Box<Proc>,
        right: Box<Proc>,
        line_num: usize,
        col_num: usize,
    },

    Add {
        left: Box<Proc>,
        right: Box<Proc>,
        line_num: usize,
        col_num: usize,
    },

    PercentPercent {
        left: Box<Proc>,
        right: Box<Proc>,
        line_num: usize,
        col_num: usize,
    },

    Mult {
        left: Box<Proc>,
        right: Box<Proc>,
        line_num: usize,
        col_num: usize,
    },

    Div {
        left: Box<Proc>,
        right: Box<Proc>,
        line_num: usize,
        col_num: usize,
    },

    Mod {
        left: Box<Proc>,
        right: Box<Proc>,
        line_num: usize,
        col_num: usize,
    },

    Not {
        proc: Box<Proc>,
        line_num: usize,
        col_num: usize,
    },

    Neg {
        proc: Box<Proc>,
        line_num: usize,
        col_num: usize,
    },

    Method {
        receiver: Box<Proc>,
        name: Var,
        args: ProcList,
        line_num: usize,
        col_num: usize,
    },

    Eval(Eval),
    Quote(Quote),
    Disjunction(Disjunction),
    Conjunction(Conjunction),
    Negation(Negation),

    // GroundExpression
    Block(Box<Block>),

    Collection(Collection),

    SimpleType(SimpleType),

    // Ground
    BoolLiteral {
        value: bool,
        line_num: usize,
        col_num: usize,
    },

    LongLiteral {
        value: i64,
        line_num: usize,
        col_num: usize,
    },

    StringLiteral {
        value: String,
        line_num: usize,
        col_num: usize,
    },

    UriLiteral(UriLiteral),

    Nil {
        line_num: usize,
        col_num: usize,
    },

    // ProcVar
    Var(Var),

    Wildcard {
        line_num: usize,
        col_num: usize,
    },

    // VarRef
    VarRef(VarRef),
}

impl Proc {
    pub fn new_proc_int(value: i64) -> Proc {
        Proc::LongLiteral {
            value,
            line_num: 0,
            col_num: 0,
        }
    }

    pub fn new_proc_string(value: String) -> Proc {
        Proc::StringLiteral {
            value,
            line_num: 0,
            col_num: 0,
        }
    }

    pub fn new_proc_var(value: &str) -> Proc {
        Proc::Var(Var {
            name: value.to_string(),
            line_num: 0,
            col_num: 0,
        })
    }

    pub fn new_proc_eval(name: Name) -> Proc {
        Proc::Eval(Eval {
            name,
            line_num: 0,
            col_num: 0,
        })
    }

    pub fn new_proc_par_with_int_and_var(value_int: i64, value_var: &str) -> Proc {
        Proc::Par {
            left: Box::new(Self::new_proc_int(value_int)),
            right: Box::new(Self::new_proc_var(value_var)),
            line_num: 0,
            col_num: 0,
        }
    }

    pub fn new_proc_add_with_par_of_var(value_left: &str, value_right: &str) -> Proc {
        Proc::Add {
            left: Box::new(Self::new_proc_var(value_left)),
            right: Box::new(Self::new_proc_var(value_right)),
            line_num: 0,
            col_num: 0,
        }
    }

    pub fn new_proc_wildcard() -> Proc {
        Proc::Wildcard {
            line_num: 0,
            col_num: 0,
        }
    }

    pub fn new_proc_nil() -> Proc {
        Proc::Nil {
            line_num: 0,
            col_num: 0,
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct ProcList {
    pub procs: Vec<Proc>,
    pub line_num: usize,
    pub col_num: usize,
}

impl ProcList {
    pub fn new(procs: Vec<Proc>) -> ProcList {
        ProcList {
            procs,
            line_num: 0,
            col_num: 0,
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum Name {
    ProcVar(Box<Proc>),
    Quote(Box<Quote>),
}

impl Name {
    pub fn new_name_var(name: &str) -> Name {
        Name::ProcVar(Box::new(Proc::Var(Var {
            name: name.to_string(),
            line_num: 0,
            col_num: 0,
        })))
    }

    pub fn new_name_wildcard() -> Name {
        Name::ProcVar(Box::new(Proc::Wildcard {
            line_num: 0,
            col_num: 0,
        }))
    }

    pub fn new_name_quote_var(name: &str) -> Name {
        Name::Quote(Box::new(Quote {
            quotable: Box::new(Proc::Var(Var {
                name: name.to_string(),
                line_num: 0,
                col_num: 0,
            })),
            line_num: 0,
            col_num: 0,
        }))
    }

    pub fn new_name_quote_nil() -> Name {
        Name::Quote(Box::new(Quote {
            quotable: Box::new(Proc::Nil {
                line_num: 0,
                col_num: 0,
            }),
            line_num: 0,
            col_num: 0,
        }))
    }

    pub fn new_name_quote_ground_long_literal(value: i64) -> Name {
        Name::Quote(Box::new(Quote {
            quotable: Box::new(Proc::LongLiteral {
                value,
                line_num: 0,
                col_num: 0,
            }),
            line_num: 0,
            col_num: 0,
        }))
    }

    pub fn new_name_quote_eval(name: &str) -> Name {
        Name::Quote(Box::new(Quote {
            quotable: Box::new(Proc::Eval(Eval {
                name: Name::new_name_var(name),
                line_num: 0,
                col_num: 0,
            })),
            line_num: 0,
            col_num: 0,
        }))
    }

    pub fn new_name_quote_par_of_evals(var_name: &str) -> Name {
        let eval_left = Proc::Eval(Eval {
            name: Name::new_name_var(var_name),
            line_num: 0,
            col_num: 0,
        });

        let eval_right = Proc::Eval(Eval {
            name: Name::new_name_var(var_name),
            line_num: 0,
            col_num: 0,
        });

        let par_proc = Proc::Par {
            left: Box::new(eval_left),
            right: Box::new(eval_right),
            line_num: 0,
            col_num: 0,
        };

        Name::Quote(Box::new(Quote {
            quotable: Box::new(par_proc),
            line_num: 0,
            col_num: 0,
        }))
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Var {
    pub name: String,
    pub line_num: usize,
    pub col_num: usize,
}

impl Var {
    pub fn new(name: String) -> Self {
        Var {
            name,
            line_num: 0,
            col_num: 0,
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Quote {
    pub quotable: Box<Proc>,
    pub line_num: usize,
    pub col_num: usize,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Eval {
    pub name: Name,
    pub line_num: usize,
    pub col_num: usize,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Disjunction {
    pub left: Box<Proc>,
    pub right: Box<Proc>,
    pub line_num: usize,
    pub col_num: usize,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Conjunction {
    pub left: Box<Proc>,
    pub right: Box<Proc>,
    pub line_num: usize,
    pub col_num: usize,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Negation {
    pub proc: Box<Proc>,
    pub line_num: usize,
    pub col_num: usize,
}

impl Negation {
    pub fn new_negation_var(name: &str) -> Proc {
        Proc::Negation(Negation {
            proc: Box::new(Proc::Var(Var {
                name: name.to_string(),
                line_num: 0,
                col_num: 0,
            })),
            line_num: 0,
            col_num: 0,
        })
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Block {
    pub proc: Proc,
    pub line_num: usize,
    pub col_num: usize,
}

impl Block {
    pub fn new(proc: Proc) -> Block {
        Block {
            proc,
            line_num: 0,
            col_num: 0,
        }
    }

    pub fn new_block_nil() -> Block {
        Block {
            proc: Proc::Nil {
                line_num: 0,
                col_num: 0,
            },
            line_num: 0,
            col_num: 0,
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct UriLiteral {
    pub value: String,
    pub line_num: usize,
    pub col_num: usize,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Collection {
    List {
        elements: Vec<Proc>,
        cont: Option<Box<Proc>>,
        line_num: usize,
        col_num: usize,
    },

    Tuple {
        elements: Vec<Proc>,
        line_num: usize,
        col_num: usize,
    },

    Set {
        elements: Vec<Proc>,
        cont: Option<Box<Proc>>,
        line_num: usize,
        col_num: usize,
    },

    Map {
        pairs: Vec<KeyValuePair>,
        cont: Option<Box<Proc>>,
        line_num: usize,
        col_num: usize,
    },
}

#[derive(Debug, PartialEq, Clone)]
pub struct KeyValuePair {
    pub key: Proc,
    pub value: Proc,
    pub line_num: usize,
    pub col_num: usize,
}

#[derive(Debug, PartialEq, Clone)]
pub enum SimpleType {
    Bool { line_num: usize, col_num: usize },
    Int { line_num: usize, col_num: usize },
    String { line_num: usize, col_num: usize },
    Uri { line_num: usize, col_num: usize },
    ByteArray { line_num: usize, col_num: usize },
}

#[derive(Debug, PartialEq, Clone)]
pub enum SyncSendCont {
    Empty {
        line_num: usize,
        col_num: usize,
    },

    NonEmpty {
        proc: Box<Proc>,
        line_num: usize,
        col_num: usize,
    },
}

#[derive(Debug, PartialEq, Clone)]
pub struct Decls {
    pub decls: Vec<NameDecl>,
    pub line_num: usize,
    pub col_num: usize,
}

#[derive(Debug, PartialEq, Clone)]
pub struct NameDecl {
    pub var: Var,
    pub uri: Option<UriLiteral>,
    pub line_num: usize,
    pub col_num: usize,
}

impl NameDecl {
    pub fn new(var_value: &str, uri_value: Option<&str>) -> NameDecl {
        NameDecl {
            var: Var {
                name: var_value.to_string(),
                line_num: 0,
                col_num: 0,
            },
            uri: {
                match uri_value {
                    Some(value) => Some(UriLiteral {
                        value: value.to_string(),
                        line_num: 0,
                        col_num: 0,
                    }),
                    None => None,
                }
            },
            line_num: 0,
            col_num: 0,
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Decl {
    pub names: Names,
    pub procs: Vec<Proc>,
    pub line_num: usize,
    pub col_num: usize,
}

#[derive(Debug, PartialEq, Clone)]
pub enum DeclsChoice {
    LinearDecls {
        decls: Vec<Decl>,
        line_num: usize,
        col_num: usize,
    },

    ConcDecls {
        decls: Vec<Decl>,
        line_num: usize,
        col_num: usize,
    },
}

#[derive(Debug, PartialEq, Clone)]
pub enum BundleType {
    BundleWrite { line_num: usize, col_num: usize },
    BundleRead { line_num: usize, col_num: usize },
    BundleEquiv { line_num: usize, col_num: usize },
    BundleReadWrite { line_num: usize, col_num: usize },
}

#[derive(Debug, PartialEq, Clone)]
pub struct Case {
    pub pattern: Proc,
    pub proc: Proc,
    pub line_num: usize,
    pub col_num: usize,
}

impl Case {
    pub fn new(pattern: Proc, proc: Proc) -> Case {
        Case {
            pattern,
            proc,
            line_num: 0,
            col_num: 0,
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Branch {
    pub pattern: Vec<LinearBind>,
    pub proc: Proc,
    pub line_num: usize,
    pub col_num: usize,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Names {
    pub names: Vec<Name>,
    pub cont: Option<Box<Proc>>,
    pub line_num: usize,
    pub col_num: usize,
}

impl Names {
    pub fn new(names: Vec<Name>, cont: Option<Box<Proc>>) -> Names {
        Names {
            names,
            cont,
            line_num: 0,
            col_num: 0,
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum Source {
    Simple {
        name: Name,
        line_num: usize,
        col_num: usize,
    },

    ReceiveSend {
        name: Name,
        line_num: usize,
        col_num: usize,
    },

    SendReceive {
        name: Name,
        inputs: ProcList,
        line_num: usize,
        col_num: usize,
    },
}

impl Source {
    pub fn new_simple_source(name: Name) -> Source {
        Source::Simple {
            name,
            line_num: 0,
            col_num: 0,
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Receipts {
    pub receipts: Vec<Receipt>,
    pub line_num: usize,
    pub col_num: usize,
}

impl Receipts {
    pub fn new(receipts: Vec<Receipt>) -> Self {
        Receipts {
            receipts,
            line_num: 0,
            col_num: 0,
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum Receipt {
    LinearBinds(LinearBind),

    RepeatedBinds(RepeatedBind),

    PeekBinds(PeekBind),
}

impl Receipt {
    pub fn new_linear_bind_receipt(names: Names, input: Source) -> Receipt {
        Receipt::LinearBinds(LinearBind {
            names,
            input,
            line_num: 0,
            col_num: 0,
        })
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct LinearBind {
    pub names: Names,
    pub input: Source,
    pub line_num: usize,
    pub col_num: usize,
}

#[derive(Debug, PartialEq, Clone)]
pub struct RepeatedBind {
    pub names: Names,
    pub input: Name,
    pub line_num: usize,
    pub col_num: usize,
}

#[derive(Debug, PartialEq, Clone)]
pub struct PeekBind {
    pub names: Names,
    pub input: Name,
    pub line_num: usize,
    pub col_num: usize,
}

#[derive(Debug, PartialEq, Clone)]
pub enum SendType {
    Single { line_num: usize, col_num: usize },

    Multiple { line_num: usize, col_num: usize },
}

impl SendType {
    pub fn new_single() -> Self {
        SendType::Single {
            line_num: 0,
            col_num: 0,
        }
    }

    pub fn new_multiple() -> Self {
        SendType::Multiple {
            line_num: 0,
            col_num: 0,
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct VarRef {
    pub var_ref_kind: VarRefKind,
    pub var: Var,
    pub line_num: usize,
    pub col_num: usize,
}

#[derive(Debug, PartialEq, Clone)]
pub enum VarRefKind {
    Proc,
    Name,
}

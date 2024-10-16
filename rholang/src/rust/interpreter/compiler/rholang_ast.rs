use rspace_plus_plus::rspace::history::Either;

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
        decls: Decls,
        body: Box<Block>,
        line_num: usize,
        col_num: usize,
    },

    Bundle {
        bundle_type: Bundle,
        proc: Box<Block>,
        line_num: usize,
        col_num: usize,
    },

    Match {
        expression: Box<ProcExpression>,
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
}

#[derive(Debug, PartialEq, Clone)]
pub struct ProcList {
    pub procs: Vec<Proc>,
    pub line_num: usize,
    pub col_num: usize,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Name {
    NameProcVar(ProcVar),
    NameQuote(Box<Quote>),
}

#[derive(Debug, PartialEq, Clone)]
pub enum ProcVar {
    Var(Var),

    Wildcard { line_num: usize, col_num: usize },
}

#[derive(Debug, PartialEq, Clone)]
pub struct Var {
    pub name: String,
    pub line_num: usize,
    pub col_num: usize,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Quote {
    pub quotable: Quotable,
    pub line_num: usize,
    pub col_num: usize,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Quotable {
    Eval(Eval),
    Disjunction(Disjunction),
    Conjunction(Conjunction),
    Negation(Negation),
    GroundExpression(GroundExpression),
}

#[derive(Debug, PartialEq, Clone)]
pub struct Eval {
    pub name: Name,
    pub line_num: usize,
    pub col_num: usize,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Disjunction {
    pub left: Proc,
    pub right: Proc,
    pub line_num: usize,
    pub col_num: usize,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Conjunction {
    pub left: Proc,
    pub right: Proc,
    pub line_num: usize,
    pub col_num: usize,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Negation {
    pub proc: Proc,
    pub line_num: usize,
    pub col_num: usize,
}

#[derive(Debug, PartialEq, Clone)]
pub enum GroundExpression {
    Block(Block),

    Ground(Ground),

    Collection(Collection),

    ProcVar(ProcVar),

    SimpleType(SimpleType),
}

#[derive(Debug, PartialEq, Clone)]
pub struct Block {
    pub proc: Proc,
    pub line_num: usize,
    pub col_num: usize,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Ground {
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
        cont: Option<ProcRemainder>,
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
        cont: Option<ProcRemainder>,
        line_num: usize,
        col_num: usize,
    },

    Map {
        pairs: Vec<KeyValuePair>,
        cont: Option<ProcRemainder>,
        line_num: usize,
        col_num: usize,
    },
}

#[derive(Debug, PartialEq, Clone)]
pub struct ProcRemainder {
    pub proc_var: ProcVar,
    pub line_num: usize,
    pub col_num: usize,
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

#[derive(Debug, PartialEq, Clone)]
pub struct Decl {
    pub names: Vec<String>,
    pub procs: Vec<Proc>,
    pub line_num: usize,
    pub col_num: usize,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Bundle {
    BundleWrite { line_num: usize, col_num: usize },
    BundleRead { line_num: usize, col_num: usize },
    BundleEquiv { line_num: usize, col_num: usize },
    BundleReadWrite { line_num: usize, col_num: usize },
}

#[derive(Debug, PartialEq, Clone)]
pub enum ProcExpression {
    Or {
        left: Proc,
        right: Proc,
        line_num: usize,
        col_num: usize,
    },

    And {
        left: Proc,
        right: Proc,
        line_num: usize,
        col_num: usize,
    },

    Matches {
        left: Proc,
        right: Proc,
        line_num: usize,
        col_num: usize,
    },

    Eq {
        left: Proc,
        right: Proc,
        line_num: usize,
        col_num: usize,
    },

    Neq {
        left: Proc,
        right: Proc,
        line_num: usize,
        col_num: usize,
    },

    Lt {
        left: Proc,
        right: Proc,
        line_num: usize,
        col_num: usize,
    },

    Lte {
        left: Proc,
        right: Proc,
        line_num: usize,
        col_num: usize,
    },

    Gt {
        left: Proc,
        right: Proc,
        line_num: usize,
        col_num: usize,
    },

    Gte {
        left: Proc,
        right: Proc,
        line_num: usize,
        col_num: usize,
    },

    Concat {
        left: Proc,
        right: Proc,
        line_num: usize,
        col_num: usize,
    },

    MinusMinus {
        left: Proc,
        right: Proc,
        line_num: usize,
        col_num: usize,
    },

    Minus {
        left: Proc,
        right: Proc,
        line_num: usize,
        col_num: usize,
    },

    Add {
        left: Proc,
        right: Proc,
        line_num: usize,
        col_num: usize,
    },

    PercentPercent {
        left: Proc,
        right: Proc,
        line_num: usize,
        col_num: usize,
    },

    Mult {
        left: Proc,
        right: Proc,
        line_num: usize,
        col_num: usize,
    },

    Div {
        left: Proc,
        right: Proc,
        line_num: usize,
        col_num: usize,
    },

    Mod {
        left: Proc,
        right: Proc,
        line_num: usize,
        col_num: usize,
    },

    Not {
        proc: Proc,
        line_num: usize,
        col_num: usize,
    },

    Neg {
        proc: Proc,
        line_num: usize,
        col_num: usize,
    },

    Method {
        receiver: Proc,
        name: Var,
        args: ProcList,
        line_num: usize,
        col_num: usize,
    },

    Parenthesized {
        proc_expression: Box<ProcExpression>,
        line_num: usize,
        col_num: usize,
    },

    Eval(Eval),
    Quote(Quote),
    Disjunction(Disjunction),
    Conjunction(Conjunction),
    Negation(Negation),
    GroundExpression(GroundExpression),
}

#[derive(Debug, PartialEq, Clone)]
pub struct Case {
    pub pattern: Proc,
    pub proc: Proc,
    pub line_num: usize,
    pub col_num: usize,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Branch {
    pub pattern: Vec<LinearBind>,
    pub proc: Either<Proc, ProcExpression>,
    pub line_num: usize,
    pub col_num: usize,
}

#[derive(Debug, PartialEq, Clone)]
pub struct LinearBind {
    pub names: Names,
    pub input: Source,
    pub line_num: usize,
    pub col_num: usize,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Names {
    pub names: Vec<Name>,
    pub cont: Option<NameRemainder>,
    pub line_num: usize,
    pub col_num: usize,
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

#[derive(Debug, PartialEq, Clone)]
pub enum Receipt {
    LinearBind(LinearBind),

    RepeatedBind {
        names: Names,
        input: Name,
        line_num: usize,
        col_num: usize,
    },

    PeekBind {
        names: Names,
        input: Name,
        line_num: usize,
        col_num: usize,
    },
}

#[derive(Debug, PartialEq, Clone)]
pub struct NameRemainder {
    pub proc_var: ProcVar,
    pub line_num: usize,
    pub col_num: usize,
}

#[derive(Debug, PartialEq, Clone)]
pub struct PVarRef {
    pub var_ref_kind: VarRefKind,
    pub var: String,
    pub line_num: usize,
    pub col_num: usize,
}

#[derive(Debug, PartialEq, Clone)]
pub enum VarRefKind {
    Proc,
    Name,
}

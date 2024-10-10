#[derive(Debug, PartialEq, Clone)]
pub enum Proc {
    Par {
        left: Box<Proc>,
        right: Box<Proc>,
        line_num: usize,
        col_num: usize,
    },

    SendSync {
        name: String,
        messages: Vec<Proc>,
        cont: SyncSendCont,
        line_num: usize,
        col_num: usize,
    },

    New {
        decls: Vec<NameDecl>,
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
        decls: Vec<Decl>,
        body: Box<Proc>,
        line_num: usize,
        col_num: usize,
    },

    Bundle {
        bundle_type: BundleType,
        proc: Box<Proc>,
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
        name: String,
        formals: Vec<String>,
        proc: Box<Proc>,
        line_num: usize,
        col_num: usize,
    },

    Input {
        formals: Vec<Receipt>,
        proc: Box<Proc>,
        line_num: usize,
        col_num: usize,
    },

    Send {
        name: String,
        send_type: SendType,
        inputs: Vec<Proc>,
        line_num: usize,
        col_num: usize,
    },

    ProcExpression(ProcExpression),
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
pub struct NameDecl {
    pub var: String,
    pub uri: Option<String>,
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
pub enum BundleType {
    Write { line_num: usize, col_num: usize },
    Read { line_num: usize, col_num: usize },
    Equiv { line_num: usize, col_num: usize },
    ReadWrite { line_num: usize, col_num: usize },
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
    pub pattern: Vec<Receipt>,
    pub proc: Proc,
    pub line_num: usize,
    pub col_num: usize,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Receipt {
    LinearBind {
        names: Vec<String>,
        input: Source,
        line_num: usize,
        col_num: usize,
    },

    RepeatedBind {
        names: Vec<String>,
        input: String,
        line_num: usize,
        col_num: usize,
    },

    PeekBind {
        names: Vec<String>,
        input: String,
        line_num: usize,
        col_num: usize,
    },
}

#[derive(Debug, PartialEq, Clone)]
pub enum Source {
    Simple {
        name: String,
        line_num: usize,
        col_num: usize,
    },

    ReceiveSend {
        name: String,
        line_num: usize,
        col_num: usize,
    },

    SendReceive {
        name: String,
        inputs: Vec<Proc>,
        line_num: usize,
        col_num: usize,
    },
}

#[derive(Debug, PartialEq, Clone)]
pub enum SendType {
    Single { line_num: usize, col_num: usize },
    Multiple { line_num: usize, col_num: usize },
}

#[derive(Debug, PartialEq, Clone)]
pub enum ProcExpression {
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
        name: String,
        args: Vec<Proc>,
        line_num: usize,
        col_num: usize,
    },

    Eval {
        name: String,
        line_num: usize,
        col_num: usize,
    },

    Disjunction {
        left: Box<Proc>,
        right: Box<Proc>,
        line_num: usize,
        col_num: usize,
    },

    Conjunction {
        left: Box<Proc>,
        right: Box<Proc>,
        line_num: usize,
        col_num: usize,
    },

    Negation {
        proc: Box<Proc>,
        line_num: usize,
        col_num: usize,
    },

    GroundExpression {
        ground: GroundExpression,
        line_num: usize,
        col_num: usize,
    },
}

#[derive(Debug, PartialEq, Clone)]
pub enum GroundExpression {
    Block {
        proc: Box<Proc>,
        line_num: usize,
        col_num: usize,
    },

    Ground {
        ground: Ground,
        line_num: usize,
        col_num: usize,
    },

    Collection {
        collection: Collection,
        line_num: usize,
        col_num: usize,
    },

    ProcVar {
        name: String,
        line_num: usize,
        col_num: usize,
    },

    SimpleType {
        simple_type: SimpleType,
        line_num: usize,
        col_num: usize,
    },
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

    UriLiteral {
        value: String,
        line_num: usize,
        col_num: usize,
    },

    Nil {
        line_num: usize,
        col_num: usize,
    },
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
pub enum Collection {
    List {
        elements: Vec<Proc>,
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
        line_num: usize,
        col_num: usize,
    },

    Map {
        pairs: Vec<KeyValuePair>,
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
pub enum PVar {
    ProcVarVar {
        name: String,
        line_num: usize,
        col_num: usize,
    },

    ProcVarWildcard {
        line_num: usize,
        col_num: usize,
    },
}

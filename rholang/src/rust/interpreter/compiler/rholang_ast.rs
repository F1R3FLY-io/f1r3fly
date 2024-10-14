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
        messages: Vec<Proc>,
        cont: SyncSendCont,
        line_num: usize,
        col_num: usize,
    },

    // PGround(Ground),
    // PCollect(Collection),
    // PVar(ProcVar),
    // PVarRef(VarRefKind, Var),
    // PNil,
    // PSimpleType(SimpleType),
    // PNegation(Box<Proc>),
    // PConjunction(Box<Proc>, Box<Proc>),
    // PDisjunction(Box<Proc>, Box<Proc>),
    // PEval(Name),
    // PMethod(Box<Proc>, Var, Vec<Proc>),
    // PExprs(Box<Proc>),
    // PNot(Box<Proc>),
    // PNeg(Box<Proc>),
    // PMult(Box<Proc>, Box<Proc>),
    // PDiv(Box<Proc>, Box<Proc>),
    // PMod(Box<Proc>, Box<Proc>),
    // PPercentPercent(Box<Proc>, Box<Proc>),
    // PAdd(Box<Proc>, Box<Proc>),
    // PMinus(Box<Proc>, Box<Proc>),
    // PPlusPlus(Box<Proc>, Box<Proc>),
    // PMinusMinus(Box<Proc>, Box<Proc>),
    // PLt(Box<Proc>, Box<Proc>),
    // PLte(Box<Proc>, Box<Proc>),
    // PGt(Box<Proc>, Box<Proc>),
    // PGte(Box<Proc>, Box<Proc>),
    // PMatches(Box<Proc>, Box<Proc>),
    // PEq(Box<Proc>, Box<Proc>),
    // PNeq(Box<Proc>, Box<Proc>),
    // PAnd(Box<Proc>, Box<Proc>),
    // POr(Box<Proc>, Box<Proc>),
    // PSend(Name, Send, Vec<Proc>),
    // PContr(Name, Vec<Name>, NameRemainder, Box<Proc>),
    // PInput(Vec<Receipt>, Box<Proc>),
    // PChoice(Vec<Branch>),
    // PMatch(Box<Proc>, Vec<Case>),
    // PBundle(Bundle, Box<Proc>),
    // PLet(Decl, Decls, Box<Proc>),
    // PIf(Box<Proc>, Box<Proc>),
    // PIfElse(Box<Proc>, Box<Proc>, Box<Proc>),
    // PNew(Vec<NameDecl>, Box<Proc>),
}

#[derive(Debug, PartialEq, Clone)]
pub enum Name {
    NameProcVar(ProcVar),
    NameQuote(Box<Quote>),
}

#[derive(Debug, PartialEq, Clone)]
pub enum ProcVar {
    Var {
        name: String,
        line_num: usize,
        col_num: usize,
    },

    Wildcard {
        line_num: usize,
        col_num: usize,
    },
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
    left: Proc,
    right: Proc,
    line_num: usize,
    col_num: usize,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Conjunction {
    left: Proc,
    right: Proc,
    line_num: usize,
    col_num: usize,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Negation {
    proc: Proc,
    line_num: usize,
    col_num: usize,
}

#[derive(Debug, PartialEq, Clone)]
pub enum GroundExpression {
    Block(Block),

    Ground(Ground),

    Collection(Collection),

    SimpleType(SimpleType),
}

#[derive(Debug, PartialEq, Clone)]
pub struct Block {
    proc: Proc,
    line_num: usize,
    col_num: usize,
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

    Nil(Nil),
}

#[derive(Debug, PartialEq, Clone)]
pub struct UriLiteral {
    value: String,
    line_num: usize,
    col_num: usize,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Nil {
    line_num: usize,
    col_num: usize,
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

//
//

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

    Negation {
        proc: Box<Proc>,
        line_num: usize,
        col_num: usize,
    },

    GroundExpression(GroundExpression),
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

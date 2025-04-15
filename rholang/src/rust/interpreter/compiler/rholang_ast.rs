use tree_sitter::Node;
use typed_arena::Arena;

use crate::rust::interpreter::errors::InterpreterError;

use super::normalizer::processes::exports::SourcePosition;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Proc<'ast> {
    Par {
        left: AnnProc<'ast>,
        right: AnnProc<'ast>,
    },

    SendSync {
        name: Name<'ast>,
        messages: Vec<AnnProc<'ast>>,
        cont: SyncSendCont<'ast>,
    },

    New {
        decls: Vec<NameDecl<'ast>>,
        proc: &'ast Proc<'ast>,
    },

    IfThenElse {
        condition: AnnProc<'ast>,
        if_true: AnnProc<'ast>,
        if_false: Option<AnnProc<'ast>>,
    },

    Let {
        bindings: Vec<Binding<'ast>>,
        body: AnnProc<'ast>,
        concurrent: bool,
    },

    Bundle {
        bundle_type: BundleType,
        proc: &'ast Proc<'ast>,
    },

    Match {
        expression: AnnProc<'ast>,
        cases: Vec<Case<'ast>>,
    },

    Choice {
        branches: Vec<Branch<'ast>>,
    },

    Contract {
        name: AnnName<'ast>,
        formals: Names<'ast>,
        body: AnnProc<'ast>,
    },

    ForComprehension {
        receipts: Vec<Receipt<'ast>>,
        proc: AnnProc<'ast>,
    },

    Send {
        name: Name<'ast>,
        send_type: SendType,
        inputs: Vec<AnnProc<'ast>>,
    },

    // ProcExpression
    Or {
        left: AnnProc<'ast>,
        right: AnnProc<'ast>,
    },

    And {
        left: AnnProc<'ast>,
        right: AnnProc<'ast>,
    },

    Matches {
        target: AnnProc<'ast>,
        pattern: AnnProc<'ast>,
    },

    Eq {
        left: AnnProc<'ast>,
        right: AnnProc<'ast>,
    },

    Neq {
        left: AnnProc<'ast>,
        right: AnnProc<'ast>,
    },

    Lt {
        left: AnnProc<'ast>,
        right: AnnProc<'ast>,
    },

    Lte {
        left: AnnProc<'ast>,
        right: AnnProc<'ast>,
    },

    Gt {
        left: AnnProc<'ast>,
        right: AnnProc<'ast>,
    },

    Gte {
        left: AnnProc<'ast>,
        right: AnnProc<'ast>,
    },

    Concat {
        left: AnnProc<'ast>,
        right: AnnProc<'ast>,
    },

    Diff {
        left: AnnProc<'ast>,
        right: AnnProc<'ast>,
    },

    Sub {
        left: AnnProc<'ast>,
        right: AnnProc<'ast>,
    },

    Add {
        left: AnnProc<'ast>,
        right: AnnProc<'ast>,
    },

    Interpolation {
        left: AnnProc<'ast>,
        right: AnnProc<'ast>,
    },

    Mult {
        left: AnnProc<'ast>,
        right: AnnProc<'ast>,
    },

    Div {
        left: AnnProc<'ast>,
        right: AnnProc<'ast>,
    },

    Mod {
        left: AnnProc<'ast>,
        right: AnnProc<'ast>,
    },
    Not(&'ast Proc<'ast>),
    Neg(&'ast Proc<'ast>),

    Method {
        receiver: &'ast Proc<'ast>,
        name: Id<'ast>,
        args: Vec<AnnProc<'ast>>,
    },

    Eval {
        name: AnnName<'ast>,
    },
    Quote(&'ast Proc<'ast>),

    Disjunction {
        left: AnnProc<'ast>,
        right: AnnProc<'ast>,
    },
    Conjunction {
        left: AnnProc<'ast>,
        right: AnnProc<'ast>,
    },
    Negation(&'ast Proc<'ast>),

    // GroundExpression
    Collection(Collection<'ast>),
    SimpleType(SimpleType),

    // Ground
    BoolLiteral(bool),
    LongLiteral(i64),
    StringLiteral(&'ast str),
    UriLiteral(Uri<'ast>),
    Nil,

    // ProcVar
    ProcVar(Var<'ast>),

    // VarRef
    VarRef(VarRef<'ast>),
}

impl<'a> Proc<'a> {
    pub fn new_var(name: &'a str, pos: SourcePosition) -> Proc<'a> {
        Proc::ProcVar(Var::new_id(name, pos))
    }

    pub fn new_list(elements: Vec<AnnProc<'a>>) -> Proc<'a> {
        Proc::Collection(Collection::List {
            elements,
            cont: None,
        })
    }

    pub fn new_tuple(elements: Vec<AnnProc<'a>>) -> Proc<'a> {
        Proc::Collection(Collection::Tuple { elements })
    }

    pub fn annotate(&'a self, pos: SourcePosition) -> AnnProc<'a> {
        AnnProc { proc: self, pos }
    }

    pub fn annotate_dummy(&'a self) -> AnnProc<'a> {
        AnnProc {
            proc: self,
            pos: SourcePosition::default(),
        }
    }

    pub fn quoted(&'a self) -> Name<'a> {
        Name::Quote(self)
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct AnnProc<'ast> {
    pub proc: &'ast Proc<'ast>,
    pub pos: SourcePosition,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Name<'ast> {
    ProcVar(Var<'ast>),
    Quote(&'ast Proc<'ast>),
}

impl<'a> Name<'a> {
    pub fn declare(name: &'a str) -> Name<'a> {
        Id {
            name,
            pos: SourcePosition::default(),
        }
        .as_name()
    }

    pub fn annotated(self, pos: SourcePosition) -> AnnName<'a> {
        AnnName(self, pos)
    }

    pub fn annotated_dummy(self) -> AnnName<'a> {
        AnnName(self, SourcePosition::default())
    }
}

impl<'a> TryFrom<Proc<'a>> for Name<'a> {
    type Error = String;

    fn try_from(proc: Proc<'a>) -> Result<Name<'a>, String> {
        match proc {
            Proc::ProcVar(var) => Ok(Name::ProcVar(var)),
            Proc::Quote(proc) => Ok(Name::Quote(proc)),
            other => Err(format!("{other:?} is not a name")),
        }
    }
}

impl<'a> TryFrom<AnnProc<'a>> for Name<'a> {
    type Error = String;

    fn try_from(ann_proc: AnnProc<'a>) -> Result<Name<'a>, String> {
        match ann_proc.proc {
            Proc::ProcVar(var) => Ok(Name::ProcVar(*var)),
            Proc::Quote(proc) => Ok(Name::Quote(*proc)),
            other => Err(format!("{other:?} is not a name")),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct AnnName<'ast>(pub Name<'ast>, pub SourcePosition);

impl<'a> TryFrom<AnnProc<'a>> for AnnName<'a> {
    type Error = String;

    fn try_from(value: AnnProc<'a>) -> Result<AnnName<'a>, String> {
        let p = value.proc;
        let pos = value.pos;
        match p {
            Proc::ProcVar(var) => Ok(AnnName(var.as_name(), pos)),
            Proc::Quote(proc) => Ok(AnnName((*proc).quoted(), pos)),
            other => Err(format!("{other:?} at {pos} is not a name")),
        }
    }
}

impl<'a> AnnName<'a> {
    pub fn declare(name: &'a str, pos: SourcePosition) -> AnnName<'a> {
        Id { name, pos }.as_name().annotated(pos)
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Var<'a> {
    Id(Id<'a>),
    Wildcard,
}

impl<'a> Var<'a> {
    pub fn new_id(name: &'a str, pos: SourcePosition) -> Var<'a> {
        Var::Id(Id { name, pos })
    }

    pub fn as_name(&self) -> Name<'a> {
        Name::ProcVar(*self)
    }

    pub fn as_proc(&self) -> Proc<'a> {
        Proc::ProcVar(*self)
    }
}

impl<'a> TryFrom<Name<'a>> for Var<'a> {
    type Error = String;

    fn try_from(value: Name<'a>) -> Result<Var<'a>, String> {
        match value {
            Name::ProcVar(var) => Ok(var),
            Name::Quote(quoted) => Err(format!(
                "attempt to convert a quoted process {{ {quoted:?} }} to a var"
            )),
        }
    }
}

impl<'a> TryFrom<&Proc<'a>> for Var<'a> {
    type Error = String;

    fn try_from(value: &Proc<'a>) -> Result<Var<'a>, String> {
        match value {
            Proc::ProcVar(var) => Ok(*var),
            other => Err(format!("attempt to convert {{ {other:?} }} to a var")),
        }
    }
}

impl<'a> TryFrom<AnnName<'a>> for Var<'a> {
    type Error = String;

    fn try_from(value: AnnName<'a>) -> Result<Var<'a>, String> {
        value.0.try_into()
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct Id<'ast> {
    pub name: &'ast str,
    pub pos: SourcePosition,
}

impl<'a> Id<'a> {
    pub fn as_name(&self) -> Name<'a> {
        Var::Id(*self).as_name()
    }

    pub fn as_proc(&self) -> Proc<'a> {
        Var::Id(*self).as_proc()
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct ProcRemainder<'a> {
    pub var: Var<'a>,
    pub pos: SourcePosition,
}

pub type NameRemainder<'b> = ProcRemainder<'b>;

impl<'a> TryFrom<AnnProc<'a>> for ProcRemainder<'a> {
    type Error = String;

    fn try_from(value: AnnProc<'a>) -> Result<Self, Self::Error> {
        value.proc.try_into().map(|var| ProcRemainder {
            var,
            pos: value.pos,
        })
    }
}

impl<'a> TryFrom<AnnName<'a>> for ProcRemainder<'a> {
    type Error = String;

    fn try_from(value: AnnName<'a>) -> Result<Self, Self::Error> {
        value
            .try_into()
            .map(|var| ProcRemainder { var, pos: value.1 })
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Collection<'ast> {
    List {
        elements: Vec<AnnProc<'ast>>,
        cont: Option<ProcRemainder<'ast>>,
    },

    Tuple {
        elements: Vec<AnnProc<'ast>>,
    },

    Set {
        elements: Vec<AnnProc<'ast>>,
        cont: Option<ProcRemainder<'ast>>,
    },

    Map {
        pairs: Vec<KeyValuePair<'ast>>,
        cont: Option<ProcRemainder<'ast>>,
    },
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct KeyValuePair<'ast> {
    pub key: AnnProc<'ast>,
    pub value: AnnProc<'ast>,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum SimpleType {
    Bool,
    Int,
    String,
    Uri,
    ByteArray,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum SyncSendCont<'ast> {
    Empty,
    NonEmpty(AnnProc<'ast>),
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct NameDecl<'ast> {
    pub id: Id<'ast>,
    pub uri: Option<Uri<'ast>>,
}

impl<'a> NameDecl<'a> {
    pub fn id(id: &'a str, pos: SourcePosition) -> NameDecl<'a> {
        NameDecl {
            id: Id { name: id, pos },
            uri: None,
        }
    }

    pub fn id_with_uri(id: &'a str, uri: Uri<'a>, pos: SourcePosition) -> NameDecl<'a> {
        NameDecl {
            id: Id { name: id, pos },
            uri: Some(uri),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Binding<'ast> {
    pub names: Names<'ast>,
    pub procs: Vec<AnnProc<'ast>>,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum BundleType {
    BundleWrite,
    BundleRead,
    BundleEquiv,
    BundleReadWrite,
}

pub struct BundleFlags {
    pub read_flag: bool,
    pub write_flag: bool,
}

impl From<BundleType> for BundleFlags {
    fn from(value: BundleType) -> Self {
        match value {
            BundleType::BundleWrite => BundleFlags {
                read_flag: false,
                write_flag: true,
            },
            BundleType::BundleRead => BundleFlags {
                read_flag: true,
                write_flag: false,
            },
            BundleType::BundleReadWrite => BundleFlags {
                read_flag: true,
                write_flag: true,
            },
            BundleType::BundleEquiv => BundleFlags {
                read_flag: false,
                write_flag: false,
            },
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct Case<'ast> {
    pub pattern: AnnProc<'ast>,
    pub proc: AnnProc<'ast>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Branch<'ast> {
    pub patterns: Vec<LinearBind<'ast>>,
    pub proc: AnnProc<'ast>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Names<'ast> {
    pub names: Vec<AnnName<'ast>>,
    pub remainder: Option<NameRemainder<'ast>>,
}

impl Clone for Names<'_> {
    fn clone(&self) -> Self {
        let mut dest_names = Vec::with_capacity(self.names.len());
        dest_names.extend(&self.names);

        Names {
            names: dest_names,
            remainder: self.remainder,
        }
    }

    fn clone_from(&mut self, source: &Self) {
        self.names.clone_from(&source.names);
        self.remainder = source.remainder;
    }
}

impl<'a> Names<'a> {
    pub fn from_slice(slice: &[AnnName<'a>], with_remainder: bool) -> Result<Names<'a>, String> {
        let mut names = Vec::new();
        if with_remainder {
            match slice.split_last() {
                None => Err("attempt to build 'x, y ...@z' out of zero names".to_string()),
                Some((_, init)) if init.is_empty() => {
                    Err("attempt to build 'x, y ...@z' out of one name".to_string())
                }
                Some((last, init)) => {
                    names.extend(init);
                    Ok(Names {
                        names,
                        remainder: Some((*last).try_into().expect("a variable expected")),
                    })
                }
            }
        } else {
            if slice.is_empty() {
                Err("attempt to build empty names".to_string())
            } else {
                names.extend(slice);
                Ok(Names {
                    names,
                    remainder: None,
                })
            }
        }
    }

    pub fn single(name: AnnName<'a>) -> Names<'a> {
        Names {
            names: vec![name],
            remainder: None,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Source<'ast> {
    Simple {
        name: AnnName<'ast>,
    },
    ReceiveSend {
        name: AnnName<'ast>,
    },
    SendReceive {
        name: AnnName<'ast>,
        inputs: Vec<AnnProc<'ast>>,
    },
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Receipt<'ast> {
    Linear(Vec<LinearBind<'ast>>),
    Repeated(Vec<RepeatedBind<'ast>>),
    Peek(Vec<PeekBind<'ast>>),
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct LinearBind<'ast> {
    pub lhs: Names<'ast>,
    pub rhs: Source<'ast>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct RepeatedBind<'ast> {
    pub lhs: Names<'ast>,
    pub rhs: AnnName<'ast>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct PeekBind<'ast> {
    pub lhs: Names<'ast>,
    pub rhs: AnnName<'ast>,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum SendType {
    Single,
    Multiple,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct VarRef<'a> {
    pub kind: VarRefKind,
    pub var: Id<'a>,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum VarRefKind {
    Proc,
    Name,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, PartialOrd, Ord)]
pub struct Uri<'a>(&'a str);

impl<'a> From<&'a str> for Uri<'a> {
    fn from(value: &'a str) -> Self {
        Uri(value.trim_matches('`'))
    }
}

impl ToString for Uri<'_> {
    fn to_string(&self) -> String {
        self.0.to_string()
    }
}

pub struct ASTBuilder<'ast> {
    arena: Arena<Proc<'ast>>,
    source_code: &'ast [u8],
}

impl<'ast> ASTBuilder<'ast> {
    pub fn new(source_code: &'ast str) -> ASTBuilder<'ast> {
        ASTBuilder {
            arena: Arena::with_capacity(64),
            source_code: source_code.as_ref(),
        }
    }

    pub fn with_capacity(capacity: usize) -> ASTBuilder<'ast> {
        ASTBuilder {
            arena: Arena::with_capacity(capacity),
            source_code: &[],
        }
    }

    pub fn get_source_code(&self) -> &[u8] {
        self.source_code
    }

    pub fn alloc_long_literal(&'ast self, value: i64) -> &'ast Proc<'ast> {
        self.arena.alloc(Proc::LongLiteral(value))
    }

    pub fn alloc_string_literal(&'ast self, value: &'ast str) -> &'ast Proc<'ast> {
        self.arena
            .alloc(Proc::StringLiteral(value.trim_matches('"')))
    }

    pub fn alloc_uri_literal(&'ast self, value: Uri<'ast>) -> &'ast Proc<'ast> {
        self.arena.alloc(Proc::UriLiteral(value))
    }

    pub fn alloc_var(&'ast self, id: Id<'ast>) -> &'ast Proc<'ast> {
        self.arena.alloc(Proc::ProcVar(Var::Id(id)))
    }

    pub fn alloc_var_ref(&'ast self, var_ref: VarRef<'ast>) -> &'ast Proc<'ast> {
        self.arena.alloc(Proc::VarRef(var_ref))
    }

    pub fn alloc_var_from_str(&'ast self, name: &'ast str, pos: SourcePosition) -> &'ast Proc<'ast> {
        self.alloc_var(Id { name, pos })
    }

    pub fn alloc_par(&'ast self, left: AnnProc<'ast>, right: AnnProc<'ast>) -> &'ast Proc<'ast> {
        self.arena.alloc(Proc::Par { left, right })
    }

    pub fn alloc_send_sync(
        &'ast self,
        name: Name<'ast>,
        messages: Vec<AnnProc<'ast>>,
    ) -> &'ast Proc<'ast> {
        self.arena.alloc(Proc::SendSync {
            name,
            messages,
            cont: SyncSendCont::Empty,
        })
    }

    pub fn alloc_send_sync_with_cont(
        &'ast self,
        name: Name<'ast>,
        messages: Vec<AnnProc<'ast>>,
        cont: AnnProc<'ast>,
    ) -> &'ast Proc<'ast> {
        self.arena.alloc(Proc::SendSync {
            name,
            messages,
            cont: SyncSendCont::NonEmpty(cont),
        })
    }

    pub fn alloc_new(
        &'ast self,
        proc: &'ast Proc<'ast>,
        decls: Vec<NameDecl<'ast>>,
    ) -> &'ast Proc<'ast> {
        self.arena.alloc(Proc::New { decls, proc })
    }

    pub fn alloc_if_then(
        &'ast self,
        condition: AnnProc<'ast>,
        if_true: AnnProc<'ast>,
    ) -> &'ast Proc<'ast> {
        self.arena.alloc(Proc::IfThenElse {
            condition,
            if_true,
            if_false: None,
        })
    }

    pub fn alloc_if_then_else(
        &'ast self,
        condition: AnnProc<'ast>,
        if_true: AnnProc<'ast>,
        if_false: AnnProc<'ast>,
    ) -> &'ast Proc<'ast> {
        self.arena.alloc(Proc::IfThenElse {
            condition,
            if_true,
            if_false: Some(if_false),
        })
    }

    pub fn alloc_let(
        &'ast self,
        bindings: Vec<Binding<'ast>>,
        body: AnnProc<'ast>,
        concurrent: bool,
    ) -> &'ast Proc<'ast> {
        self.arena.alloc(Proc::Let {
            bindings,
            body,
            concurrent,
        })
    }

    pub fn alloc_bundle(
        &'ast self,
        bundle_type: BundleType,
        proc: &'ast Proc<'ast>,
    ) -> &'ast Proc<'ast> {
        self.arena.alloc(Proc::Bundle { bundle_type, proc })
    }

    pub fn alloc_match(
        &'ast self,
        expression: AnnProc<'ast>,
        cases: Vec<Case<'ast>>,
    ) -> &'ast Proc<'ast> {
        self.arena.alloc(Proc::Match { expression, cases })
    }

    pub fn alloc_contract(
        &'ast self,
        name: AnnName<'ast>,
        formals: Names<'ast>,
        body: AnnProc<'ast>,
    ) -> &'ast Proc<'ast> {
        self.arena.alloc(Proc::Contract {
            name,
            formals,
            body,
        })
    }

    pub fn alloc_for(
        &'ast self,
        receipts: Vec<Receipt<'ast>>,
        proc: AnnProc<'ast>,
    ) -> &'ast Proc<'ast> {
        self.arena.alloc(Proc::ForComprehension { receipts, proc })
    }

    pub fn alloc_send(
        &'ast self,
        send_type: SendType,
        name: Name<'ast>,
        inputs: Vec<AnnProc<'ast>>,
    ) -> &'ast Proc<'ast> {
        self.arena.alloc(Proc::Send {
            name,
            send_type,
            inputs,
        })
    }

    pub fn alloc_method(
        &'ast self,
        name: Id<'ast>,
        receiver: &'ast Proc<'ast>,
        args: Vec<AnnProc<'ast>>,
    ) -> &'ast Proc<'ast> {
        self.arena.alloc(Proc::Method {
            receiver,
            name,
            args,
        })
    }

    pub fn alloc_eval(&'ast self, name: AnnName<'ast>) -> &'ast Proc<'ast> {
        self.arena.alloc(Proc::Eval { name })
    }

    pub fn alloc_quote(&'ast self, proc: &'ast Proc<'ast>) -> &'ast Proc<'ast> {
        self.arena.alloc(Proc::Quote(proc))
    }

    pub fn alloc_or(&'ast self, left: AnnProc<'ast>, right: AnnProc<'ast>) -> &'ast Proc {
        self.arena.alloc(Proc::Or { left, right })
    }

    pub fn alloc_and(&'ast self, left: AnnProc<'ast>, right: AnnProc<'ast>) -> &'ast Proc {
        self.arena.alloc(Proc::And { left, right })
    }

    pub fn alloc_matches(&'ast self, target: AnnProc<'ast>, pattern: AnnProc<'ast>) -> &'ast Proc {
        self.arena.alloc(Proc::Matches { target, pattern })
    }

    pub fn alloc_eq(&'ast self, left: AnnProc<'ast>, right: AnnProc<'ast>) -> &'ast Proc {
        self.arena.alloc(Proc::Eq { left, right })
    }

    pub fn alloc_neq(&'ast self, left: AnnProc<'ast>, right: AnnProc<'ast>) -> &'ast Proc {
        self.arena.alloc(Proc::Neq { left, right })
    }

    pub fn alloc_lt(&'ast self, left: AnnProc<'ast>, right: AnnProc<'ast>) -> &'ast Proc {
        self.arena.alloc(Proc::Lt { left, right })
    }

    pub fn alloc_lte(&'ast self, left: AnnProc<'ast>, right: AnnProc<'ast>) -> &'ast Proc {
        self.arena.alloc(Proc::Lte { left, right })
    }

    pub fn alloc_gt(&'ast self, left: AnnProc<'ast>, right: AnnProc<'ast>) -> &'ast Proc {
        self.arena.alloc(Proc::Gt { left, right })
    }

    pub fn alloc_gte(&'ast self, left: AnnProc<'ast>, right: AnnProc<'ast>) -> &'ast Proc {
        self.arena.alloc(Proc::Gte { left, right })
    }

    pub fn alloc_concat(&'ast self, left: AnnProc<'ast>, right: AnnProc<'ast>) -> &'ast Proc {
        self.arena.alloc(Proc::Concat { left, right })
    }

    pub fn alloc_diff(&'ast self, left: AnnProc<'ast>, right: AnnProc<'ast>) -> &'ast Proc {
        self.arena.alloc(Proc::Diff { left, right })
    }

    pub fn alloc_sub(&'ast self, left: AnnProc<'ast>, right: AnnProc<'ast>) -> &'ast Proc {
        self.arena.alloc(Proc::Sub { left, right })
    }

    pub fn alloc_add(&'ast self, left: AnnProc<'ast>, right: AnnProc<'ast>) -> &'ast Proc {
        self.arena.alloc(Proc::Add { left, right })
    }

    pub fn alloc_interpolation(
        &'ast self,
        left: AnnProc<'ast>,
        right: AnnProc<'ast>,
    ) -> &'ast Proc {
        self.arena.alloc(Proc::Interpolation { left, right })
    }

    pub fn alloc_mult(&'ast self, left: AnnProc<'ast>, right: AnnProc<'ast>) -> &'ast Proc {
        self.arena.alloc(Proc::Mult { left, right })
    }

    pub fn alloc_div(&'ast self, left: AnnProc<'ast>, right: AnnProc<'ast>) -> &'ast Proc {
        self.arena.alloc(Proc::Div { left, right })
    }

    pub fn alloc_mod(&'ast self, left: AnnProc<'ast>, right: AnnProc<'ast>) -> &'ast Proc {
        self.arena.alloc(Proc::Mod { left, right })
    }

    pub fn alloc_conjunction(&'ast self, left: AnnProc<'ast>, right: AnnProc<'ast>) -> &'ast Proc {
        self.arena.alloc(Proc::Conjunction { left, right })
    }

    pub fn alloc_disjunction(&'ast self, left: AnnProc<'ast>, right: AnnProc<'ast>) -> &'ast Proc {
        self.arena.alloc(Proc::Disjunction { left, right })
    }

    pub fn alloc_not(&'ast self, proc: &'ast Proc) -> &'ast Proc {
        self.arena.alloc(Proc::Not(proc))
    }

    pub fn alloc_neg(&'ast self, proc: &'ast Proc) -> &'ast Proc {
        self.arena.alloc(Proc::Neg(proc))
    }

    pub fn alloc_negation(&'ast self, proc: &'ast Proc) -> &'ast Proc {
        self.arena.alloc(Proc::Negation(proc))
    }

    pub fn alloc_list(&'ast self, procs: Vec<AnnProc<'ast>>) -> &'ast Proc {
        self.arena.alloc(Proc::Collection(Collection::List {
            elements: procs,
            cont: None,
        }))
    }

    pub fn alloc_list_with_remainder(
        &'ast self,
        procs: Vec<AnnProc<'ast>>,
        remainder: ProcRemainder<'ast>,
    ) -> &'ast Proc {
        self.arena.alloc(Proc::Collection(Collection::List {
            elements: procs,
            cont: Some(remainder),
        }))
    }

    pub fn alloc_set(&'ast self, procs: Vec<AnnProc<'ast>>) -> &'ast Proc {
        self.arena.alloc(Proc::Collection(Collection::Set {
            elements: procs,
            cont: None,
        }))
    }

    pub fn alloc_set_with_remainder(
        &'ast self,
        procs: Vec<AnnProc<'ast>>,
        remainder: ProcRemainder<'ast>,
    ) -> &'ast Proc {
        self.arena.alloc(Proc::Collection(Collection::Set {
            elements: procs,
            cont: Some(remainder),
        }))
    }

    pub fn alloc_tuple(&'ast self, procs: Vec<AnnProc<'ast>>) -> &'ast Proc {
        self.arena
            .alloc(Proc::Collection(Collection::Tuple { elements: procs }))
    }

    pub fn alloc_map(&'ast self, elements: Vec<KeyValuePair<'ast>>) -> &'ast Proc {
        self.arena.alloc(Proc::Collection(Collection::Map {
            pairs: elements,
            cont: None,
        }))
    }

    pub fn alloc_map_with_remainder(
        &'ast self,
        elements: Vec<KeyValuePair<'ast>>,
        remainder: ProcRemainder<'ast>,
    ) -> &'ast Proc {
        self.arena.alloc(Proc::Collection(Collection::Map {
            pairs: elements,
            cont: Some(remainder),
        }))
    }

    pub fn alloc_choice(&'ast self, branches: Vec<Branch<'ast>>) -> &'ast Proc {
        self.arena.alloc(Proc::Choice { branches })
    }

    pub fn get_node_value(&'ast self, node: &Node) -> Result<&'ast str, InterpreterError> {
        node.utf8_text(self.source_code)
            .or_else(|e| Err(InterpreterError::from_utf8_error(&e, node)))
    }

    pub fn fold_procs_into_par<I>(&'ast self, procs: &'ast [Proc], positions: I) -> &'ast Proc
    where
        I: IntoIterator<Item = SourcePosition>,
    {
        fn ann<'a>(
            proc: &'a Proc,
            pos_stream: &mut dyn Iterator<Item = SourcePosition>,
        ) -> AnnProc<'a> {
            proc.annotate(pos_stream.next().unwrap())
        }

        let mut eternal_positions = positions
            .into_iter()
            .chain(std::iter::repeat(SourcePosition::default()));
        match procs.len() {
            0 => &NIL,
            1 => panic!("Impossible to form a par from a single process"),
            _ => {
                let init = self.alloc_par(
                    ann(&procs[0], &mut eternal_positions),
                    ann(&procs[1], &mut eternal_positions),
                );
                (&procs[2..]).iter().fold(init, |acc, proc| {
                    self.alloc_par(
                        ann(acc, &mut eternal_positions),
                        ann(proc, &mut eternal_positions),
                    )
                })
            }
        }
    }
}

pub const NIL: Proc = Proc::Nil;
pub const GTRUE: Proc = Proc::BoolLiteral(true);
pub const GFALSE: Proc = Proc::BoolLiteral(false);
pub const WILD: Proc = Proc::ProcVar(Var::Wildcard);
pub const NAME_WILD: Name = Name::ProcVar(Var::Wildcard);
pub const TYPE_URI: Proc = Proc::SimpleType(SimpleType::Uri);
pub const TYPE_STRING: Proc = Proc::SimpleType(SimpleType::String);
pub const TYPE_INT: Proc = Proc::SimpleType(SimpleType::Int);
pub const TYPE_BOOL: Proc = Proc::SimpleType(SimpleType::Bool);
pub const TYPE_BYTEA: Proc = Proc::SimpleType(SimpleType::ByteArray);

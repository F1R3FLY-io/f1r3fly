// See rholang/src/main/scala/coop/rchain/rholang/interpreter/errors.scala
use std::{fmt, num::ParseIntError, str::Utf8Error};

use itertools::Itertools;
use rspace_plus_plus::rspace::errors::RSpaceError;
use tree_sitter::Node;

use super::compiler::{exports::SourcePosition, free_map::ConnectiveInstance, Context};

// PartialEq here is needed for testing purposes
#[derive(Debug, Clone, PartialEq)]
pub enum InterpreterError {
    RSpaceError(RSpaceError),
    BugFoundError(String),
    UndefinedRequiredProtobufFieldError,
    NormalizerError(String),
    SyntaxError(String),
    LexerError(String),
    ParserError(String, String, SourcePosition),
    UnboundVariableRef {
        var_name: String,
        pos: SourcePosition,
    },
    UnexpectedNameContext {
        var_name: String,
        proc_var_source_position: SourcePosition,
        name_source_position: SourcePosition,
    },
    UnexpectedReuseOfNameContextFree {
        var_name: String,
        first_use: SourcePosition,
        second_use: SourcePosition,
    },
    UnexpectedProcContext {
        var_name: String,
        name_var_source_position: SourcePosition,
        process_source_position: SourcePosition,
    },
    UnexpectedReuseOfProcContextFree {
        var_name: String,
        first_use: SourcePosition,
        second_use: SourcePosition,
    },
    UnexpectedBundleContent {
        wildcards: Vec<SourcePosition>,
        free_vars: Vec<Context<String>>,
    },
    TopLevelConnectiveInBundle(SourcePosition),
    UnrecognizedNormalizerError(String),
    OutOfPhlogistonsError,
    TopLevelWildcardsNotAllowedError(Vec<SourcePosition>),
    TopLevelFreeVariablesNotAllowedError(Vec<Context<String>>),
    TopLevelLogicalConnectivesNotAllowedError(Vec<Context<ConnectiveInstance>>),
    SubstituteError(String),
    PatternReceiveError(Context<ConnectiveInstance>),
    SetupError(String),
    UnrecognizedInterpreterError(String),
    SortMatchError(String),
    ReduceError(String),
    MethodNotDefined {
        method: String,
        other_type: String,
    },
    MethodArgumentNumberMismatch {
        method: String,
        expected: usize,
        actual: usize,
    },
    OperatorNotDefined {
        op: String,
        other_type: String,
    },
    OperatorExpectedError {
        op: String,
        expected: String,
        other_type: String,
    },
    AggregateError {
        interpreter_errors: Vec<InterpreterError>,
    },
    ReceiveOnSameChannelsError {
        line: usize,
        col: usize,
    },
}

impl InterpreterError {
    pub fn parser_expects_named_node_at(
        index: usize,
        of: &Node,
        context: &Node,
    ) -> InterpreterError {
        let string = format!(
            "Expected a named node in {:?} at index {:?}",
            of.kind(),
            index
        );
        InterpreterError::parser_error_from_node(&string, context)
    }

    pub fn parser_expects_kind(kind: &str, context: &Node) -> InterpreterError {
        let string = format!("Expected {:?} in {:?}", kind, context.kind());
        InterpreterError::parser_error_from_node(&string, context)
    }

    pub fn parser_unexpected_node_kind(kind: &str, of: &str, context: &Node) -> InterpreterError {
        let string = format!("Unexpected '{:?}' in {:?}", kind, of);
        InterpreterError::parser_error_from_node(&string, context)
    }

    pub fn parser_unexpected_value(value: &str, context: &Node) -> InterpreterError {
        let string = format!("Unexpected value: '{:?}' of {:?}", value, context.kind());
        InterpreterError::parser_error_from_node(&string, context)
    }

    pub fn parser_expects_child(of: &Node, context: &Node) -> InterpreterError {
        let string = format!("No child node of {:?}", of.kind());
        InterpreterError::ParserError(string, context.to_sexp(), of.start_position().into())
    }

    pub fn parser_expects_named_node_inside(node: &Node, context: &Node) -> InterpreterError {
        let string = format!("Expected a process inside {:?}", node.kind());
        InterpreterError::ParserError(string, context.to_sexp(), node.start_position().into())
    }

    pub fn parser_expects_rule(name: &str, context: &Node) -> InterpreterError {
        let string = format!("Expected {:?}", name);
        InterpreterError::parser_error_from_node(&string, context)
    }

    pub fn parser_did_not_recognize(context: &Node) -> InterpreterError {
        let string = format!("Unrecognizable process: {:?}", context.kind());
        InterpreterError::parser_error_from_node(&string, context)
    }

    pub fn parser_expects_field(name: &str, context: &Node) -> InterpreterError {
        let string = format!(
            "Did not find expected field: {:?} in {:?}",
            name,
            context.kind()
        );
        InterpreterError::parser_error_from_node(&string, context)
    }

    pub fn from_parse_int_error(
        literal: &str,
        e: &ParseIntError,
        context: &Node,
    ) -> InterpreterError {
        let string = format!("Failed to convert {:?} into i64. Error: {:?}", literal, e);
        InterpreterError::parser_error_from_node(&string, context)
    }

    pub fn from_utf8_error(e: &Utf8Error, context: &Node) -> InterpreterError {
        let string = format!("Failed to get node value. Error: {:?}", e);
        InterpreterError::parser_error_from_node(&string, context)
    }

    pub fn parser_error_from_node(msg: &str, context: &Node) -> InterpreterError {
        InterpreterError::ParserError(
            msg.to_string(),
            context.to_sexp(),
            context.start_position().into(),
        )
    }
}

impl fmt::Display for InterpreterError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            InterpreterError::BugFoundError(msg) => write!(f, "Bug found: {}", msg),

            InterpreterError::RSpaceError(msg) => write!(f, "RSpace Error: {}", msg),

            InterpreterError::UndefinedRequiredProtobufFieldError => {
                write!(f, "A parsed Protobuf field was None, should be Some",)
            }

            InterpreterError::NormalizerError(msg) => write!(f, "Normalizer error: {}", msg),

            InterpreterError::SyntaxError(msg) => write!(f, "Syntax error: {}", msg),

            InterpreterError::LexerError(msg) => write!(f, "Lexer error: {}", msg),

            InterpreterError::ParserError(msg, context, pos) => {
                write!(
                    f,
                    "Parser error: {}\nFull context ({}): {}",
                    msg, pos, context
                )
            }

            InterpreterError::UnboundVariableRef { var_name, pos } => {
                write!(
                    f,
                    "Variable reference: ={} at {}:{} is unbound.",
                    var_name, pos.row, pos.column
                )
            }

            InterpreterError::UnexpectedNameContext {
                var_name,
                proc_var_source_position,
                name_source_position,
            } => {
                write!(
                    f,
                    "Proc variable: {} at {} used in Name context at {}",
                    var_name, proc_var_source_position, name_source_position
                )
            }

            InterpreterError::UnexpectedReuseOfNameContextFree {
                var_name,
                first_use,
                second_use,
            } => {
                write!(
                    f,
                    "Free variable {} is used twice as a binder (at {} and {}) in name context.",
                    var_name, first_use, second_use
                )
            }

            InterpreterError::UnexpectedProcContext {
                var_name,
                name_var_source_position,
                process_source_position,
            } => {
                write!(
                    f,
                    "Name variable: {} at {} used in process context at {}",
                    var_name, name_var_source_position, process_source_position
                )
            }

            InterpreterError::UnexpectedReuseOfProcContextFree {
                var_name,
                first_use,
                second_use,
            } => {
                write!(
                    f,
                    "Free variable {} is used twice as a binder (at {} and {}) in process context.",
                    var_name, first_use, second_use
                )
            }

            InterpreterError::UnexpectedBundleContent {
                wildcards,
                free_vars,
            } => {
                let mut err_msg = String::new();

                if !wildcards.is_empty() {
                    err_msg.push_str("Wildcards positions: ");
                    err_msg.push_str(&wildcards.iter().join("; "));
                    err_msg.push('.');
                }

                if !free_vars.is_empty() {
                    err_msg.push_str("Free variables positions: ");
                    err_msg.push_str(&free_vars.iter().join("; "));
                    err_msg.push('.');
                }

                write!(
                    f,
                    "Bundle's content must not have free variables or wildcards. {}",
                    err_msg
                )
            }
            InterpreterError::TopLevelConnectiveInBundle(pos) => {
                write!(f, "Illegal top-level connective in bundle at {}.", pos)
            }

            InterpreterError::UnrecognizedNormalizerError(msg) => {
                write!(f, "Unrecognized normalizer error: {}", msg)
            }

            InterpreterError::OutOfPhlogistonsError => {
                write!(f, "Computation ran out of phlogistons.")
            }

            InterpreterError::TopLevelWildcardsNotAllowedError(wildcards) => {
                write!(
                    f,
                    "Top level wildcards are not allowed: {}",
                    wildcards.iter().join("; ")
                )
            }

            InterpreterError::TopLevelFreeVariablesNotAllowedError(free_vars) => {
                write!(
                    f,
                    "Top level free variables are not allowed: {}",
                    free_vars.iter().join("; ")
                )
            }

            InterpreterError::TopLevelLogicalConnectivesNotAllowedError(connectives) => write!(
                f,
                "Top level logical connectives are not allowed: {}",
                connectives.iter().join("; ")
            ),

            InterpreterError::SubstituteError(msg) => write!(f, "Substitute error: {}", msg),

            InterpreterError::PatternReceiveError(connective) => write!(
                f,
                "Invalid pattern in the receive: {}. Only logical AND is allowed.",
                connective
            ),

            InterpreterError::SetupError(msg) => write!(f, "Setup error: {}", msg),

            InterpreterError::UnrecognizedInterpreterError(_) => {
                write!(f, "Unrecognized interpreter error.")
            }

            InterpreterError::SortMatchError(msg) => write!(f, "Sort match error: {}", msg),

            InterpreterError::ReduceError(msg) => write!(f, "Reduce error: {}", msg),

            InterpreterError::MethodNotDefined { method, other_type } => write!(
                f,
                "Error: Method `{}` is not defined on {}.",
                method, other_type
            ),

            InterpreterError::MethodArgumentNumberMismatch {
                method,
                expected,
                actual,
            } => {
                write!(
                    f,
                    "Error: Method `{}` expects {} Par argument(s), but got {} argument(s).",
                    method, expected, actual
                )
            }

            InterpreterError::OperatorNotDefined { op, other_type } => write!(
                f,
                "Error: Operator `{}` is not defined on {}.",
                op, other_type
            ),

            InterpreterError::OperatorExpectedError {
                op,
                expected: _,
                other_type,
            } => write!(
                f,
                "Error: Operator `{}` is not defined on {}.",
                op, other_type
            ),

            InterpreterError::AggregateError { interpreter_errors } => {
                let error_messages = interpreter_errors
                    .iter()
                    .map(|e| format!("{:?}", e))
                    .collect::<Vec<_>>();

                write!(f, "Error: Aggregate Error\n{}", error_messages.join("\n"))
            }

            InterpreterError::ReceiveOnSameChannelsError { line, col } => {
                write!(
                    f,
                    "Receiving on the same channels is currently not allowed (at {}:{}).",
                    line, col
                )
            }
        }
    }
}

impl From<RSpaceError> for InterpreterError {
    fn from(err: RSpaceError) -> InterpreterError {
        InterpreterError::RSpaceError(err)
    }
}

impl From<InterpreterError> for RSpaceError {
    fn from(error: InterpreterError) -> Self {
        RSpaceError::InterpreterError(error.to_string())
    }
}

impl std::error::Error for InterpreterError {}

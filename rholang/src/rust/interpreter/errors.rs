// See rholang/src/main/scala/coop/rchain/rholang/interpreter/errors.scala
use std::fmt;

// PartialEq here is needed for testing purposes
#[derive(Debug, Clone, PartialEq)]
pub enum InterpreterError {
    BugFoundError(String),
    UndefinedRequiredProtobufFieldError,
    NormalizerError(String),
    SyntaxError(String),
    LexerError(String),
    ParserError(String),
    UnboundVariableRef {
        var_name: String,
        line: usize,
        col: usize,
    },
    UnexpectedNameContext {
        var_name: String,
        proc_var_source_position: String,
        name_source_position: String,
    },
    UnexpectedReuseOfNameContextFree {
        var_name: String,
        first_use: String,
        second_use: String,
    },
    UnexpectedProcContext {
        var_name: String,
        name_var_source_position: String,
        process_source_position: String,
    },
    UnexpectedReuseOfProcContextFree {
        var_name: String,
        first_use: String,
        second_use: String,
    },
    UnexpectedBundleContent(String),
    UnrecognizedNormalizerError(String),
    OutOfPhlogistonsError,
    TopLevelWildcardsNotAllowedError(String),
    TopLevelFreeVariablesNotAllowedError(String),
    TopLevelLogicalConnectivesNotAllowedError(String),
    SubstituteError(String),
    PatternReceiveError(String),
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

impl fmt::Display for InterpreterError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            InterpreterError::BugFoundError(msg) => write!(f, "Bug found: {}", msg),

            InterpreterError::UndefinedRequiredProtobufFieldError => {
                write!(f, "A parsed Protobuf field was None, should be Some",)
            }

            InterpreterError::NormalizerError(msg) => write!(f, "Normalizer error: {}", msg),

            InterpreterError::SyntaxError(msg) => write!(f, "Syntax error: {}", msg),

            InterpreterError::LexerError(msg) => write!(f, "Lexer error: {}", msg),

            InterpreterError::ParserError(msg) => write!(f, "Parser error: {}", msg),

            InterpreterError::UnboundVariableRef {
                var_name,
                line,
                col,
            } => {
                write!(
                    f,
                    "Variable reference: ={} at {}:{} is unbound.",
                    var_name, line, col
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

            InterpreterError::UnexpectedBundleContent(msg) => {
                write!(f, "Unexpected bundle content: {}", msg)
            }

            InterpreterError::UnrecognizedNormalizerError(msg) => {
                write!(f, "Unrecognized normalizer error: {}", msg)
            }

            InterpreterError::OutOfPhlogistonsError => {
                write!(f, "Computation ran out of phlogistons.")
            }

            InterpreterError::TopLevelWildcardsNotAllowedError(wildcards) => {
                write!(f, "Top level wildcards are not allowed: {}", wildcards)
            }

            InterpreterError::TopLevelFreeVariablesNotAllowedError(free_vars) => {
                write!(f, "Top level free variables are not allowed: {}", free_vars)
            }

            InterpreterError::TopLevelLogicalConnectivesNotAllowedError(connectives) => write!(
                f,
                "Top level logical connectives are not allowed: {}",
                connectives
            ),

            InterpreterError::SubstituteError(msg) => write!(f, "Substitute error: {}", msg),

            InterpreterError::PatternReceiveError(connectives) => write!(
                f,
                "Invalid pattern in the receive: {}. Only logical AND is allowed.",
                connectives
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

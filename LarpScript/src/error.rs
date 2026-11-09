use crate::parser::Rule;
use std::fmt;

#[derive(Debug)]
pub enum RuntimeError {
    InvalidNumber,
    InvalidBoolean,
    InvalidOperandType,
    DivisionByZero,
    UnexpectedToken(Rule),
    UndefinedVariable(String),
    DuplicateVariable(String),
    ConstantReassignment(String),
    MalformedStatement,
    InvalidCondition,
    BreakOutsideLoop,
    ContinueOutsideLoop,
    ReturnOutsideFunction,
    InvalidIterable,
    UndefinedFunction(String),
    ArgumentCountMismatch {
        name: String,
        expected: usize,
        received: usize,
    },
}

impl fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidNumber => write!(f, "Invalid number"),
            Self::InvalidBoolean => write!(f, "Invalid boolean"),
            Self::InvalidOperandType => write!(f, "Invalid operand type"),
            Self::DivisionByZero => write!(f, "Division by zero"),
            Self::UnexpectedToken(rule) => write!(f, "Unexpected token: {rule:?}"),
            Self::UndefinedVariable(name) => write!(f, "Undefined variable: {name}"),
            Self::DuplicateVariable(name) => write!(f, "Variable already exists: {name}"),
            Self::ConstantReassignment(name) => write!(f, "Cannot reassign constant: {name}"),
            Self::MalformedStatement => write!(f, "Malformed statement"),
            Self::InvalidCondition => write!(f, "Condition must be a boolean"),
            Self::BreakOutsideLoop => write!(f, "'bet' used outside a loop"),
            Self::ContinueOutsideLoop => write!(f, "'clown' used outside a loop"),
            Self::ReturnOutsideFunction => write!(f, "'mog' used outside a function"),
            Self::InvalidIterable => write!(
                f,
                "'grind' needs a non-negative whole number or a string"
            ),
            Self::UndefinedFunction(name) => write!(f, "Undefined function: {name}"),
            Self::ArgumentCountMismatch {
                name,
                expected,
                received,
            } => {
                write!(
                    f,
                    "{name} expects {expected} arguments, received {received}"
                )
            }
        }
    }
}

impl std::error::Error for RuntimeError {}

//! DynamoDB expression parsing and evaluation.
//!
//! This module provides a complete implementation of DynamoDB's expression language,
//! supporting condition/filter/key-condition expressions, update expressions, and
//! projection expressions. The pipeline is:
//!
//! 1. **Lexing**: Tokenize the expression string into a token stream.
//! 2. **Parsing**: Build an AST from the token stream using recursive descent.
//! 3. **Evaluation**: Walk the AST to evaluate conditions, apply updates, or project attributes.

pub mod ast;
pub mod evaluator;
pub mod parser;

pub use ast::{AttributePath, Expr, Operand, PathElement, UpdateExpr};
pub use evaluator::EvalContext;
pub use parser::{ExpressionError, parse_condition, parse_projection, parse_update};
